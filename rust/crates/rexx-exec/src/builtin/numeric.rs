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
//!
//! # Every one of them is a method call on its first argument
//!
//! `BUILTIN(ABS)` and its six neighbours (`expression/BuiltinFunctions.cpp`)
//! all have the same body: take argument 1, and send it the method of the same
//! name with the remaining arguments. That is not an implementation detail --
//! it is what a program sees, because the two layers raise **different
//! errors**:
//!
//! ```text
//! max('a',1,3)   93.943   MAX method target must be a number; found "a".
//! max(1,'a',3)   93.904   Method argument 1 must be a number; found "a".
//! max(1,2,'a')   93.904   Method argument 2 must be a number; found "a".
//! ```
//!
//! Argument 1 answers with the *builtin's* name and a different sub-code from
//! arguments 2 and up, and the position those report is the **method's** own,
//! one lower than the call's. `RexxString`'s `ArithmeticMethod` macro
//! (`classes/StringClass.cpp:1060`) is where the first of those comes from:
//! it converts the target and reports 93.943 naming the method when the
//! conversion fails, and every one of the seven goes through it.
//!
//! # `41.1` is never one of these builtins' own errors
//!
//! `40.x` is rc 216 and `93.x` is rc 163, as they are for the string
//! families. `41.1` is rc 215 and turns up in `SIGN`'s test group, which
//! makes it look like a third code this family raises. It is not: a
//! non-numeric value reaches arithmetic *before* the call when the argument
//! expression contains an operator, and only after it otherwise. Measured,
//! `sign(-1E1234567890)` is 41.1 because the unary minus runs first, while
//! `sign('-1E1234567890')` is 93.943 from inside `SIGN`. Nothing here raises
//! `41.1`.
//!
//! # What each returns, and why only some of them capture D15's pair
//!
//! `FORMAT` and `TRUNC` build a `RexxString` (`formatInternal` and
//! `truncInternal` both end in `raw_string`), so their results are text and a
//! later `NUMERIC DIGITS` cannot reshape them. `RANDOM` answers a
//! `RexxInteger` (`new_integer`), whose spelling is likewise its own and
//! fixed -- measured, `numeric digits 3 ; say random(12345,12345)` is
//! `12345` and not `1.23E+4` -- so it is built as text here too, the rule
//! `WORDS` and `LENGTH` already follow. `ABS`, `SIGN`, `MAX` and `MIN`
//! produce numbers, and those capture the `DIGITS`/`FORM` pair in force at
//! the call (D15). Measured with the setting changed in between:
//!
//! ```rexx
//! numeric digits 5 ; x = max(123456789,1) ; numeric digits 12 ; say x
//!   -> 1.2346E+8, not 123456789
//! numeric form engineering ; x = max(1e10,1) ; numeric form scientific ; say x
//!   -> 10E+9, not 1E+10
//! numeric digits 3 ; x = format(1.23456,,4) ; numeric digits 9 ; say x
//!   -> 1.2300, which is text and could not have moved
//! ```
//!
//! # The ones that take a number round it first, and never raise LOSTDIGITS
//!
//! `ABS`, `TRUNC`, `FORMAT`, `MAX` and `MIN` each begin with
//! `prepareNumber(digits, ROUND)`, so an argument wider than the current
//! `NUMERIC DIGITS` is silently reduced rather than reported. Measured:
//! `numeric digits 3 ; trunc(123456,2)` is `123000.00`, and
//! `keyword/LOSTDIGITS.testGroup:388` asserts `TRUNC` raises no condition for
//! it. `SIGN` is exempt because rounding cannot change a sign, and `RANDOM`
//! takes no number at all -- its three arguments are integers.

use std::cmp::Ordering;

use rexx_core::ObjRef;
use rexx_num::{CompareOp, Form, Number, compare_decoded};

use super::{arg, buffer, required_string, whole_number};
use crate::Interp;
use crate::error::{Failure, Raised};

/// The `NUMERIC` settings this family reads, fetched once per call.
///
/// All three, because `MAX`/`MIN` need `FUZZ` for their comparison where the
/// other five do not: measured, `numeric fuzz 3 ; max(100000000.0,100000001)`
/// is `100000000` -- the two compare equal at the reduced precision, so the
/// target keeps the answer -- while the same call at `fuzz 0` is `100000001`.
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
///
/// `found` is the value's own rendered bytes, so a value whose spelling and
/// rendering differ reports the rendering -- the same rule
/// `Raised::argument_not_whole` carries its own measurement for.
fn target_number(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
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
///
/// `method_position` is the position the 93.906 message names, which is the
/// *method's* own numbering: one lower than the call's, because argument 1 is
/// the method target rather than a method argument. Measured,
/// `trunc(1.5,-1)` and `format(1,-1)` both report `Method argument 1` for
/// their own argument 2.
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
///
/// **Both refusals are Error 5 at rc 251, and both are measured.** The oracle
/// reaches this by being refused by the allocator rather than by testing
/// against a limit, so this asks the same question the same way (see
/// `Raised::system_resources`): `format(1,3000000000)` and
/// `trunc(1,4294967296)` are both `System resources exhausted.`, while
/// `format(1,999999999)` and `trunc(1,999999999)` succeed and produce results
/// of 999,999,999 and 1,000,000,001 bytes.
///
/// The reservation is released again before the result is built, so the peak
/// is one buffer of this size rather than two.
fn padding_width(value: i64) -> Result<u32, Failure> {
    let width = u32::try_from(value).map_err(|_| Failure::from(Raised::system_resources()))?;
    buffer(width as usize)?;
    Ok(width)
}

// ---- ABS and SIGN ----

/// `ABS(number)`.
pub(crate) fn abs(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Numeric { digits, form, .. } = current(interp);
    let value = target_number(interp, name, args)?;
    // The rounding is the oracle's `copyForCurrentSettings`, and it is not
    // skipped for an already-positive value: measured, `numeric digits 3 ;
    // abs(1.23456)` is `1.23`, the same answer `abs(-1.23456)` gives.
    Ok(interp.number(value.abs().round_to(digits), saturate(digits), form))
}

/// `SIGN(number)`.
pub(crate) fn sign(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Numeric { digits, form, .. } = current(interp);
    let value = target_number(interp, name, args)?;
    // No rounding here, unlike `ABS` above: rounding cannot turn a non-zero
    // value into a zero one, so `NumberString::Sign`'s own `copyIfNecessary`
    // can never change the answer. Measured, `sign(-0.0)` is `0` -- every
    // spelling of zero is unsigned, which is `Number::signum`'s rule.
    let answer = Number::parse(&value.signum().to_string()).expect("-1, 0 and 1 are all numbers");
    Ok(interp.number(answer, saturate(digits), form))
}

// ---- TRUNC and FORMAT ----

/// `TRUNC(number, decimals)`.
///
/// **The three checks are in the oracle's own order, which is observable**:
/// argument 2's *type* first, then argument 1's, then argument 2's *range*.
/// Measured, three programs that differ only in argument 2:
///
/// ```text
/// trunc('AB.CD','V')   40.12   TRUNC argument 2 must be a whole number; found "V".
/// trunc('AB.CD',-1)    93.943  TRUNC method target must be a number; found "AB.CD".
/// trunc(1.5,-1)        93.906  Method argument 1 must be zero or a positive whole number.
/// ```
///
/// That falls out of where each check lives rather than from a chosen
/// sequence: `BUILTIN(TRUNC)` converts the arguments (40.12), `RexxString::
/// trunc` converts the target (93.943), and `NumberString::trunc`'s own
/// `optionalNonNegative` runs last (93.906).
pub(crate) fn trunc(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Numeric { digits, .. } = current(interp);
    let places = whole_number(interp, name, args, 2)?;
    let value = target_number(interp, name, args)?;
    let places = padding_width(non_negative(places, 1)?.unwrap_or(0))?;
    Ok(interp.text_owned(value.trunc(digits, places).into_bytes()))
}

/// `FORMAT(number, before, after, expp, expt)`.
///
/// Same three-layer order as [`trunc`], with all four optional arguments
/// type-checked before the target is: measured, `format('a','x')` is 40.12
/// naming argument 2, not the 93.943 the target alone would give, and
/// `format('a',1,-1)` is 93.943 rather than argument 3's 93.906.
///
/// `expp` and `expt` are not symmetric and the difference is easy to get
/// backwards. `expp == 0` **suppresses** exponential form and beats
/// everything else; `expt == 0` merely sets the trigger to zero, which almost
/// always forces it. Measured: `format(12345,,,0,0)` is `12345` while
/// `format(12345,,,2,0)` is `1.2345E+04`.
pub(crate) fn format(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Numeric { digits, form, .. } = current(interp);
    let before = whole_number(interp, name, args, 2)?;
    let after = whole_number(interp, name, args, 3)?;
    let expp = whole_number(interp, name, args, 4)?;
    let expt = whole_number(interp, name, args, 5)?;
    let value = target_number(interp, name, args)?;

    let before = non_negative(before, 1)?;
    let after = non_negative(after, 2)?;
    let expp = non_negative(expp, 3)?;
    let expt = non_negative(expt, 4)?;

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
    //
    // Saturating into `u32` to *ask* the question is exact, because only
    // `expp == 0` changes the answer and zero is never saturated.
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
    Ok(interp.text_owned(text.into_bytes()))
}

// ---- MAX and MIN ----

/// Which end of the range [`max_min`] is looking for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Extreme {
    Max,
    Min,
}

impl Extreme {
    /// The comparison outcome that makes a candidate the new answer.
    ///
    /// Strict, so a tie keeps the earlier value -- which the oracle's
    /// `maxMin` states explicitly (`rc > 0 && compResult > 0`) and its
    /// integer path repeats with a bare `v > maxValue`.
    fn wins(self, ordering: Ordering) -> bool {
        match self {
            Extreme::Max => ordering == Ordering::Greater,
            Extreme::Min => ordering == Ordering::Less,
        }
    }

    /// [`wins`] as a `rexx-num` predicate, for the general path, where the
    /// comparison honours `FUZZ` and so cannot be a bare `Ord`.
    ///
    /// **Both ends need their own strict operator; one of them plus a
    /// negation is wrong.** `CompareOp::Greater` answers `false` for *equal*
    /// as well as for *less*, so reading a `false` as "the candidate wins"
    /// for `MIN` swaps on every tie -- measured, `min(1,1.0)` is `1` on the
    /// oracle and was `1.0` here until this took `Less` for `MIN`.
    ///
    /// [`wins`]: Extreme::wins
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
///
/// **This distinction is observable and this crate has no type that carries
/// it, so it is read back off the value's own rendering.** The oracle's rule
/// is a property of a literal's *spelling*: `LanguageParser::addVariable`
/// builds an integer object exactly when the token `isIntegerConstant()`,
/// which `Scanner.cpp:1546` sets for a run of digits no longer than
/// [`REXX_INTEGER_DIGITS`] with no leading zero unless the whole symbol is
/// `0`. A leading `-` is admitted here because the negation of such a literal
/// stays an integer object -- measured, `max(-1,,3)` and `max(1,,3)` answer
/// alike.
///
/// Measured, the shapes this must and must not accept, each through
/// `max(X,,3)`, whose two answers are 93.903 for an integer target and 40.5
/// for anything else: `1`, `+1`, `-1`, `1+0`, `10/2` and `2*3` are integers;
/// `01`, `1.0`, `1.`, `1e1`, `1.0+0` and a 19-digit literal are not.
///
/// **The one shape it gets wrong is a string that spells an integer**:
/// `max('1',,3)` and `max(word('1 2',1),,3)` are 40.5 on the oracle, because
/// a `RexxString` never becomes an integer object however it reads, and 93.903
/// here. Nothing in this crate's value model separates `1` from `'1'` --
/// `eval.rs` builds both as text -- so no rule available at this layer can
/// tell them apart.
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
///
/// Measured through the same `max(X,,3)` pair as [`integer_object`]:
/// `numeric digits 9 ; max(12345,,3)` is 93.903 and `numeric digits 3 ;
/// max(12345,,3)` is 40.5, for the identical call.
fn valid_under(value: i64, digits: u64) -> bool {
    let width = digits.min(REXX_INTEGER_DIGITS as u64) as u32;
    // `10i64.pow(18)` is the largest this can compute and fits comfortably.
    value.unsigned_abs() < 10u64.pow(width)
}

/// `MAX(number, ...)`.
pub(crate) fn max(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    max_min(interp, name, args, Extreme::Max)
}

/// `MIN(number, ...)`.
pub(crate) fn min(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    max_min(interp, name, args, Extreme::Min)
}

/// The body both of them share, and the one place in this file where the
/// answer depends on which *representation* the target has rather than only
/// on its value.
///
/// `RexxInteger::Max`/`::Min` (`classes/IntegerClass.cpp:1578`) is a fast path
/// over integer objects that falls back to `NumberString::maxMin`
/// (`classes/NumberStringMath.cpp:240`) the moment any operand is not one.
/// The two disagree about an **omitted** argument, and a program can see it:
///
/// ```text
/// max(1,,3)     93.903  Missing argument in method; argument 0 is required.   rc 163
/// max(1.0,,3)   40.5    Missing argument in invocation of MAX; argument 1 ...  rc 216
/// ```
///
/// Different major, different position base, different exit code, for calls
/// that differ only in a decimal point. See [`integer_object`] for what this
/// crate can and cannot tell apart there.
fn max_min(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
    want: Extreme,
) -> Result<ObjRef, Failure> {
    let Numeric { digits, fuzz, form } = current(interp);
    let target = arg(args, 1).expect("check_arity admitted the required first argument");
    let rest = &args[1..];

    if let Some(answer) = integer_path(interp, target, rest, digits, want)? {
        return Ok(answer);
    }

    let mut best = target_number(interp, name, args)?.round_to(digits);
    let mut best_text = required_string(interp, args, 1);
    for (index, slot) in rest.iter().enumerate() {
        let Some(candidate) = slot else {
            return Err(Raised::missing_argument(name, index + 1).into());
        };
        let found = interp.to_text(*candidate).into_owned();
        let Ok(number) = interp.to_number(*candidate) else {
            return Err(Raised::method_argument_not_a_number(index + 1, &found).into());
        };
        let number = number.round_to(digits);
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
///
/// The answer is the winning **object**, not a recomputed value, which is
/// what the C++ returns (`return maxObject`) and is why a lone `MIN` argument
/// can come back untouched by the precision in force.
///
/// **`MIN` answers a lone target before testing it against `DIGITS` and `MAX`
/// does not**, which is deliberate upstream -- `RexxInteger::Min` carries a
/// comment saying the check was moved so that `RexxInteger.testGroup` can
/// tell the two representations apart from inside Rexx. Measured, and the
/// only place in this family where `MAX` and `MIN` answer differently for
/// reasons other than direction: at `numeric digits 3`, `min(12345)` is
/// `12345` while `max(12345)` is `1.23E+4`.
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
///
/// **The requirement is a stream, not a seed.** `bif/RANDOM.testGroup` seeds
/// once, makes 99 further *unseeded* calls, re-seeds with the same value and
/// repeats, and requires all 100 numbers to match; a generator that re-seeds
/// on every call satisfies "seedable and deterministic" and fails that. The
/// state therefore lives on the interpreter (`Interp::random_seed`) and every
/// call advances it.
///
/// Reproducing the generator exactly rather than merely being deterministic
/// buys a real property: a *seeded* stream matches the oracle number for
/// number, across processes. Measured -- `random(1,999999999,12345)` is
/// `776163098` on `build/bin/rexx`, and the following two unseeded calls in
/// the same program are `950445098` and `552120333`, identically on three
/// separate runs. An unseeded first call is not reproducible on either side.
pub(crate) fn random(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let minimum = whole_number(interp, name, args, 1)?;
    let maximum = whole_number(interp, name, args, 2)?;
    let seed = whole_number(interp, name, args, 3)?;

    // The seed is validated and *applied* before the range is looked at,
    // which is `RexxActivation::random`'s own first statement. A call that
    // then fails its range check has still moved the stream on.
    //
    // Every one of this builtin's messages substitutes the **converted**
    // integer and not the argument's own text, because the C++ hands
    // `reportException` the `RexxInteger` rather than the operand. Measured
    // with the two forms apart: `random(1,2,'-1.0')` reports `found "-1"`,
    // and `random('5.0','1.0')` reports `("5")` and `("1")`.
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
    //
    // `BUILTIN(RANDOM)`'s own special case for `argcount == 2` with both
    // range arguments omitted is not reproduced, because it cannot be
    // reached and would not change the answer if it were: a trailing
    // omission is not an argument (`random(,)` arrives with none at all),
    // and the values it substitutes are `DEFAULT_MIN`/`DEFAULT_MAX` anyway.
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
    Ok(interp.text_owned(low.to_string().into_bytes()))
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
///
/// The oracle's is `Activity::getRandomSeed`, which folds the C library's
/// `rand()` in and is genuinely different per process. This does the same job
/// from the clock and the process id: what matters is only that an unseeded
/// program is not reproducible, since anything that *is* reproducible on the
/// oracle goes through a seed and starts from [`next_seed`]'s own branch
/// instead.
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
mod tests {
    use super::super::dispatch;
    use crate::plan::{BodyKey, ProgramId};
    use crate::{Activation, Interp, error::Failure};
    use rexx_parse::parse_program;
    use std::rc::Rc;

    /// An interpreter with a live top-level activation, which is where every
    /// one of these builtins reads `DIGITS`, `FUZZ` and `FORM` from. The
    /// program it activates is a `NOP`: nothing here executes an instruction,
    /// and only the settings on the frame matter.
    fn interp_with(digits: &str, form: &str, fuzz: &str) -> Interp {
        let mut interp = Interp::new();
        let program = Rc::new(parse_program(b"nop".to_vec()).expect("a NOP program parses"));
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        let activation = interp.next_activation_id();
        interp
            .activations
            .push(Activation::new(activation, program, plan, frame));
        let settings = &mut interp.activation_mut().settings;
        settings.set_digits_str(digits).expect("a legal DIGITS");
        settings.set_form_str(form).expect("a legal FORM");
        settings.set_fuzz_str(fuzz).expect("a legal FUZZ");
        interp
    }

    /// Runs `name` over `arguments`, each `None` standing for an omitted
    /// interior position, and answers the result's own bytes.
    ///
    /// Goes through [`dispatch`] rather than calling an implementation
    /// directly, so every case here exercises the arity check and the name
    /// lookup a real call would.
    fn call_in(
        interp: &mut Interp,
        name: &[u8],
        arguments: &[Option<&[u8]>],
    ) -> Result<Vec<u8>, Failure> {
        let args: Vec<_> = arguments
            .iter()
            .map(|argument| argument.map(|bytes| interp.text(bytes)))
            .collect();
        let result = dispatch(interp, name, &args).expect("a builtin name")?;
        Ok(interp.to_text(result).into_owned())
    }

    fn call_with(
        digits: &str,
        form: &str,
        name: &[u8],
        arguments: &[Option<&[u8]>],
    ) -> Result<Vec<u8>, Failure> {
        call_in(&mut interp_with(digits, form, "0"), name, arguments)
    }

    fn call(name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
        call_with("9", "SCIENTIFIC", name, arguments)
    }

    /// [`call`], for the cases whose answer is the bytes and nothing else.
    fn answer(name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
        call(name, &arguments).expect("this call succeeds")
    }

    fn answer_at(digits: &str, name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
        call_with(digits, "SCIENTIFIC", name, &arguments).expect("this call succeeds")
    }

    /// The `(major, sub)` and substitutions of the condition `name` raises.
    fn raised(name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
        raised_at("9", name, arguments)
    }

    fn raised_at(
        digits: &str,
        name: &[u8],
        arguments: &[Option<&[u8]>],
    ) -> (u16, u16, Vec<Vec<u8>>) {
        let failure = call_with(digits, "SCIENTIFIC", name, arguments).expect_err("this raises");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        (raised.number, raised.sub, raised.additional)
    }

    fn subs(values: &[&[u8]]) -> Vec<Vec<u8>> {
        values.iter().map(|bytes| bytes.to_vec()).collect()
    }

    /// Argument 1 raises a different number from arguments 2 and up, and the
    /// position those name is one lower than the call's.
    ///
    /// The whole family's shape in one test: 93.943 names the builtin,
    /// 93.904 names a *method* position. A mutation that answers 93.943 for
    /// every argument, or that reports `index + 2`, fails here.
    #[test]
    fn the_target_and_the_later_arguments_raise_different_numbers() {
        for name in [
            b"ABS".as_slice(),
            b"SIGN",
            b"TRUNC",
            b"FORMAT",
            b"MAX",
            b"MIN",
        ] {
            assert_eq!(
                raised(name, &[Some(b"abc")]),
                (93, 943, subs(&[name, b"abc"])),
                "{} did not name itself in 93.943",
                String::from_utf8_lossy(name)
            );
        }
        // The null string and a lone blank reach it too, and report exactly
        // what they are.
        assert_eq!(
            raised(b"ABS", &[Some(b"")]),
            (93, 943, subs(&[b"ABS", b""]))
        );
        assert_eq!(
            raised(b"SIGN", &[Some(b" ")]),
            (93, 943, subs(&[b"SIGN", b" "]))
        );
        // Quoted, so no arithmetic runs before the call: this is SIGN's own
        // error, not the 41.1 an unquoted `-1E1234567890` would raise from
        // the unary minus.
        assert_eq!(
            raised(b"SIGN", &[Some(b"-1E1234567890")]),
            (93, 943, subs(&[b"SIGN", b"-1E1234567890"]))
        );

        // Arguments 2 and up: a different sub-code, and the *method's* own
        // numbering.
        assert_eq!(
            raised(b"MAX", &[Some(b"1"), Some(b"a"), Some(b"3")]),
            (93, 904, subs(&[b"1", b"a"]))
        );
        assert_eq!(
            raised(b"MAX", &[Some(b"1"), Some(b"2"), Some(b"a")]),
            (93, 904, subs(&[b"2", b"a"]))
        );
        assert_eq!(
            raised(b"MIN", &[Some(b"1"), Some(b"")]),
            (93, 904, subs(&[b"1", b""]))
        );
    }

    /// An omitted `MAX` argument answers differently depending on which
    /// representation argument 1 has, and both halves are pinned.
    ///
    /// The integer path reports 93.903 with a **0-based** position; the
    /// general path reports 40.5 naming the routine and a 1-based one. A
    /// mutation that keeps only one of the two, or that increments the
    /// integer path's index, fails here.
    #[test]
    fn an_omitted_argument_answers_differently_on_the_two_paths() {
        assert_eq!(
            raised(b"MAX", &[Some(b"1"), None, Some(b"3")]),
            (93, 903, subs(&[b"0"]))
        );
        assert_eq!(
            raised(b"MAX", &[Some(b"1"), Some(b"2"), None, Some(b"4")]),
            (93, 903, subs(&[b"1"]))
        );
        assert_eq!(
            raised(
                b"MIN",
                &[Some(b"1"), Some(b"2"), Some(b"3"), None, Some(b"5")]
            ),
            (93, 903, subs(&[b"2"]))
        );

        // A target that is not an integer object takes the general path.
        assert_eq!(
            raised(b"MAX", &[Some(b"1.0"), None, Some(b"3")]),
            (40, 5, subs(&[b"MAX", b"1"]))
        );
        assert_eq!(
            raised(b"MAX", &[Some(b"1"), Some(b"2.5"), None, Some(b"4")]),
            (40, 5, subs(&[b"MAX", b"2"]))
        );
        // ...and so does one too wide for the precision in force, which is
        // the same call at two settings.
        assert_eq!(
            raised_at("9", b"MAX", &[Some(b"12345"), None, Some(b"3")]),
            (93, 903, subs(&[b"0"]))
        );
        assert_eq!(
            raised_at("3", b"MAX", &[Some(b"12345"), None, Some(b"3")]),
            (40, 5, subs(&[b"MAX", b"1"]))
        );

        // One non-integer sends the whole list back to the general path, so
        // an omission *after* it is never reached.
        assert_eq!(
            raised(b"MAX", &[Some(b"1"), Some(b"a"), None, Some(b"4")]),
            (93, 904, subs(&[b"1", b"a"]))
        );
    }

    /// A tie keeps the incumbent at **both** ends, which needs a strict
    /// operator for each.
    ///
    /// `MIN` implemented as "not greater" swaps on every equal comparison and
    /// answers `1.0` for the first case below; the oracle answers `1`. Found
    /// by the differential sweep, not by reading the code.
    #[test]
    fn a_tie_keeps_the_earlier_value_at_both_ends() {
        assert_eq!(answer(b"MIN", &[b"1", b"1.0"]), b"1");
        assert_eq!(answer(b"MAX", &[b"1", b"1.0"]), b"1");
        assert_eq!(answer(b"MIN", &[b"1.0", b"1"]), b"1.0");
        assert_eq!(answer(b"MAX", &[b"1.0", b"1"]), b"1.0");
        // The adjacent non-tie, so this cannot pass by never swapping at all.
        assert_eq!(answer(b"MAX", &[b"1", b"2.5"]), b"2.5");
        assert_eq!(answer(b"MIN", &[b"2.5", b"1"]), b"1");
    }

    /// `MIN` answers a lone target before testing it against `DIGITS` and
    /// `MAX` does not -- the one place the two differ for a reason other
    /// than direction.
    #[test]
    fn min_answers_a_lone_target_before_max_would() {
        assert_eq!(answer_at("3", b"MIN", &[b"12345"]), b"12345");
        assert_eq!(answer_at("3", b"MAX", &[b"12345"]), b"1.23E+4");
        // At a precision the value fits, the two agree again.
        assert_eq!(answer_at("9", b"MIN", &[b"12345"]), b"12345");
        assert_eq!(answer_at("9", b"MAX", &[b"12345"]), b"12345");
    }

    /// Both are variadic, and the arity row is what makes that so.
    #[test]
    fn max_and_min_take_as_many_arguments_as_they_are_given() {
        assert_eq!(
            answer(b"MAX", &[b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8"]),
            b"8"
        );
        assert_eq!(
            answer(b"MIN", &[b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8"]),
            b"1"
        );
        assert_eq!(raised(b"MAX", &[]), (40, 3, subs(&[b"MAX", b"1"])));
        assert_eq!(raised(b"MIN", &[]), (40, 3, subs(&[b"MIN", b"1"])));
    }

    /// `FORMAT` rounds half up away from zero, not to even.
    ///
    /// A banker's-rounding implementation answers `2` for the first case.
    #[test]
    fn format_rounds_half_up_away_from_zero() {
        let after = |value: &'static [u8], places: &'static [u8]| {
            call(b"FORMAT", &[Some(value), None, Some(places)]).expect("this call succeeds")
        };
        assert_eq!(after(b"2.5", b"0"), b"3");
        assert_eq!(after(b"3.5", b"0"), b"4");
        assert_eq!(after(b"-2.5", b"0"), b"-3");
        assert_eq!(after(b"1.245", b"2"), b"1.25");
        // The adjacent round-down, so this cannot pass by always rounding up.
        assert_eq!(after(b"2.4", b"0"), b"2");
    }

    /// `before == 0` never succeeds, not even for zero, while an omitted
    /// `before` renders zero happily.
    #[test]
    fn a_before_of_zero_always_fails_and_an_omitted_one_does_not() {
        assert_eq!(
            raised(b"FORMAT", &[Some(b"1"), Some(b"0")]),
            (93, 942, subs(&[b"1", b"0"]))
        );
        assert_eq!(
            raised(b"FORMAT", &[Some(b"0"), Some(b"0")]),
            (93, 942, subs(&[b"0", b"0"]))
        );
        assert_eq!(answer(b"FORMAT", &[b"0"]), b"0");
    }

    /// 93.942 substitutes the number as `FORMAT` has it when it gives up --
    /// rounded by `after`, and reframed by the exponential decision -- not
    /// the argument it was handed.
    ///
    /// A mutation that substitutes the original operand answers `1.5`, `99.996`
    /// and `123456.789` for the three below.
    #[test]
    fn the_oversize_message_names_the_number_as_it_stands_at_the_failure() {
        assert_eq!(
            raised(b"FORMAT", &[Some(b"1.5"), Some(b"0"), Some(b"0")]),
            (93, 942, subs(&[b"2", b"0"]))
        );
        assert_eq!(
            raised(b"FORMAT", &[Some(b"99.996"), Some(b"2"), Some(b"2")]),
            (93, 942, subs(&[b"100.0", b"2"]))
        );
        let failure = call_with(
            "9",
            "ENGINEERING",
            b"FORMAT",
            &[Some(b"123456.789"), Some(b"2"), None, None, Some(b"0")],
        )
        .expect_err("two spaces cannot hold three integer digits");
        let Failure::Raised(engineering) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(
            (engineering.number, engineering.sub, engineering.additional),
            (93, 942, subs(&[b"123.456789", b"2"]))
        );
        // The same call without `after` leaves the value alone, which is
        // what makes the first case's `2` a rounding rather than a constant.
        assert_eq!(
            raised(b"FORMAT", &[Some(b"1.5"), Some(b"0")]),
            (93, 942, subs(&[b"1.5", b"0"]))
        );

        // It goes through the same rendering a `SAY` would, so it honours
        // `NUMERIC FORM` as well -- measured, `numeric form engineering ;
        // format(1e10,0,,0)` reports `"10E+9"` where the same call under
        // SCIENTIFIC reports `"1E+10"`. `expp` is `0` in both, which
        // suppresses exponential form in the *result* and not here.
        let narrow = [Some(b"1e10".as_slice()), Some(b"0"), None, Some(b"0")];
        for (form, expected) in [
            ("SCIENTIFIC", b"1E+10".as_slice()),
            ("ENGINEERING", b"10E+9"),
        ] {
            let failure = call_with("9", form, b"FORMAT", &narrow)
                .expect_err("no spaces can hold an integer part");
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!(
                (raised.number, raised.sub, raised.additional),
                (93, 942, subs(&[expected, b"0"])),
                "{form} named the wrong value"
            );
        }
    }

    /// `expp == 0` suppresses exponential form and beats `expt == 0`, which
    /// otherwise forces it.
    #[test]
    fn expp_zero_suppresses_exponential_and_beats_expt_zero() {
        let exp =
            |value: &'static [u8], expp: Option<&'static [u8]>, expt: Option<&'static [u8]>| {
                call(b"FORMAT", &[Some(value), None, None, expp, expt]).expect("this call succeeds")
            };
        assert_eq!(exp(b"12345", Some(b"0"), None), b"12345");
        assert_eq!(exp(b"12345", None, Some(b"0")), b"1.2345E+4");
        assert_eq!(exp(b"12345", Some(b"0"), Some(b"0")), b"12345");
        assert_eq!(exp(b"12345", Some(b"2"), Some(b"0")), b"1.2345E+04");
        assert_eq!(exp(b"12345", Some(b"4"), Some(b"0")), b"1.2345E+0004");
        assert_eq!(exp(b"1e10", None, Some(b"20")), b"10000000000");
        // `expt` is a threshold and never occupies a byte, so a value far
        // past what a width could ever be is simply a trigger nothing
        // reaches -- measured, `format(1,,,,999999999999999999)` is `1`.
        assert_eq!(exp(b"1", None, Some(b"999999999999999999")), b"1");
        // An exponent too wide for an explicit `expp` is 93.941, and the
        // mantissa it names is the reframed one.
        assert_eq!(
            raised(
                b"FORMAT",
                &[Some(b"1e10"), None, None, Some(b"1"), Some(b"0")]
            ),
            (93, 941, subs(&[b"1", b"1"]))
        );
    }

    /// `FORMAT` honours `NUMERIC FORM`, which is the second half of the pair
    /// `DIGITS` alone cannot show.
    #[test]
    fn format_honours_numeric_form() {
        let args = [Some(b"1e10".as_slice()), None, None, None, Some(b"0")];
        assert_eq!(
            call_with("9", "SCIENTIFIC", b"FORMAT", &args).expect("legal"),
            b"1E+10"
        );
        assert_eq!(
            call_with("9", "ENGINEERING", b"FORMAT", &args).expect("legal"),
            b"10E+9"
        );
    }

    /// The three validation layers run in the oracle's own order, which a
    /// program can tell apart because each names a different number.
    ///
    /// Argument 2's *type* beats argument 1's, which beats argument 2's
    /// *range* -- so an implementation that validated its own arguments
    /// front to back would answer 93.943 for the first row and 93.906 for
    /// the second.
    #[test]
    fn the_three_validation_layers_run_in_the_oracles_order() {
        assert_eq!(
            raised(b"TRUNC", &[Some(b"AB.CD"), Some(b"V")]),
            (40, 12, subs(&[b"TRUNC", b"2", b"V"]))
        );
        assert_eq!(
            raised(b"TRUNC", &[Some(b"AB.CD"), Some(b"-1")]),
            (93, 943, subs(&[b"TRUNC", b"AB.CD"]))
        );
        assert_eq!(
            raised(b"TRUNC", &[Some(b"1.5"), Some(b"-1")]),
            (93, 906, subs(&[b"1", b"-1"]))
        );

        assert_eq!(
            raised(b"FORMAT", &[Some(b"1"), Some(b"x")]),
            (40, 12, subs(&[b"FORMAT", b"2", b"x"]))
        );
        assert_eq!(
            raised(b"FORMAT", &[Some(b"a"), Some(b"-1")]),
            (93, 943, subs(&[b"FORMAT", b"a"]))
        );
        assert_eq!(
            raised(b"FORMAT", &[Some(b"1"), Some(b"-1")]),
            (93, 906, subs(&[b"1", b"-1"]))
        );
        // All four optional positions are type-checked before the target,
        // in call order, and each names its own method position when it is
        // the range that is wrong.
        assert_eq!(
            raised(b"FORMAT", &[Some(b"a"), Some(b"x"), Some(b"y")]),
            (40, 12, subs(&[b"FORMAT", b"2", b"x"]))
        );
        assert_eq!(
            raised(b"FORMAT", &[Some(b"1"), Some(b"1"), Some(b"y")]),
            (40, 12, subs(&[b"FORMAT", b"3", b"y"]))
        );
        for (position, method) in [(2usize, b"1"), (3, b"2"), (4, b"3"), (5, b"4")] {
            let mut args: Vec<Option<&[u8]>> = vec![Some(b"1"), None, None, None, None];
            args[position - 1] = Some(b"-1");
            assert_eq!(
                raised(b"FORMAT", &args),
                (93, 906, subs(&[method, b"-1"])),
                "argument {position} named the wrong method position"
            );
        }
    }

    /// `TRUNC` rounds its input to `DIGITS` before truncating, and never
    /// produces exponential form.
    #[test]
    fn trunc_rounds_to_digits_first_and_never_goes_exponential() {
        assert_eq!(answer_at("3", b"TRUNC", &[b"123456", b"2"]), b"123000.00");
        assert_eq!(
            answer_at("9", b"TRUNC", &[b"1e20"]),
            b"100000000000000000000"
        );
        assert_eq!(answer(b"TRUNC", &[b"12.987", b"2"]), b"12.98");
        assert_eq!(answer(b"TRUNC", &[b"1.5"]), b"1");
        assert_eq!(answer(b"TRUNC", &[b"-1.5"]), b"-1");
        assert_eq!(answer(b"TRUNC", &[b"0", b"3"]), b"0.000");
        // A value that truncates away entirely loses its sign.
        assert_eq!(answer(b"TRUNC", &[b"-0.0001234", b"2"]), b"0.00");
    }

    /// `ABS` and `SIGN` round to the precision in force, and the rounding is
    /// not skipped for a value that is already positive.
    #[test]
    fn abs_rounds_whatever_the_sign_and_sign_ignores_the_precision() {
        assert_eq!(answer_at("3", b"ABS", &[b"1.23456"]), b"1.23");
        assert_eq!(answer_at("3", b"ABS", &[b"-1.23456"]), b"1.23");
        assert_eq!(answer(b"ABS", &[b"-4.5"]), b"4.5");
        assert_eq!(answer(b"ABS", &[b"  -4.5  "]), b"4.5");
        assert_eq!(answer(b"SIGN", &[b"-12"]), b"-1");
        assert_eq!(answer(b"SIGN", &[b"0"]), b"0");
        // Every spelling of zero is unsigned.
        assert_eq!(answer(b"SIGN", &[b"-0.0"]), b"0");

        // **The rounding is in the stored value, not only in the rendering,
        // and only a *later* read at a wider precision can see that.**
        // `to_text` re-rounds through the captured pair either way, so a
        // result built without `round_to` renders identically; feeding it
        // back into a builtin that rounds to the precision then in force is
        // what tells the two apart. Measured: `numeric digits 3 ; x =
        // abs(-1.23456) ; numeric digits 9 ; say max(x,-1)` is `1.23`, not
        // `1.23456`.
        let mut interp = interp_with("3", "SCIENTIFIC", "0");
        let argument = interp.text(b"-1.23456");
        let rounded = dispatch(&mut interp, b"ABS", &[Some(argument)])
            .expect("a builtin name")
            .expect("a legal call");
        assert_eq!(&*interp.to_text(rounded), b"1.23");
        interp
            .activation_mut()
            .settings
            .set_digits_str("9")
            .expect("a legal DIGITS");
        // The result object itself, not a fresh value built from its bytes:
        // re-parsing the rendering would round it a second time and the
        // mutation would survive.
        let minus_one = interp.text(b"-1");
        let widened = dispatch(&mut interp, b"MAX", &[Some(rounded), Some(minus_one)])
            .expect("a builtin name")
            .expect("a legal call");
        assert_eq!(&*interp.to_text(widened), b"1.23");
    }

    /// The five builtins that answer a *number* capture the `DIGITS`/`FORM`
    /// pair at the call, and the two that answer *text* have nothing a later
    /// setting could reshape (D15).
    ///
    /// The mutation this catches is rendering a result through the settings
    /// in force at the *read* rather than at the call. Only a program that
    /// changes a setting in between can see it, which is what this does.
    #[test]
    fn a_result_is_rendered_under_the_settings_that_produced_it() {
        // DIGITS moved down after the value was made.
        let mut interp = interp_with("5", "SCIENTIFIC", "0");
        let max =
            call_in(&mut interp, b"MAX", &[Some(b"123456789"), Some(b"1")]).expect("a legal call");
        let abs = call_in(&mut interp, b"ABS", &[Some(b"-1.23456789")]).expect("a legal call");
        let format = call_in(
            &mut interp,
            b"FORMAT",
            &[Some(b"1.23456"), None, Some(b"4")],
        )
        .expect("a legal call");
        let trunc = call_in(&mut interp, b"TRUNC", &[Some(b"1.23456"), Some(b"5")]).expect("legal");
        interp
            .activation_mut()
            .settings
            .set_digits_str("12")
            .expect("a legal DIGITS");
        assert_eq!(max, b"1.2346E+8");
        assert_eq!(abs, b"1.2346");
        assert_eq!(format, b"1.2346");
        assert_eq!(trunc, b"1.23460");
        // Re-running the same calls at the new precision gives different
        // answers, which is what makes the four above evidence of capture
        // rather than of a fixed rendering.
        assert_eq!(
            call_in(&mut interp, b"MAX", &[Some(b"123456789"), Some(b"1")]).expect("legal"),
            b"123456789"
        );

        // FORM moved after the value was made.
        let mut interp = interp_with("9", "ENGINEERING", "0");
        let max = call_in(&mut interp, b"MAX", &[Some(b"1e10"), Some(b"1")]).expect("legal");
        interp
            .activation_mut()
            .settings
            .set_form_str("SCIENTIFIC")
            .expect("a legal FORM");
        assert_eq!(max, b"10E+9");
        assert_eq!(
            call_in(&mut interp, b"MAX", &[Some(b"1e10"), Some(b"1")]).expect("legal"),
            b"1E+10"
        );
    }

    /// `MAX`/`MIN` compare under `NUMERIC FUZZ`, which no other builtin in
    /// this family reads.
    #[test]
    fn max_and_min_compare_under_the_fuzz_in_force() {
        let mut fuzzy = interp_with("9", "SCIENTIFIC", "3");
        assert_eq!(
            call_in(
                &mut fuzzy,
                b"MAX",
                &[Some(b"100000000.0"), Some(b"100000001")]
            )
            .expect("legal"),
            b"100000000"
        );
        let mut exact = interp_with("9", "SCIENTIFIC", "0");
        assert_eq!(
            call_in(
                &mut exact,
                b"MAX",
                &[Some(b"100000000.0"), Some(b"100000001")]
            )
            .expect("legal"),
            b"100000001"
        );
    }

    /// `RANDOM`'s argument validation, including the two errors that are
    /// easy to swap.
    ///
    /// A lone negative argument is the *maximum*, so it is a reversed range
    /// (40.33) and not a bad seed (40.13); the omitted argument's
    /// substitution is the null string.
    #[test]
    fn random_validates_its_range_and_its_seed_separately() {
        assert_eq!(
            raised(b"RANDOM", &[Some(b"-1")]),
            (40, 33, subs(&[b"-1", b""]))
        );
        assert_eq!(
            raised(b"RANDOM", &[Some(b"5"), Some(b"1")]),
            (40, 33, subs(&[b"5", b"1"]))
        );
        assert_eq!(
            raised(b"RANDOM", &[Some(b"1"), Some(b"2"), Some(b"-1")]),
            (40, 13, subs(&[b"RANDOM", b"3", b"-1"]))
        );
        // Every substitution is the *converted* integer, not the argument's
        // own text: measured, `random(1,2,'-1.0')` reports `found "-1"` and
        // `random('5.0','1.0')` reports `("5")` and `("1")`.
        assert_eq!(
            raised(b"RANDOM", &[Some(b"1"), Some(b"2"), Some(b"-1.0")]),
            (40, 13, subs(&[b"RANDOM", b"3", b"-1"]))
        );
        assert_eq!(
            raised(b"RANDOM", &[Some(b"5.0"), Some(b"1.0")]),
            (40, 33, subs(&[b"5", b"1"]))
        );
        assert_eq!(
            raised(b"RANDOM", &[Some(b"0"), Some(b"1000000000")]),
            (40, 32, subs(&[b"0", b"1000000000"]))
        );
        assert_eq!(
            raised(b"RANDOM", &[Some(b"1.5")]),
            (40, 12, subs(&[b"RANDOM", b"1", b"1.5"]))
        );
        // A degenerate range is legal at both ends, and a widest-legal one
        // is too -- the check is `>`, not `>=`.
        assert_eq!(answer(b"RANDOM", &[b"5", b"5"]), b"5");
        assert_eq!(answer(b"RANDOM", &[b"0", b"0"]), b"0");
        assert!(call(b"RANDOM", &[Some(b"0"), Some(b"999999999")]).is_ok());
    }

    /// A seeded stream continues across later *unseeded* calls, and it is
    /// the oracle's own stream number for number.
    ///
    /// **The stream is the requirement, not the seed.** `RANDOM.testGroup`
    /// seeds once and then makes 99 unseeded calls before re-seeding and
    /// requiring the whole run to repeat, so an implementation that re-seeds
    /// per call is deterministic and still wrong. Re-seeding here restarts
    /// the identical three numbers, which is what a per-call reseed could
    /// not produce for calls two and three.
    #[test]
    fn a_seed_starts_a_stream_that_later_calls_continue() {
        let seeded = [b"1".as_slice(), b"999999999", b"12345"];
        let unseeded = [b"1".as_slice(), b"999999999"];
        let mut interp = interp_with("9", "SCIENTIFIC", "0");
        let mut run = || {
            let first = call_in(
                &mut interp,
                b"RANDOM",
                &seeded.iter().map(|b| Some(*b)).collect::<Vec<_>>(),
            )
            .expect("legal");
            let rest: Vec<Vec<u8>> = (0..2)
                .map(|_| {
                    call_in(
                        &mut interp,
                        b"RANDOM",
                        &unseeded.iter().map(|b| Some(*b)).collect::<Vec<_>>(),
                    )
                    .expect("legal")
                })
                .collect();
            (first, rest)
        };
        let (first, rest) = run();
        // Measured on `build/bin/rexx`, and identical on three separate runs
        // of the same program.
        assert_eq!(first, b"776163098");
        assert_eq!(rest, vec![b"950445098".to_vec(), b"552120333".to_vec()]);
        assert_eq!(run(), (first, rest));
    }

    /// `RANDOM`'s answer keeps its own spelling whatever `DIGITS` and `FORM`
    /// say, because the oracle answers a `RexxInteger` and not a value
    /// carrying the D15 pair.
    ///
    /// **The probe varies `DIGITS` *and* uses a value whose rendering can
    /// change, and it needs both.** Three separate instruments were blind to
    /// this for three separate reasons: the differential sweep excludes
    /// `RANDOM` by rule (D11); every other test in this module runs at
    /// `DIGITS 9`, where no answer in range can go exponential; and
    /// `builtin-probes.txt` asked `random(5,5)`, which matches at every
    /// setting. `random(12345,12345)` at `DIGITS 3` is the whole test.
    #[test]
    fn a_random_answer_keeps_its_own_spelling_at_every_precision() {
        let degenerate = [Some(b"12345".as_slice()), Some(b"12345")];
        for digits in ["1", "3", "9", "12"] {
            assert_eq!(
                call_with(digits, "SCIENTIFIC", b"RANDOM", &degenerate).expect("legal"),
                b"12345",
                "DIGITS {digits} reshaped the answer"
            );
        }
        assert_eq!(
            call_with("3", "ENGINEERING", b"RANDOM", &degenerate).expect("legal"),
            b"12345"
        );
        // The adjacent contrast, so this cannot pass on an implementation
        // where *nothing* is ever reshaped: `ABS` answers a real number and
        // the same precision does move it.
        assert_eq!(answer_at("3", b"ABS", &[b"-12345"]), b"1.23E+4");
    }

    /// A padded result the allocator refuses is Error 5 at rc 251 rather
    /// than an abort, for both builtins that pad.
    #[test]
    fn a_padding_width_too_large_to_allocate_raises_error_5() {
        for (name, args) in [
            (
                b"TRUNC".as_slice(),
                vec![Some(b"1".as_slice()), Some(b"123456789012345678")],
            ),
            (
                b"FORMAT",
                vec![Some(b"1".as_slice()), Some(b"123456789012345678")],
            ),
            (
                b"FORMAT",
                vec![Some(b"1".as_slice()), None, Some(b"123456789012345678")],
            ),
        ] {
            let failure = call(name, &args).expect_err("that width cannot be allocated");
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (5, 0));
        }
        // The adjacent success: a width that is merely large is honoured.
        assert_eq!(answer(b"TRUNC", &[b"1", b"1000"]).len(), 1002);
        assert_eq!(answer(b"FORMAT", &[b"1", b"1000"]).len(), 1000);

        // `expp` is the third width, and it is the one that is reserved
        // only when the exponent field is actually written -- the same line
        // the oracle draws. The two calls below differ *only* in `expt`,
        // which is what decides whether there is an exponent to pad, and
        // measured they answer `1` at rc 0 and `System resources exhausted.`
        // at rc 251 respectively.
        //
        // The width is nine quintillion rather than a merely huge one on
        // purpose: a `cargo test` process has no `ulimit -v`, so a 3 GB
        // reservation can genuinely succeed here and then build a 3 GB
        // string. Only a width no allocator anywhere can supply makes this
        // assertion mean the same thing on every machine.
        assert_eq!(
            call(
                b"FORMAT",
                &[Some(b"1"), None, None, Some(b"999999999999999999")]
            )
            .expect("a width that is never written is never allocated"),
            b"1"
        );
        let failure = call(
            b"FORMAT",
            &[
                Some(b"1"),
                None,
                None,
                Some(b"999999999999999999"),
                Some(b"0"),
            ],
        )
        .expect_err("that exponent field cannot be allocated");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (5, 0));

        // And a width Rust's own formatter cannot express is still a
        // result, not a panic: `format!("{:0width$}", ..)` takes a `u16`.
        // Measured, `length(format(1e10,,,65535))` is 65538 and
        // `length(format(1e10,,,65536))` is 65539.
        assert_eq!(
            call(b"FORMAT", &[Some(b"1e10"), None, None, Some(b"65535")])
                .expect("legal")
                .len(),
            65538
        );
        assert_eq!(
            call(b"FORMAT", &[Some(b"1e10"), None, None, Some(b"65536")])
                .expect("legal")
                .len(),
            65539
        );
    }
}
