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

//! `NUMERIC FORM ENGINEERING`, and the `FORMAT`/`TRUNC` builtins.
//!
//! `Number::format` (in `lib.rs`) renders SCIENTIFIC form and must stay exactly
//! as it is -- it is verified across ~150,000 differential cases. Everything
//! here is new: a form-aware sibling for ENGINEERING, and the two builtins
//! layered on top of it. Measured against ooRexx 5.3.0; see
//! `rust/corpus/num/form_notation.rex` and `format_trunc.rex`.

use crate::{Form, Number};

/// What `FORMAT` can fail with. Both are error 93 (`Incorrect call to
/// method`); the interpreter further distinguishes 93.941/93.942, but this
/// crate follows `ArithError`'s lead and exposes only the number a trapped
/// Rexx program actually sees in `RC`, using the variant itself to carry the
/// finer distinction for Rust callers. Every raise site lives in this file,
/// so each variant carries the substitution *values* directly rather than
/// pre-rendered text -- `message()` renders from the generated table on
/// demand, and `additional()` exposes those same values in the
/// interpreter's own order (what `condition('o')~additional` would return).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FormatError {
    /// `before` is too narrow for the integer part actually produced (which,
    /// in exponential form, means the mantissa's integer digits). Error
    /// 93.942; `additional()` is `[value.format(digits), before]`. `value`
    /// is `self` rounded to `digits` -- *not* the reframed mantissa this
    /// error's own padding works with, and not affected by which `form`/
    /// `expt` the call uses. Confirmed by provoking the exponential-mantissa
    /// case (ENGINEERING, before too narrow for a reframed "123.456789")
    /// and getting back the un-reframed "123456.789" instead, and separately
    /// by lowering DIGITS until rounding changes the value and seeing *that*
    /// show up rather than the original literal text.
    BeforeOversize {
        value: Number,
        digits: u64,
        before: u32,
    },
    /// `expp` is too narrow to hold the exponent's digits. Error 93.941;
    /// `additional()` is `[render(mantissa), width]`, where `render` is a
    /// plain (no `before`/`after`) rendering of `mantissa` -- see the two
    /// call sites (`format_with`'s inline check, and
    /// `post_carry_exponent_error`) for what `mantissa` is reframed to and
    /// why there are two of them.
    ExponentOversize { mantissa: Number, width: u32 },
}

impl FormatError {
    pub fn code(self) -> u16 {
        93
    }

    /// The sub-message identifying this failure's exact table row -- see
    /// `ArithError::sub_code`'s doc comment for why `code()` alone isn't
    /// enough (here it would be, since both variants share major 93, but
    /// the pattern matches the crate's other two error types).
    fn sub(&self) -> u16 {
        match self {
            FormatError::BeforeOversize { .. } => 942,
            FormatError::ExponentOversize { .. } => 941,
        }
    }

    /// The substitution values in the interpreter's own order -- what
    /// `condition('o')~additional` would return for this failure.
    pub fn additional(&self) -> Vec<String> {
        match self {
            FormatError::BeforeOversize {
                value,
                digits,
                before,
            } => {
                vec![value.format(*digits), before.to_string()]
            }
            FormatError::ExponentOversize { mantissa, width } => {
                let rendered = render_integer_padded(mantissa, None, None, mantissa, 0)
                    .expect("`before` is None, so the width check cannot fail");
                vec![rendered, width.to_string()]
            }
        }
    }

    /// The interpreter's exact message text, rendered from the generated
    /// table on demand.
    pub fn message(&self) -> String {
        let subs = self.additional();
        let refs: Vec<&str> = subs.iter().map(String::as_str).collect();
        crate::error_text(93, self.sub(), &refs)
    }
}

impl Number {
    /// Renders the number the way automatic-conversion display and `FORMAT`'s
    /// defaults do, honouring `NUMERIC FORM`. `Number::format` only ever
    /// produces SCIENTIFIC; this is the form-aware entry point the brief
    /// asks for, implemented as the all-defaults case of [`Number::format_with`]
    /// so the two can never drift apart.
    pub fn format_form(&self, digits: u64, form: Form) -> String {
        self.format_with(digits, form, None, None, None, None)
            .expect("no `before`/`expp` supplied, so no width check can fail")
    }

    /// Implements the `FORMAT(number, before, after, expp, expt)` builtin.
    ///
    /// Each optional argument is `None` exactly when the Rexx caller omitted
    /// it; validating that a *supplied* argument is a non-negative whole
    /// number is the interpreter's job (error 93/40 on malformed input), not
    /// this crate's -- by the time it gets here, an argument is either absent
    /// or a `u32`.
    ///
    /// `expp` has a sentinel value with a special meaning that plain
    /// "default" does not cover: `expp == Some(0)` forces plain form no
    /// matter how large the number, skipping the exponential decision
    /// entirely. Found by provoking it: omitting the argument and passing a
    /// literal `0` are observably different, and no documentation states
    /// the difference.
    ///
    /// `expt == Some(0)` is *not* a similar "force exponential" sentinel --
    /// that was an earlier, wrong conclusion drawn from cases that were
    /// already exponential regardless. It plugs into the ordinary trigger
    /// exactly like any other value: exponential once the adjusted exponent
    /// `>= expt`. What makes `expt == 0` look special is a separate rule
    /// that fires whenever the *displayed* exponent -- not the trigger --
    /// comes out to exactly zero, which `expt == 0` makes easy to hit
    /// (`adjusted == 0`) but ENGINEERING's grouping can also produce from a
    /// nonzero `adjusted` (grouping 1 or 2 down to 0). A displayed exponent
    /// of exactly `0` is never written as `E+0`; the exponential path was
    /// still taken (`before`/`after` still apply to the mantissa), it just
    /// has nothing to show after it -- unless `expp` was explicit, in which
    /// case the field it would have taken is reserved as blanks instead of
    /// vanishing outright.
    pub fn format_with(
        &self,
        digits: u64,
        form: Form,
        before: Option<u32>,
        after: Option<u32>,
        expp: Option<u32>,
        expt: Option<u32>,
    ) -> Result<String, FormatError> {
        // Every FORMAT call rounds to the *current* DIGITS first, regardless
        // of what `expt` says -- `expt` only moves the exponential-form
        // trigger, it does not add or remove precision. Confirmed with a
        // 15-significant-digit literal at DIGITS 9: it always renders as if
        // it had been rounded to 9 digits first, however `expt` is set.
        let n1 = self.round_to(digits);
        // `BeforeOversize`'s &1 comes from `(n1, digits)` -- see its doc
        // comment for what exactly that renders and why. Stored as the
        // `Number` and the `digits` it needs, not pre-rendered, since
        // `additional()` does the rendering on demand.

        // `expp == 0` is not "no padding": it suppresses exponential form
        // altogether, so a number that would otherwise trigger it renders in
        // full plain digits instead (verified with 1E100 at expp 0, which
        // prints all 101 digits). Everything else about it -- notably the
        // `before` check -- then operates on that plain rendering.
        let expt = if expp == Some(0) {
            None
        } else {
            // The `digits` default is saturated into i64, not narrowed: a
            // bare u64 past 2^63 must stay a huge trigger threshold, and
            // every adjusted exponent it is compared against is within
            // +/-`MAX_EXPONENT`, so saturation decides identically.
            Some(
                expt.map(|e| e as i64)
                    .unwrap_or_else(|| i64::try_from(digits).unwrap_or(i64::MAX)),
            )
        };

        // The C++ side checks `expp` *twice*. The first check
        // (`NumberStringClass.cpp:2021-2062`) runs against the exponent it
        // derives from `this` alone, before the section that applies
        // `after` and can carry -- it has no notion yet of the value carry
        // will eventually produce. So this first check (and its
        // substitution) needs its own, uncarried trigger/grouping computed
        // the same way `resolve_exponential_state` computes its first
        // guess, not the final, carry-resolved one that drives the
        // successful-render path below. Confirmed by forcing a carry that
        // bumps the exponent from 20 to 21 digits (`9.996E+20` rounded to 0
        // decimals) while `expp` is too narrow for either: the reported
        // mantissa is `"9.996"` (reframed at the pre-carry 20), not
        // `"0.9996"` (reframed at the post-carry 21).
        let initial_eng_exp = expt.and_then(|expt| {
            let a = adjusted(&n1);
            // `saturating_mul`: `expt` may itself be the saturated-i64 form
            // of a huge bare `digits` (see the default above), and doubling
            // that must stay huge rather than wrap.
            let triggered =
                a as i64 >= expt || (a < 0 && (n1.exponent as i64).abs() > expt.saturating_mul(2));
            triggered.then(|| group(form, a))
        });
        if let Some(width) = expp {
            if let Some(exp0) = initial_eng_exp {
                let needed = exp0.unsigned_abs().to_string().len() as u32;
                if needed > width {
                    return Err(FormatError::ExponentOversize {
                        mantissa: reframe(&n1, exp0),
                        width,
                    });
                }
            }
            // The C++ redoes the whole trigger/width check a *second* time,
            // right after the decimal-rounding carry that the first check
            // couldn't have known about (`NumberStringClass.cpp:2126-2193`,
            // a kludge the comment there attributes to [bugs:#1474]). Two
            // shapes need it: a carry that grows an *already* -triggered
            // exponent's digit count (`9.996E+99` rounded to 0 decimals at
            // DIGITS 9, `expp` 2 -- pre-carry exponent 99 fits, post-carry
            // 100 does not), and a carry that triggers exponential form for
            // the first time on a value that started plain (`9999999999.6`
            // at DIGITS 15, `expt` 10, `after` 0, `expp` 1 -- adjusted
            // exponent 9 does not clear the trigger, but rounding away the
            // ".6" carries it to 10, which does). Both confirmed against
            // `build/bin/rexx`; the first was this crate's own regression,
            // caught by review, not by any of the 21,296 cases the four
            // curated FORMAT sets already ran.
            if let Some(err) =
                post_carry_exponent_error(&n1, initial_eng_exp, form, expt, after, width)
            {
                return Err(err);
            }
        }

        let (eng_exp, rounded) = resolve_exponential_state(&n1, form, expt, after);

        match eng_exp {
            None => render_integer_padded(&rounded, before, after, &n1, digits),
            Some(exp) => {
                // The exponent check comes before the `before` check -- an
                // exponent that doesn't fit is reported even when `before`
                // would have been wide enough for the mantissa.
                let mantissa = render_integer_padded(&rounded, before, after, &n1, digits)?;

                if exp == 0 {
                    // A displayed exponent of exactly zero is never written
                    // as `E+0` -- confirmed with `format(3.14159,,,,0)` at
                    // DIGITS 5 (adjusted exponent 0 exactly), which prints
                    // `3.1416`, not `3.1416E+0`, and with ENGINEERING
                    // grouping an adjusted exponent of 1 or 2 down to a
                    // displayed 0 the same way. If `expp` was explicit, the
                    // field it would have taken is reserved as blanks
                    // instead of vanishing outright.
                    return Ok(match expp {
                        Some(width) => format!("{mantissa}{}", " ".repeat(width as usize + 2)),
                        None => mantissa,
                    });
                }
                let e_sign = if exp < 0 { '-' } else { '+' };
                let exp_digits = match expp {
                    Some(width) => {
                        format!("{:0width$}", exp.unsigned_abs(), width = width as usize)
                    }
                    None => exp.unsigned_abs().to_string(),
                };
                Ok(format!("{mantissa}E{e_sign}{exp_digits}"))
            }
        }
    }

    /// Implements `TRUNC(number, places)`. Truncates, does not round, and
    /// -- unlike `FORMAT` -- never produces exponential form: `TRUNC(1E10)`
    /// is the eleven-digit integer, not `1E+10`.
    pub fn trunc(&self, digits: u64, places: u32) -> String {
        let n = self.round_to(digits);
        let truncated = truncate_to_places(&n, places);
        // `before` is always `None` here, so the oversize check can never
        // run and the placeholder `value`/`digits` are never read.
        render_integer_padded(&truncated, None, Some(places), &truncated, 0)
            .expect("`before` is None, so the width check cannot fail")
    }
}

/// Chooses ENGINEERING's exponent for a value whose most significant digit
/// sits at `adjusted`: the largest multiple of 3 not exceeding it. Found by
/// sweeping literal `12eK` across `-30..=30` and comparing engineering to
/// scientific at each `K` -- `div_euclid` is exactly this floor division,
/// including on the negative side (`-25 -> -27`, not `-24`).
fn group(form: Form, adjusted: i32) -> i32 {
    match form {
        Form::Scientific => adjusted,
        Form::Engineering => adjusted.div_euclid(3) * 3,
    }
}

/// Rescales `n1` so its value reads as `mantissa * 10^exp`. Used both to turn
/// a number into the digits that sit before `E`, and -- inside
/// `resolve_exponential_state` -- to redo that rescaling from scratch once
/// rounding has changed which `exp` is the right one, rather than patching a
/// stale mantissa.
fn reframe(n1: &Number, exp: i32) -> Number {
    Number {
        negative: n1.negative,
        digits: n1.digits.clone(),
        exponent: n1.exponent - exp,
    }
}

fn adjusted(n: &Number) -> i32 {
    n.exponent + n.digits.len() as i32 - 1
}

/// Decides whether `n1` renders in exponential form and, if so, at what
/// exponent, then applies `after` to the resulting mantissa (or to `n1`
/// itself in plain form).
///
/// The awkward part: rounding the decimals can carry, and a carry can grow
/// the integer-digit count past what the *original* exponent choice assumed
/// -- `9.996E+20` rounded to 0 decimals is not `10E+20`, it is `1E+21`, and
/// `99.996E+20` the same way is `10E+21`, not `100E+20`. Whether the exponent
/// needs to move depends on whether the growth stays inside the current
/// grouping (ENGINEERING tolerates 1-3 integer digits before it must roll
/// over; SCIENTIFIC tolerates exactly 1, so any growth moves it). The same
/// thing happens to a number that started in *plain* form: rounding
/// `999.9996` to 0 decimals at DIGITS 9 / expt 3 carries to `1000`, whose
/// adjusted exponent (3) now clears the trigger, so it must render as `1E+3`,
/// not print all four digits.
///
/// Patching the already-rounded mantissa in place cannot get this right --
/// its digit string is the rounding of the *wrong* target. So each pass
/// starts over from the untouched `n1`, reframes it at the latest guess for
/// the exponent, and rounds fresh; growth can only ever push the guess up
/// (rounding never shrinks a magnitude), so this converges in a couple of
/// passes for anything a real carry chain can produce. The cap is a
/// defensive bound, not an expected iteration count.
fn resolve_exponential_state(
    n1: &Number,
    form: Form,
    expt: Option<i64>,
    after: Option<u32>,
) -> (Option<i32>, Number) {
    // The very first guess is the only place the low-end (fractional-value)
    // trigger matters: it needs `n1`'s own exponent, which later passes,
    // working from a bare adjusted-exponent integer, no longer have. That is
    // fine -- rounding only ever grows a number, so a value already outside
    // exponential range can only move further from the fractional trigger's
    // territory, never into it.
    let mut eng_exp = expt.and_then(|expt| {
        let a = adjusted(n1);
        // `saturating_mul` for the same reason as `format_with`'s first
        // trigger: `expt` may be a saturated huge `digits` default.
        let triggered =
            a as i64 >= expt || (a < 0 && (n1.exponent as i64).abs() > expt.saturating_mul(2));
        triggered.then(|| group(form, a))
    });

    for _ in 0..8 {
        let framed = match eng_exp {
            Some(exp) => reframe(n1, exp),
            None => n1.clone(),
        };
        let rounded = match after {
            Some(places) => round_to_places(&framed, places),
            None => framed,
        };
        let true_adjusted = eng_exp.unwrap_or(0) + adjusted(&rounded);
        // Once exponential, always exponential (growth cannot shrink a
        // magnitude back below the trigger); a number that started plain can
        // only newly trigger via the upper (adjusted >= expt) arm.
        let new_eng_exp = match expt {
            Some(expt) if eng_exp.is_some() || true_adjusted as i64 >= expt => {
                Some(group(form, true_adjusted))
            }
            _ => None,
        };
        if new_eng_exp == eng_exp {
            return (eng_exp, rounded);
        }
        eng_exp = new_eng_exp;
    }
    let framed = match eng_exp {
        Some(exp) => reframe(n1, exp),
        None => n1.clone(),
    };
    let rounded = match after {
        Some(places) => round_to_places(&framed, places),
        None => framed,
    };
    (eng_exp, rounded)
}

/// Redoes the exponent-width trigger/check after the decimal-place cut
/// `after` makes, the way `NumberStringClass.cpp:2126-2193` does right
/// after its own rounding call (`mathRound`, `NumberStringMath.cpp:315`) --
/// see the call site's doc comment for *why* a second check exists at all.
/// Returns `None` when no cut is even possible (`after` omitted), when the
/// number has no decimals to cut, or when nothing is actually dropped --
/// the interpreter's own decimals section only reaches its redo under the
/// same conditions.
///
/// This cannot reuse `resolve_exponential_state`'s already-carry-aware
/// result for two reasons. First, its trigger check is a genuine *redo*,
/// not a refinement: the interpreter recomputes `adjustedExponent` from
/// scratch after rounding and, if the number was not already exponential,
/// applies the ordinary upper-bound trigger fresh -- which
/// `resolve_exponential_state` also does, so the two agree on *whether* and
/// *at what exponent* the result ends up exponential (confirmed: this
/// crate's existing carry tests, e.g.
/// `after_rounding_carry_can_cross_from_plain_into_exponential`, were
/// unaffected by this fix). Second, and this is what actually needs a
/// separate path, the interpreter's rounding here (`mathRound`) carries
/// by **bumping the exponent and holding the digit count fixed**, not by
/// growing the digit count the way `round_to_places` (`resolve_exponential_
/// state`'s rounder) does -- both land on the same *value*, so the
/// successful-render path is unaffected either way, but they disagree on
/// digit count, and this substitution echoes the mid-computation digit
/// count verbatim. Confirmed against `build/bin/rexx`: `9999999999.6` at
/// DIGITS 15, `after` 0, `expp` 1, `expt` 10 reports the mantissa as
/// `"1.000000000"` (nine trailing zeros, matching `mathRound`/`Number::
/// round_to`'s fixed-digit-count carry), not `resolve_exponential_state`'s
/// trimmed `"1"`.
fn post_carry_exponent_error(
    n1: &Number,
    eng_exp0: Option<i32>,
    form: Form,
    expt: Option<i64>,
    after: Option<u32>,
    width: u32,
) -> Option<FormatError> {
    let after = after?;
    // The state `mathRound` actually rounds: reframed to mantissa scale by
    // the first check's exponent if it triggered, `n1` itself (a no-op
    // reframe) otherwise.
    let pre_round = reframe(n1, eng_exp0.unwrap_or(0));
    if pre_round.exponent >= 0 {
        return None; // no decimal places to cut
    }
    let adjusted_decimals = -i64::from(pre_round.exponent);
    if adjusted_decimals <= i64::from(after) {
        return None; // `after` already covers every decimal place present
    }
    let excess = adjusted_decimals - i64::from(after);
    let len = pre_round.digits.len() as i64;
    if excess >= len {
        // The interpreter's own "rounds away to a single digit or zero"
        // branch (`NumberStringClass.cpp:2100-2118`), which does not redo
        // the trigger/width check at all.
        return None;
    }
    // `Number::round_to`'s carry -- bump the exponent, keep the digit count
    // -- is `mathRound`'s; `round_to_places` (used for the successful
    // render) grows the digit vector instead. See this function's doc
    // comment for why that difference matters here specifically.
    let rounded = pre_round.round_to((len - excess) as u64);
    // Back to true scale: `rounded` is still relative to `eng_exp0`.
    let true_scale = Number {
        negative: rounded.negative,
        digits: rounded.digits,
        exponent: rounded.exponent + eng_exp0.unwrap_or(0),
    };
    let adjusted2 = adjusted(&true_scale);
    let triggered = eng_exp0.is_some()
        || matches!(
            expt,
            // `saturating_mul` for the same reason as the other two
            // triggers: `expt` may be a saturated huge `digits` default.
            Some(expt) if adjusted2 as i64 >= expt
                || (adjusted2 < 0 && (true_scale.exponent as i64).abs() > expt.saturating_mul(2))
        );
    if !triggered {
        return None;
    }
    let exp2 = group(form, adjusted2);
    let needed = exp2.unsigned_abs().to_string().len() as u32;
    if needed <= width {
        return None;
    }
    Some(FormatError::ExponentOversize {
        mantissa: reframe(&true_scale, exp2),
        width,
    })
}

/// Rounds (half up) `n` to exactly `places` digits after the decimal point --
/// the cut FORMAT's `after` and TRUNC both make, just at a fixed decimal
/// position rather than a fixed significant-digit count. `Number::round_to`
/// cannot be reused here: its `digits == 0` is a no-op sentinel (see its own
/// doc comment for why), whereas `places == 0` is an ordinary, meaningful
/// cut for this crate's callers (`FORMAT(x, , 0)`), and must still round a
/// `0.6` up to `1`.
///
/// When `n` already has at most `places` decimal digits, this returns `n`
/// **unchanged** rather than padding it with the missing zeros -- `places`
/// is a legitimate `u32` up to `u32::MAX` (the interpreter accepts
/// `TRUNC(1, 2147483648)` and returns the ~2.1-billion-character result),
/// and `Number::exponent` is `i32`, which cannot encode a decimal-place
/// count anywhere near that. Padding is deferred entirely to
/// `render_integer_padded`, which does that arithmetic in `u64`/`usize`
/// against the original `places` value instead of trying to fold it into a
/// `Number` that would have to represent it as an exponent.
fn round_to_places(n: &Number, places: u32) -> Number {
    // Widened to `i64` up front: `-(places as i32)` alone overflows once
    // `places >= 2^31` (a `u32` value the interpreter accepts without
    // complaint), so even deciding which branch to take must not go
    // through `i32`.
    let target_exponent_wide = -i64::from(places);
    if i64::from(n.exponent) >= target_exponent_wide {
        return n.clone();
    }

    // Only reachable once `target_exponent_wide > n.exponent`. `n.exponent`
    // is always within `+/-MAX_EXPONENT` (comfortably inside `i32`), and
    // `target_exponent_wide <= 0` always (`places` is never negative), so
    // it is squeezed into `(n.exponent, 0]` here -- which always fits `i32`.
    // A `places` large enough to overflow can therefore never reach this
    // branch; it always takes the early return above instead.
    let target_exponent = target_exponent_wide as i32;
    let drop = (target_exponent - n.exponent) as usize;
    let len = n.digits.len();
    if drop > len {
        // The cut falls to the left of every stored digit -- the digit that
        // would decide round-up-or-down is an implicit, unstored zero, so
        // there is nothing to round: `0.04` to 0 places rounds down to zero.
        // But it is a *fixed-decimal* zero, not the canonical one --
        // `Number::zero()`/`assemble` would collapse it to exponent 0 and
        // lose the requested `places`, when the caller needs it preserved
        // (`FORMAT`/`TRUNC` on `0.000012345` at 1 place is `0.0`, not `0`,
        // confirmed against the interpreter). The sign is still dropped: a
        // value that rounds away to nothing has no sign left to show,
        // exactly as a whole-number underflow already does.
        return Number {
            negative: false,
            digits: vec![0],
            exponent: target_exponent,
        };
    }
    let keep = len - drop;
    let mut kept: Vec<u8> = n.digits[..keep].to_vec();
    if n.digits[keep] >= 5 {
        // A full carry chain (all-9s down to `keep`) grows the digit count
        // by one instead of shifting the exponent -- unlike `round_to`,
        // which holds the digit count fixed and lets the exponent absorb the
        // growth. Here the exponent (`places`, hence `target_exponent`) is
        // what the caller fixed, so the growth has to show up as an extra
        // digit: `99.96` to 1 place is `100.0`, not `10.0` at one exponent
        // higher.
        let mut i = keep;
        loop {
            if i == 0 {
                kept.insert(0, 1);
                break;
            }
            i -= 1;
            if kept[i] == 9 {
                kept[i] = 0;
            } else {
                kept[i] += 1;
                break;
            }
        }
    }
    if kept.is_empty() {
        // `keep == 0` and no carry: rounds down to nothing, same underflow
        // as the `drop > len` branch above, and the same reason `assemble`
        // cannot be used -- it would collapse this to the canonical zero
        // and lose `target_exponent` (`0.001` to 2 places is `0.00`, not
        // `0`, confirmed against the interpreter). A carry reaching this
        // point instead (`if i == 0` above) leaves `kept` non-empty, so it
        // takes the normal `assemble` path below.
        return Number {
            negative: false,
            digits: vec![0],
            exponent: target_exponent,
        };
    }
    Number::assemble(n.negative, kept, target_exponent)
}

/// TRUNC's cut: same fixed decimal position as `round_to_places`, but drops
/// the extra digits outright instead of deciding whether to carry. Padding
/// (when `n` already has at most `places` decimal digits) is deferred to
/// `render_integer_padded` the same way and for the same reason -- see
/// `round_to_places`'s doc comment.
fn truncate_to_places(n: &Number, places: u32) -> Number {
    let target_exponent_wide = -i64::from(places);
    if i64::from(n.exponent) >= target_exponent_wide {
        return n.clone();
    }
    let target_exponent = target_exponent_wide as i32;
    let drop = (target_exponent - n.exponent) as usize;
    let len = n.digits.len();
    if drop >= len {
        // Everything gets dropped -- same underflow as `round_to_places`,
        // and the same fix: keep `target_exponent` so the requested
        // decimal-place count still shows (`TRUNC(0.000012345, 1)` is
        // `0.0`, not `0`), constructed directly rather than through
        // `assemble`, which would collapse it back to the canonical zero.
        return Number {
            negative: false,
            digits: vec![0],
            exponent: target_exponent,
        };
    }
    let keep = len - drop;
    // `assemble` here only ever sees a genuinely nonzero leading digit
    // (`keep > 0` and `n` is nonzero, which never stores a leading zero),
    // so it cannot re-trigger the collapse above. At `places == 0` the
    // branch above and this one agree anyway -- `target_exponent` is 0
    // there, which is exactly `Number::zero()` -- so `TRUNC(-0.5, 0)` still
    // drops its sign; it is only `places > 0` where preserving the exponent
    // here matters.
    Number::assemble(n.negative, n.digits[..keep].to_vec(), target_exponent)
}

/// Splits `n` into plain sign/integer/decimal text, extends the decimal part
/// to `after` places if requested, and, if `before` is supplied, pads or
/// rejects the integer part. Used both for genuinely plain numbers and,
/// unmodified, for an exponential mantissa -- reframing already arranges for
/// the mantissa's own digit-count-plus-exponent to equal the number of
/// integer digits it should display, so the same split and the same
/// oversize check apply to both without knowing which one it is.
///
/// `after` is applied here against `n`'s *natural* decimal digits, rather
/// than earlier by folding it into `n.exponent` (which is what
/// `round_to_places`/`truncate_to_places` used to do): both `after`
/// (`FORMAT`) and `places` (`TRUNC`) are legitimate up to `u32::MAX`, and the
/// interpreter really does accept that (`TRUNC(1, 2147483648)` is a
/// ~2.1-billion-character result) -- far more than `Number::exponent`'s
/// `i32` could ever hold. Doing this arithmetic here, in `u64`/`usize`
/// against the original value, instead of against a `Number` that would
/// have had to encode it as an exponent, is what stays correct at that
/// scale. By the time `n` gets here it has already been rounded/truncated
/// down to at most `after` decimal digits if it had more, so `extra` below
/// is only ever adding, never needing to trim.
fn render_integer_padded(
    n: &Number,
    before: Option<u32>,
    after: Option<u32>,
    oversize_value: &Number,
    oversize_digits: u64,
) -> Result<String, FormatError> {
    let sign = if n.negative { "-" } else { "" };
    let d: String = n.digits.iter().map(|x| (b'0' + x) as char).collect();

    let (int_part, natural_dec) = if n.exponent >= 0 {
        (format!("{d}{}", "0".repeat(n.exponent as usize)), None)
    } else {
        let point = n.digits.len() as i32 + n.exponent;
        if point > 0 {
            let point = point as usize;
            (d[..point].to_string(), Some(d[point..].to_string()))
        } else {
            (
                "0".to_string(),
                Some(format!("{}{d}", "0".repeat((-point) as usize))),
            )
        }
    };

    let dec_part = match after {
        None => natural_dec,
        Some(0) => None,
        Some(places) => {
            let natural_len = natural_dec.as_deref().map_or(0u64, |s| s.len() as u64);
            let extra = u64::from(places).saturating_sub(natural_len) as usize;
            let mut s = natural_dec.unwrap_or_default();
            s.push_str(&"0".repeat(extra));
            Some(s)
        }
    };
    let needed = int_part.len() as i64;

    let pad = match before {
        None => 0,
        Some(before) => {
            // Widened to `i64`: `before` is a legitimate `u32` up to
            // `u32::MAX` (the interpreter accepts `FORMAT(1, 3000000000)`
            // and returns the three-billion-character result), and casting
            // it to `i32` first can turn it negative, producing a spurious
            // oversize error instead of the huge space-padding the
            // interpreter actually shows.
            //
            // The error text substitutes the requested `before` itself, not
            // the space actually available after the sign -- confirmed with
            // `format(-123.456, 3)`, which reports "too large for 3 spaces"
            // even though only 2 of those 3 are usable once the sign is set
            // aside.
            let available = i64::from(before) - i64::from(n.negative);
            if available < needed {
                return Err(FormatError::BeforeOversize {
                    value: oversize_value.clone(),
                    digits: oversize_digits,
                    before,
                });
            }
            (available - needed) as usize
        }
    };

    let mut out = String::with_capacity(
        pad + sign.len() + int_part.len() + 1 + dec_part.as_ref().map_or(0, String::len),
    );
    out.push_str(&" ".repeat(pad));
    out.push_str(sign);
    out.push_str(&int_part);
    if let Some(dp) = dec_part {
        out.push('.');
        out.push_str(&dp);
    }
    Ok(out)
}
