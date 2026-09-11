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

use crate::{Digits, Form, Number};
use std::borrow::Cow;
use std::fmt::Write;

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
    /// 93.942; `additional()` is `[reported.format(digits), before]`.
    /// ```text
    /// format(1.5,0)                            "1.5"          no `after`, nothing rounded
    /// format(1.5,0,0)                          "2"            `after` rounded it, and carried
    /// format(99.996,2,2)                       "100.0"        `mathRound`'s digit count, not four
    /// format(-0.5,1,0)                         "-1"           the sign survives a full round-away
    /// format(123456.789,2)                     "123456.789"   plain: not reframed
    /// ENGINEERING format(123456.789,2,,,0)     "123.456789"   reframed by the engineering exponent
    /// ENGINEERING format(123456.789,2,3,,0)    "123.457"      reframed and then rounded
    /// format(1e10,2,,0)                        "1E+10"        rendered through `stringValue()`
    /// ENGINEERING format(1e10,0,,0)            "10E+9"        ...which honours FORM too
    /// ```
    BeforeOversize {
        reported: Number,
        digits: u64,
        form: Form,
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

    /// The full `(major, sub)` pair, for a caller that has to build the
    /// interpreter's own condition object rather than a message.
    pub fn sub_code(&self) -> (u16, u16) {
        (93, self.sub())
    }

    /// The substitution values in the interpreter's own order -- what
    /// `condition('o')~additional` would return for this failure.
    pub fn additional(&self) -> Vec<String> {
        match self {
            FormatError::BeforeOversize {
                reported,
                digits,
                form,
                before,
            } => {
                vec![reported.format_form(*digits, *form), before.to_string()]
            }
            FormatError::ExponentOversize { mantissa, width } => {
                let rendered =
                    render_integer_padded(mantissa, None, None, None, 0, Form::Scientific)
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
        let expt = trigger(digits, expp, expt);

        // The C++ side checks `expp` *twice*. The first check
        // (`NumberStringClass.cpp:2021-2062`) runs against the exponent it
        // derives from `this` alone, before the section that applies
        // `after` and can carry -- it has no notion yet of the value carry
        // will eventually produce. So this first check (and its
        // substitution) needs the uncarried trigger/grouping --
        // `initial_exponent`, which is also `resolve_exponential_shape`'s
        // first guess -- not the final, carry-resolved one that drives the
        // successful-render path below. Confirmed by forcing a carry that
        // bumps the exponent from 20 to 21 digits (`9.996E+20` rounded to 0
        // decimals) while `expp` is too narrow for either: the reported
        // mantissa is `"9.996"` (reframed at the pre-carry 20), not
        // `"0.9996"` (reframed at the post-carry 21).
        let initial_eng_exp = initial_exponent(Shape::of(&n1), form, expt);
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
        // What `BeforeOversize` substitutes, computed here rather than at the
        // raise site because it needs both exponent choices -- see
        // `reported_value` -- and computed only when `before` is supplied,
        // since nothing else can reach that raise.
        let reported = before.map(|_| reported_value(&n1, initial_eng_exp, eng_exp, after));

        // The exponent check comes before the `before` check -- an exponent
        // that doesn't fit is reported even when `before` would have been
        // wide enough for the mantissa. It has already run above, so the
        // `?` here is the later of the two.
        let layout = pad_to_before(
            plain_layout(Shape::of(&rounded), after),
            before,
            reported.as_ref(),
            digits,
            form,
        )?;
        let field = ExponentField::decide(eng_exp, expp);

        // **Measured out first and written once**, which is what lets this
        // one `String` be allocated at its final size and never grow.
        let mut out = String::with_capacity(layout.len() + field.len());
        emit_plain(&mut out, &rounded, layout);
        field.push_to(&mut out);
        // What makes the layout a promise about the bytes rather than a
        // capacity hint for them -- and what says the allocation above stayed
        // one: the final length equals the capacity asked for, so nothing
        // pushed into it ever had to grow it.
        debug_assert_eq!(
            out.len(),
            layout.len() + field.len(),
            "the rendering disagrees with the layout it was emitted from"
        );
        Ok(out)
    }

    /// The exponent a [`format_with`] call with these same arguments would
    /// display, or `None` when the result is plain and no exponent field is
    /// written at all.
    pub fn format_exponent(
        &self,
        digits: u64,
        form: Form,
        after: Option<u32>,
        expp: Option<u32>,
        expt: Option<u32>,
    ) -> Option<i32> {
        let n1 = self.round_to(digits);
        resolve_exponential_state(&n1, form, trigger(digits, expp, expt), after).0
    }

    /// Implements `TRUNC(number, places)`. Truncates, does not round, and
    /// -- unlike `FORMAT` -- never produces exponential form: `TRUNC(1E10)`
    /// is the eleven-digit integer, not `1E+10`.
    pub fn trunc(&self, digits: u64, places: u32) -> String {
        let n = self.round_to(digits);
        let truncated = truncate_to_places(&n, places);
        // `before` is always `None` here, so the oversize check can never
        // run and the placeholder `value`/`digits` are never read.
        render_integer_padded(&truncated, None, Some(places), None, 0, Form::Scientific)
            .expect("`before` is None, so the width check cannot fail")
    }
}

/// The exponential trigger `format_with` compares against, with `expp`'s
/// suppression folded in. `None` means "never exponential".
fn trigger(digits: u64, expp: Option<u32>, expt: Option<u32>) -> Option<i64> {
    if expp == Some(0) {
        return None;
    }
    Some(
        expt.map(i64::from)
            .unwrap_or_else(|| i64::try_from(digits).unwrap_or(i64::MAX)),
    )
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
/// `resolve_exponential_shape` -- to redo that rescaling from scratch once
/// rounding has changed which `exp` is the right one, rather than patching a
/// stale mantissa.
fn reframe(n1: &Number, exp: i32) -> Number {
    Number {
        negative: n1.negative,
        digits: n1.digits.clone(),
        exponent: Shape::of(n1).reframed(exp).exponent,
    }
}

/// A number reduced to everything the exponential decision and the layout
/// read: how many digits it has, where the point sits relative to them, and
/// whether it has a sign.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Shape {
    len: usize,
    exponent: i32,
    negative: bool,
}

impl Shape {
    fn of(n: &Number) -> Self {
        Shape {
            len: n.digits.len(),
            exponent: n.exponent,
            negative: n.negative,
        }
    }

    /// The exponent of the most significant digit.
    fn adjusted(self) -> i32 {
        self.exponent + self.len as i32 - 1
    }

    /// The shape [`reframe`] produces: rescaling a value to read as
    /// `mantissa * 10^exp` moves the point and leaves the digits and the sign
    /// where they were.
    fn reframed(self, exp: i32) -> Self {
        Shape {
            exponent: self.exponent - exp,
            ..self
        }
    }
}

/// Whether a value of this shape renders exponentially before `after` has cut
/// anything, and at what exponent -- `None` for plain form.
fn initial_exponent(shape: Shape, form: Form, expt: Option<i64>) -> Option<i32> {
    let expt = expt?;
    let a = shape.adjusted();
    // `saturating_mul`: `expt` may itself be the saturated-i64 form of a huge
    // bare `digits` (see `trigger`), and doubling that must stay huge rather
    // than wrap.
    let triggered =
        a as i64 >= expt || (a < 0 && (shape.exponent as i64).abs() > expt.saturating_mul(2));
    triggered.then(|| group(form, a))
}

/// Decides whether `n1` renders in exponential form and, if so, at what
/// exponent, then applies `after` to the resulting mantissa (or to `n1`
/// itself in plain form).
fn resolve_exponential_shape(
    n1: &Number,
    form: Form,
    expt: Option<i64>,
    after: Option<u32>,
) -> (Option<i32>, Shape) {
    let base = Shape::of(n1);
    // The shape a pass works on: `n1` reframed at the current guess, and cut
    // to `after` places if there is a cut to make. Reframing at `0` -- what an
    // untriggered guess gives -- is the identity, which is why the plain arm
    // needs no case of its own.
    let step = |eng_exp: Option<i32>| -> Shape {
        let at = eng_exp.unwrap_or(0);
        match after {
            None => base.reframed(at),
            Some(places) => Shape::of(&round_to_places(&reframe(n1, at), places)),
        }
    };

    // The very first guess is the only place the low-end (fractional-value)
    // trigger matters -- see `initial_exponent`. That is fine: rounding only
    // ever grows a number, so a value already outside exponential range can
    // only move further from the fractional trigger's territory, never into
    // it.
    let mut eng_exp = initial_exponent(base, form, expt);

    for _ in 0..8 {
        let rounded = step(eng_exp);
        let true_adjusted = eng_exp.unwrap_or(0) + rounded.adjusted();
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
    (eng_exp, step(eng_exp))
}

/// The value [`resolve_exponential_shape`] decided on, materialised.
fn resolve_exponential_state<'a>(
    n1: &'a Number,
    form: Form,
    expt: Option<i64>,
    after: Option<u32>,
) -> (Option<i32>, Cow<'a, Number>) {
    let (eng_exp, shape) = resolve_exponential_shape(n1, form, expt, after);
    let framed = match eng_exp {
        Some(exp) => Cow::Owned(reframe(n1, exp)),
        None => Cow::Borrowed(n1),
    };
    let rounded = match after {
        Some(places) => Cow::Owned(round_to_places(&framed, places)),
        None => framed,
    };
    // The one place both halves are in hand at once: the shape the decision
    // settled on is the shape of the value built from it. That decision is
    // made in shape arithmetic, keeping no value it can be checked against --
    // so this is what says the arithmetic tracks what `reframe` and
    // `round_to_places` actually do.
    debug_assert_eq!(
        Shape::of(&rounded),
        shape,
        "the resolved value disagrees with the shape resolved for it"
    );
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
    let rounded = pre_round.into_round((len - excess) as u64);
    // Back to true scale: `rounded` is still relative to `eng_exp0`.
    let true_scale = Number {
        negative: rounded.negative,
        digits: rounded.digits,
        exponent: rounded.exponent + eng_exp0.unwrap_or(0),
    };
    let adjusted2 = Shape::of(&true_scale).adjusted();
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

/// The number `formatInternal` is holding by the time it reports
/// `BeforeOversize`, which is neither the operand nor the rendered result.
fn reported_value(
    n1: &Number,
    eng_exp0: Option<i32>,
    eng_exp: Option<i32>,
    after: Option<u32>,
) -> Number {
    let framed = reframe(n1, eng_exp0.unwrap_or(0));
    let cut = match after {
        None => framed,
        Some(places) => math_round_places(framed, places),
    };
    let true_scale = Number {
        negative: cut.negative,
        digits: cut.digits,
        exponent: cut.exponent + eng_exp0.unwrap_or(0),
    };
    reframe(&true_scale, eng_exp.unwrap_or(0))
}

/// The decimals cut `formatInternal` makes, digit for digit.
fn math_round_places(n: Number, places: u32) -> Number {
    if n.exponent >= 0 {
        return n;
    }
    let adjusted_decimals = -i64::from(n.exponent);
    if adjusted_decimals <= i64::from(places) {
        return n;
    }
    let excess = adjusted_decimals - i64::from(places);
    let len = n.digits.len() as i64;
    if excess < len {
        return n.into_round((len - excess) as u64);
    }
    if excess == len && n.digits[0] >= 5 {
        // `excess < adjusted_decimals` always holds here, and
        // `adjusted_decimals` is within `MAX_EXPONENT`, so `places` fits i32.
        return Number {
            negative: n.negative,
            digits: Digits::single(1),
            exponent: -(places as i32),
        };
    }
    Number::zero()
}

/// Rounds (half up) `n` to exactly `places` digits after the decimal point --
/// the cut FORMAT's `after` and TRUNC both make, just at a fixed decimal
/// position rather than a fixed significant-digit count. `Number::round_to`
/// cannot be reused here: its `digits == 0` is a no-op sentinel (see its own
/// doc comment for why), whereas `places == 0` is an ordinary, meaningful
/// cut for this crate's callers (`FORMAT(x, , 0)`), and must still round a
/// `0.6` up to `1`.
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
            digits: Digits::single(0),
            exponent: target_exponent,
        };
    }
    let keep = len - drop;
    let mut kept = Digits::from_slice(&n.digits[..keep]);
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
                kept.insert_front(1);
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
            digits: Digits::single(0),
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
            digits: Digits::single(0),
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
    Number::assemble(
        n.negative,
        Digits::from_slice(&n.digits[..keep]),
        target_exponent,
    )
}

/// A plain rendering of `n` on its own: [`plain_layout`] and [`pad_to_before`]
/// decide it, [`emit_plain`] writes it. For the callers that want no exponent
/// field at all -- `TRUNC`, and the substitution `ExponentOversize` carries.
fn render_integer_padded(
    n: &Number,
    before: Option<u32>,
    after: Option<u32>,
    reported: Option<&Number>,
    oversize_digits: u64,
    oversize_form: Form,
) -> Result<String, FormatError> {
    let layout = pad_to_before(
        plain_layout(Shape::of(n), after),
        before,
        reported,
        oversize_digits,
        oversize_form,
    )?;
    let mut out = String::with_capacity(layout.len());
    emit_plain(&mut out, n, layout);
    debug_assert_eq!(
        out.len(),
        layout.len(),
        "the rendering disagrees with the layout it was emitted from"
    );
    Ok(out)
}

/// The exact byte counts a plain rendering is about to have, decided before
/// any of those bytes exist.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PlainLayout {
    /// The spaces `before` asks for, ahead of the sign.
    pad: usize,
    /// The sign, written only for a negative value.
    sign: usize,
    /// Digits before the decimal point, including the lone `0` a value below
    /// one shows there.
    int_len: usize,
    /// The decimal part as (the value's own digits, the trailing zeros
    /// `after` adds beyond them). `None` is "no decimal part", which is not
    /// the same as `Some((0, 0))`: the first prints no `.` and the second
    /// would.
    dec: Option<(usize, usize)>,
    /// Where the decimal point falls inside the digit run. Meaningful only
    /// for a negative exponent -- a non-negative one puts the point past
    /// every digit, which is the arm that has no natural decimal part at all.
    point: i32,
}

impl PlainLayout {
    /// The `+ 1` is the point itself, and it is inside the `map_or` rather
    /// than added unconditionally: a value with no decimal part prints no
    /// point, and a count one byte over would be right by luck rather than by
    /// construction.
    fn len(self) -> usize {
        self.pad
            + self.sign
            + self.int_len
            + self.dec.map_or(0, |(natural, extra)| 1 + natural + extra)
    }
}

/// The number of decimal digits `value` is written with, without writing it.
fn decimal_width(value: u32) -> usize {
    value.checked_ilog10().unwrap_or(0) as usize + 1
}

/// Everything a rendering carries after its mantissa, decided once so that
/// the count reserving room for it and the code writing it read the same
/// decision.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ExponentField {
    /// Nothing follows -- plain form, or a displayed exponent of exactly zero
    /// with no explicit `expp` whose field would need reserving.
    Absent,
    /// A displayed exponent of exactly zero *with* an explicit `expp`: the
    /// field it would have taken is reserved as this many blanks rather than
    /// vanishing outright.
    Reserved(usize),
    /// `E`, a sign, `zeros` leading zeros, and then `exp`'s own digits.
    Present { exp: i32, zeros: usize },
}

impl ExponentField {
    fn decide(eng_exp: Option<i32>, expp: Option<u32>) -> Self {
        let Some(exp) = eng_exp else {
            return ExponentField::Absent;
        };
        if exp == 0 {
            // A displayed exponent of exactly zero is never written as `E+0`
            // -- confirmed with `format(3.14159,,,,0)` at DIGITS 5 (adjusted
            // exponent 0 exactly), which prints `3.1416`, not `3.1416E+0`,
            // and with ENGINEERING grouping an adjusted exponent of 1 or 2
            // down to a displayed 0 the same way.
            return match expp {
                Some(width) => ExponentField::Reserved(width as usize + 2),
                None => ExponentField::Absent,
            };
        }
        // **Not `format!("{:0width$}", ..)`.** Rust's format width is a
        // `u16`, so that spelling panics with "Formatting argument out of
        // range" at `expp == 65536`, where the interpreter answers a
        // 65,539-byte result. Measured both sides:
        // `length(format(1e10,,,65535))` is 65538 and
        // `length(format(1e10,,,65536))` is 65539.
        let zeros = match expp {
            Some(width) => (width as usize).saturating_sub(decimal_width(exp.unsigned_abs())),
            None => 0,
        };
        ExponentField::Present { exp, zeros }
    }

    fn len(self) -> usize {
        match self {
            ExponentField::Absent => 0,
            ExponentField::Reserved(blanks) => blanks,
            // The `E` and the sign, then the padding and the digits.
            ExponentField::Present { exp, zeros } => 2 + zeros + decimal_width(exp.unsigned_abs()),
        }
    }

    fn push_to(self, out: &mut String) {
        match self {
            ExponentField::Absent => {}
            ExponentField::Reserved(blanks) => push_repeated(out, ' ', blanks),
            ExponentField::Present { exp, zeros } => {
                out.push('E');
                out.push(if exp < 0 { '-' } else { '+' });
                push_repeated(out, '0', zeros);
                // Written straight into `out` rather than through
                // `to_string`, which would allocate a `String` to be copied
                // into this one; `write!` to a `String` cannot fail.
                let _ = write!(out, "{}", exp.unsigned_abs());
            }
        }
    }
}

/// Decides [`PlainLayout`] for a value of this shape, before `before` has had
/// its say. **Infallible, and that is deliberate**: `before` is the only
/// argument that can raise, so keeping it out of here leaves a rendering that
/// supplies none -- which is what a number's ordinary display is -- with no
/// `Result` carrying a `Number` to construct and drop.
fn plain_layout(shape: Shape, after: Option<u32>) -> PlainLayout {
    let point = shape.len as i32 + shape.exponent;
    let int_len = if shape.exponent >= 0 {
        shape.len + shape.exponent as usize
    } else if point > 0 {
        point as usize
    } else {
        1 // the "0" before the point
    };
    // The digits this value naturally shows after the point, before `after`
    // asks for more. `None` is "no decimal part", which is not the same as
    // `Some(0)`: the first prints no `.` and the second would.
    let natural_dec = if shape.exponent >= 0 {
        None
    } else if point > 0 {
        Some(shape.len - point as usize)
    } else {
        Some((-point) as usize + shape.len)
    };
    // The decimal part as (natural digits, trailing zeros `after` adds).
    let dec = match after {
        None => natural_dec.map(|natural| (natural, 0usize)),
        Some(0) => None,
        Some(places) => {
            let natural = natural_dec.unwrap_or(0);
            let extra = u64::from(places).saturating_sub(natural as u64) as usize;
            Some((natural, extra))
        }
    };
    PlainLayout {
        pad: 0,
        sign: usize::from(shape.negative),
        int_len,
        dec,
        point,
    }
}

/// Applies `before` to a layout: the leading spaces that widen the integer
/// part to it, or the raise for an integer part already wider.
fn pad_to_before(
    layout: PlainLayout,
    before: Option<u32>,
    // `Some` exactly when `before` is, which is the only way the oversize
    // raise below can be reached.
    reported: Option<&Number>,
    oversize_digits: u64,
    // Only ever read to build `BeforeOversize`, which renders `reported`
    // through `stringValue()` and so honours `NUMERIC FORM` as well as
    // `DIGITS`. Every call that passes `before: None` can never reach that
    // raise and passes `Form::Scientific` as an unread placeholder.
    oversize_form: Form,
) -> Result<PlainLayout, FormatError> {
    let Some(before) = before else {
        return Ok(layout);
    };
    let needed = layout.int_len as i64;
    // Widened to `i64`: `before` is a legitimate `u32` up to `u32::MAX` (the
    // interpreter accepts `FORMAT(1, 3000000000)` and returns the
    // three-billion-character result), and casting it to `i32` first can turn
    // it negative, producing a spurious oversize error instead of the huge
    // space-padding the interpreter actually shows.
    let available = i64::from(before) - layout.sign as i64;
    if available < needed {
        return Err(FormatError::BeforeOversize {
            reported: reported
                .expect("`before` is Some here, and the caller pairs the two")
                .clone(),
            digits: oversize_digits,
            form: oversize_form,
            before,
        });
    }
    Ok(PlainLayout {
        pad: (available - needed) as usize,
        ..layout
    })
}

/// Writes the bytes [`plain_layout`] counted, and decides nothing of its own:
/// every branch here is keyed on the exponent and the `point` the layout
/// already carries.
fn emit_plain(out: &mut String, n: &Number, layout: PlainLayout) {
    let (dec, point) = (layout.dec, layout.point);
    push_repeated(out, ' ', layout.pad);
    if n.negative {
        out.push('-');
    }
    if n.exponent >= 0 {
        push_digits(out, n.digits.as_slice());
        push_repeated(out, '0', n.exponent as usize);
    } else if point > 0 {
        push_digits(out, &n.digits.as_slice()[..point as usize]);
    } else {
        out.push('0');
    }
    if let Some((_, extra)) = dec {
        out.push('.');
        // The natural digits, which exist only for a negative exponent. A
        // non-negative one reaches here only because `after` asked for places,
        // and then the whole decimal part is the zeros below.
        if n.exponent < 0 {
            if point > 0 {
                push_digits(out, &n.digits.as_slice()[point as usize..]);
            } else {
                push_repeated(out, '0', (-point) as usize);
                push_digits(out, n.digits.as_slice());
            }
        }
        push_repeated(out, '0', extra);
    }
}

/// Appends `digits` -- one decimal digit per byte, as [`Number`] stores them --
/// to `out` as characters.
fn push_digits(out: &mut String, digits: &[u8]) {
    out.extend(digits.iter().map(|d| char::from(b'0' + d)));
}

/// Appends `count` copies of `ch` to `out`.
fn push_repeated(out: &mut String, ch: char, count: usize) {
    for _ in 0..count {
        out.push(ch);
    }
}
