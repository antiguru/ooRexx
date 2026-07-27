use rexx_num::{Form, FormatError, Number};

fn n(s: &str) -> Number {
    Number::parse(s).unwrap()
}

/// All arguments omitted, at the given `digits`/`form`.
fn form(s: &str, digits: u32, form: Form) -> String {
    n(s).format_form(digits, form)
}

#[allow(clippy::too_many_arguments)]
fn fmt(
    s: &str,
    digits: u32,
    form: Form,
    before: Option<u32>,
    after: Option<u32>,
    expp: Option<u32>,
    expt: Option<u32>,
) -> Result<String, FormatError> {
    n(s).format_with(digits, form, before, after, expp, expt)
}

fn trunc(s: &str, digits: u32, places: u32) -> String {
    n(s).trunc(digits, places)
}

// ---- format_form: ENGINEERING vs SCIENTIFIC -------------------------------

#[test]
fn format_form_scientific_matches_number_format_exactly() {
    // format_form(SCIENTIFIC) is implemented independently of the already-
    // verified `Number::format`, so cross-check them across the interesting
    // magnitudes rather than trust they agree by construction.
    for spelling in [
        "0", "1", "-1", "3.14159", "1e10", "1e-10", "10e-19", "1e-18", "123456789012",
        "0.000000000012345678", "999999999", "1.50", "100.00",
    ] {
        let num = n(spelling);
        for digits in [1, 3, 9] {
            assert_eq!(
                num.format_form(digits, Form::Scientific),
                num.format(digits),
                "digits={digits} spelling={spelling}"
            );
        }
    }
}

#[test]
fn engineering_forces_the_exponent_to_a_multiple_of_three() {
    // rust/corpus/num/form_notation.rex
    assert_eq!(form("12345678901", 9, Form::Engineering), "12.3456789E+9");
    // Adjusted exponent -7, raw exponent -11: neither trigger condition
    // fires at DIGITS 9, so this one stays plain in *both* forms -- ENGINEERING
    // only changes the grouping of numbers that are already exponential.
    assert_eq!(form("0.00000012345", 9, Form::Engineering), "0.00000012345");
    assert_eq!(form("1234000000", 9, Form::Engineering), "1.23400000E+9");
    assert_eq!(form("12345678901", 9, Form::Scientific), "1.23456789E+10");
}

#[test]
fn engineering_pads_a_short_mantissa_with_trailing_zeros_not_a_decimal() {
    // Ground truth from literal `1eK`/`12eK` sweeps -- the padding shows up
    // as extra integer digits, never as a spurious decimal point.
    assert_eq!(form("1e10", 9, Form::Engineering), "10E+9");
    assert_eq!(form("1e11", 9, Form::Engineering), "100E+9");
    assert_eq!(form("1e12", 9, Form::Engineering), "1E+12");
    assert_eq!(form("12e9", 9, Form::Engineering), "12E+9");
    assert_eq!(form("12e10", 9, Form::Engineering), "120E+9");
    assert_eq!(form("12e-30", 9, Form::Engineering), "12E-30");
    assert_eq!(form("12e-29", 9, Form::Engineering), "120E-30");
    assert_eq!(form("12e-28", 9, Form::Engineering), "1.2E-27");
}

#[test]
fn engineering_negative_exponents_group_by_floor_division_not_truncation() {
    // -25 is not a multiple of 3; the engineering exponent is -27 (the
    // largest multiple of 3 that does not exceed -25), not -24.
    assert_eq!(form("12e-25", 9, Form::Engineering), "1.2E-24");
    assert_eq!(form("1e-25", 9, Form::Engineering), "100E-27");
}

#[test]
fn form_does_not_move_the_plain_versus_exponential_boundary() {
    // Only the *grouping* of an already-exponential number changes between
    // forms; the decision of whether to go exponential at all is identical.
    let boundary_low = "1.2345678e-12"; // adjusted -19: still exponential
    let boundary_high = "1.2345678e-11"; // adjusted -18: plain
    assert!(form(boundary_low, 9, Form::Scientific).contains('E'));
    assert!(form(boundary_low, 9, Form::Engineering).contains('E'));
    assert!(!form(boundary_high, 9, Form::Scientific).contains('E'));
    assert!(!form(boundary_high, 9, Form::Engineering).contains('E'));
}

// ---- FORMAT: argument defaults ---------------------------------------------

#[test]
fn one_arg_format_reproduces_the_default_display() {
    assert_eq!(fmt("3.14159", 9, Form::Scientific, None, None, None, None).unwrap(), "3.14159");
    assert_eq!(fmt("-3.14159", 9, Form::Scientific, None, None, None, None).unwrap(), "-3.14159");
    assert_eq!(fmt("1.50", 9, Form::Scientific, None, None, None, None).unwrap(), "1.50");
    assert_eq!(
        fmt("1234567890123", 9, Form::Scientific, None, None, None, None).unwrap(),
        "1.23456789E+12"
    );
}

#[test]
fn before_only_pads_or_rejects_the_integer_part() {
    assert_eq!(fmt("3.14159", 9, Form::Scientific, Some(6), None, None, None).unwrap(), "     3.14159");
    assert_eq!(fmt("-3.14159", 9, Form::Scientific, Some(6), None, None, None).unwrap(), "    -3.14159");
    // Exact fit: no padding at all.
    assert_eq!(fmt("123.456", 9, Form::Scientific, Some(3), None, None, None).unwrap(), "123.456");
    assert_eq!(fmt("-123.456", 9, Form::Scientific, Some(4), None, None, None).unwrap(), "-123.456");
}

#[test]
fn before_zero_always_errors_even_for_a_single_integer_digit() {
    // Zero itself still needs one digit of room; `before == 0` can never
    // succeed.
    assert_eq!(
        fmt("0", 9, Form::Scientific, Some(0), None, None, None),
        Err(FormatError::BeforeOversize)
    );
    assert_eq!(
        fmt("0.5", 9, Form::Scientific, Some(0), None, None, None),
        Err(FormatError::BeforeOversize)
    );
}

#[test]
fn before_too_narrow_for_the_integer_part_is_error_93() {
    assert_eq!(
        fmt("123.456", 9, Form::Scientific, Some(2), None, None, None),
        Err(FormatError::BeforeOversize)
    );
    assert_eq!(FormatError::BeforeOversize.code(), 93);
    // A negative number needs one extra slot for the sign.
    assert_eq!(
        fmt("-123.456", 9, Form::Scientific, Some(3), None, None, None),
        Err(FormatError::BeforeOversize)
    );
    assert!(fmt("-123.456", 9, Form::Scientific, Some(4), None, None, None).is_ok());
}

#[test]
fn after_only_rounds_or_pads_the_decimal_part() {
    assert_eq!(fmt("3.14159", 9, Form::Scientific, None, Some(2), None, None).unwrap(), "3.14");
    assert_eq!(fmt("3.14159", 9, Form::Scientific, None, Some(0), None, None).unwrap(), "3");
    assert_eq!(fmt("3", 9, Form::Scientific, None, Some(2), None, None).unwrap(), "3.00");
    assert_eq!(fmt("3.1", 9, Form::Scientific, None, Some(5), None, None).unwrap(), "3.10000");
    // Half-up rounding, not banker's rounding.
    assert_eq!(fmt("3.145", 9, Form::Scientific, None, Some(2), None, None).unwrap(), "3.15");
    assert_eq!(fmt("3.135", 9, Form::Scientific, None, Some(2), None, None).unwrap(), "3.14");
    assert_eq!(fmt("-3.145", 9, Form::Scientific, None, Some(2), None, None).unwrap(), "-3.15");
}

#[test]
fn after_rounding_underflow_still_shows_the_requested_decimal_places() {
    // Same underflow as TRUNC's: 0.000012345 has no digit within the first
    // decimal place, so `after` must still produce that many zeros rather
    // than collapsing to the bare canonical zero. Sign is dropped either
    // way, matching TRUNC.
    assert_eq!(fmt("0.000012345", 5, Form::Scientific, None, Some(1), None, None).unwrap(), "0.0");
    assert_eq!(fmt("-0.000012345", 5, Form::Scientific, None, Some(1), None, None).unwrap(), "0.0");
    assert_eq!(fmt("0.000012345", 5, Form::Scientific, None, Some(0), None, None).unwrap(), "0");
    // A second, distinct underflow path: the cut lands *exactly* on the
    // last stored digit (no digits left over, but no leading-zero-overrun
    // either -- `drop == len`, not `drop > len`), and that digit rounds
    // down rather than carrying. Caught by an independent differential
    // run: an earlier version only guarded the `drop > len` case and still
    // funnelled this one through `Number::assemble`, which collapses an
    // empty digit vector to the canonical zero and loses `after`.
    assert_eq!(fmt("0.001", 5, Form::Scientific, None, Some(2), None, None).unwrap(), "0.00");
    assert_eq!(fmt("0.000012345", 5, Form::Scientific, None, Some(4), None, None).unwrap(), "0.0000");
}

#[test]
fn after_rounding_carry_can_grow_the_integer_part_before_before_is_checked() {
    // 9.996 rounded to 2 decimals is 10.00 -- the `before` check sees the
    // grown integer part, not the original.
    assert_eq!(fmt("9.996", 9, Form::Scientific, Some(2), Some(2), None, None).unwrap(), "10.00");
    assert_eq!(
        fmt("99.996", 9, Form::Scientific, Some(2), Some(2), None, None),
        Err(FormatError::BeforeOversize)
    );
    assert_eq!(fmt("99.996", 9, Form::Scientific, Some(3), Some(2), None, None).unwrap(), "100.00");
}

#[test]
fn zero_never_goes_exponential_but_still_takes_before_and_after() {
    assert_eq!(fmt("0", 9, Form::Scientific, Some(5), None, None, None).unwrap(), "    0");
    assert_eq!(fmt("0", 9, Form::Scientific, Some(5), Some(2), None, None).unwrap(), "    0.00");
    assert_eq!(fmt("0", 9, Form::Scientific, None, None, None, Some(0)).unwrap(), "0");
}

#[test]
fn zero_still_triggers_the_displayed_exponent_zero_padding_when_expt_is_zero() {
    // Zero's adjusted exponent is always 0, so `expt == 0` still fires the
    // trigger for it (`0 >= 0`) even though nothing ever actually shows --
    // it just always lands on the same displayed-exponent-zero suppression
    // as any other value with adjusted exponent 0. Caught by an independent
    // differential run: an earlier version special-cased zero to skip the
    // exponential machinery entirely, which is right for the *digits*
    // (zero never shows `E...`) but wrong for `expp`'s blank-padding.
    assert_eq!(fmt("0", 9, Form::Scientific, None, None, Some(2), Some(0)).unwrap(), "0    ");
    assert_eq!(fmt("0", 9, Form::Scientific, None, None, Some(4), Some(0)).unwrap(), "0      ");
    assert_eq!(fmt("-0", 9, Form::Scientific, None, None, Some(2), Some(0)).unwrap(), "0    ");
    assert_eq!(
        fmt("0", 1, Form::Scientific, Some(4), Some(2), Some(2), Some(0)).unwrap(),
        "   0.00    "
    );
}

// ---- FORMAT: exponential trigger controlled by expt ------------------------

#[test]
fn expt_moves_the_upper_exponential_trigger() {
    assert_eq!(fmt("123456", 9, Form::Scientific, None, None, None, Some(6)).unwrap(), "123456");
    assert_eq!(fmt("123456", 9, Form::Scientific, None, None, None, Some(5)).unwrap(), "1.23456E+5");
    // Boundary is `>=`: adjusted exponent of 123456 is 5.
    assert_eq!(fmt("99.6", 9, Form::Scientific, None, None, None, Some(1)).unwrap(), "9.96E+1");
}

#[test]
fn expt_moves_the_lower_exponential_trigger_only_for_fractional_values() {
    // 0.001234 has adjusted exponent -3 (fractional) and raw exponent -6.
    assert_eq!(fmt("0.001234", 9, Form::Scientific, None, None, None, Some(3)).unwrap(), "0.001234");
    assert_eq!(fmt("0.001234", 9, Form::Scientific, None, None, None, Some(2)).unwrap(), "1.234E-3");
    // The low-end trigger requires the adjusted exponent to be negative; a
    // value with a nonzero integer part is exempt from it even when it has
    // many more significant digits than `expt` -- 9.996996 has raw exponent
    // -6 (would trip the low-end rule if it applied) but stays plain.
    assert_eq!(fmt("9.996996", 9, Form::Scientific, None, None, None, Some(1)).unwrap(), "9.996996");
}

#[test]
fn expt_zero_plugs_into_the_ordinary_trigger_like_any_other_value() {
    // `expt == 0` is not a "force exponential" sentinel -- that was an
    // earlier, wrong conclusion drawn only from cases whose adjusted
    // exponent was already > 0. It is the ordinary `adjusted >= expt`
    // trigger with `expt` literally 0, which *does* fire whenever the
    // adjusted exponent is non-negative...
    assert_eq!(fmt("123", 9, Form::Scientific, None, None, None, Some(0)).unwrap(), "1.23E+2");
    assert_eq!(fmt("123456", 9, Form::Scientific, None, None, None, Some(0)).unwrap(), "1.23456E+5");
    // ...but when the adjusted exponent is exactly 0, the exponential path
    // is still taken internally, only nothing is left to display for it --
    // see `displayed_exponent_of_zero_is_never_written_as_e_plus_zero`.
    assert_eq!(fmt("3.14159", 5, Form::Scientific, None, None, None, Some(0)).unwrap(), "3.1416");
}

#[test]
fn displayed_exponent_of_zero_is_never_written_as_e_plus_zero() {
    // 3.14159 rounds to 3.1416 at DIGITS 5 (adjusted exponent exactly 0);
    // `expt = 0` triggers the exponential path (adjusted >= expt), but a
    // displayed exponent of exactly 0 is suppressed rather than shown as
    // `E+0`. Without `expp`, it vanishes outright, so this is
    // indistinguishable from plain form.
    assert_eq!(fmt("3.14159", 5, Form::Scientific, None, None, None, Some(0)).unwrap(), "3.1416");
    // With `expp` given, the field the exponent would have taken is
    // reserved as blanks (`expp` + 2 for `E+`/`E-`) instead of vanishing.
    assert_eq!(fmt("3.14159", 5, Form::Scientific, None, None, Some(2), Some(0)).unwrap(), "3.1416    ");
    assert_eq!(fmt("3.14159", 5, Form::Scientific, None, None, Some(4), Some(0)).unwrap(), "3.1416      ");
    // `expp` alone, with `expt` omitted (defaulting to DIGITS 5), never
    // triggers exponential form at all for this value (adjusted 0 < 5), so
    // there is no reserved field.
    assert_eq!(fmt("3.14159", 5, Form::Scientific, None, None, Some(2), None).unwrap(), "3.1416");
}

#[test]
fn engineering_grouping_can_also_produce_a_displayed_exponent_of_zero() {
    // Not just `expt == 0`: ENGINEERING's floor-to-multiple-of-3 grouping
    // can collapse a *nonzero* adjusted exponent (1 or 2) down to a
    // displayed 0 too, and the same suppression applies -- confirmed
    // against the interpreter at DIGITS 5, `expt` 1 (so both 99 and 150
    // trigger the exponential path via `adjusted >= expt`, but their
    // ENGINEERING-grouped exponent is 0 either way).
    assert_eq!(fmt("99", 5, Form::Engineering, None, None, None, Some(1)).unwrap(), "99");
    assert_eq!(fmt("150", 5, Form::Engineering, None, None, None, Some(1)).unwrap(), "150");
    assert_eq!(fmt("99", 5, Form::Engineering, None, None, Some(2), Some(1)).unwrap(), "99    ");
    // The same value under SCIENTIFIC groups to exponent 1, not 0, so the
    // suffix is written normally -- the suppression is specific to the
    // *displayed* exponent being zero, not to `expt` or to ENGINEERING.
    assert_eq!(fmt("99", 5, Form::Scientific, None, None, None, Some(1)).unwrap(), "9.9E+1");
}

// ---- FORMAT: expp controls exponent width, and expp == 0 forces plain -----

#[test]
fn expp_pads_the_exponent_with_leading_zeros() {
    assert_eq!(fmt("1e10", 9, Form::Scientific, None, None, Some(5), None).unwrap(), "1E+00010");
}

#[test]
fn expp_too_narrow_for_the_exponent_is_error_93() {
    assert_eq!(
        fmt("1e10", 9, Form::Scientific, None, None, Some(1), None),
        Err(FormatError::ExponentOversize)
    );
    assert_eq!(FormatError::ExponentOversize.code(), 93);
    assert_eq!(
        fmt("1e100", 9, Form::Scientific, None, None, Some(2), None),
        Err(FormatError::ExponentOversize)
    );
}

#[test]
fn exponent_oversize_is_reported_before_before_oversize() {
    // before=1 would be exactly enough for a one-digit mantissa, so this
    // only fails on the exponent -- confirming the exponent check runs
    // first, not merely that both would fail.
    assert_eq!(
        fmt("1e100", 9, Form::Scientific, Some(1), None, Some(1), None),
        Err(FormatError::ExponentOversize)
    );
}

#[test]
fn expp_zero_overrides_expt_zero() {
    // expp=0 (force plain) and expt=0 (force exponential) directly conflict;
    // expp wins, exactly as the doc comment on `format_with` says it should.
    assert_eq!(fmt("123", 9, Form::Scientific, None, None, Some(0), Some(0)).unwrap(), "123");
    assert_eq!(
        fmt("1e10", 9, Form::Scientific, None, None, Some(0), Some(0)).unwrap(),
        "10000000000"
    );
}

#[test]
fn expp_zero_forces_plain_form_no_matter_how_large() {
    assert_eq!(fmt("1e10", 9, Form::Scientific, None, None, Some(0), None).unwrap(), "10000000000");
    assert_eq!(
        fmt("1e100", 9, Form::Scientific, None, None, Some(0), None).unwrap(),
        "1".to_string() + &"0".repeat(100)
    );
    // The `before` check still applies, against the *plain* integer width.
    assert_eq!(fmt("1e10", 9, Form::Scientific, Some(11), None, Some(0), None).unwrap(), "10000000000");
    assert_eq!(
        fmt("1e10", 9, Form::Scientific, Some(1), None, Some(0), None),
        Err(FormatError::BeforeOversize)
    );
}

// ---- FORMAT: before/after inside exponential (mantissa) form --------------

#[test]
fn before_and_after_apply_to_the_mantissa_in_exponential_form() {
    assert_eq!(fmt("1e10", 9, Form::Scientific, Some(5), None, None, None).unwrap(), "    1E+10");
    assert_eq!(fmt("1e10", 9, Form::Scientific, Some(5), Some(2), None, None).unwrap(), "    1.00E+10");
    assert_eq!(
        fmt("123456789012345", 9, Form::Scientific, Some(20), Some(3), None, None).unwrap(),
        "                   1.235E+14"
    );
}

#[test]
fn before_oversize_in_exponential_form_reports_the_engineering_mantissa() {
    // 1.2E+11 in ENGINEERING is `120E+9` -- a 3-digit mantissa integer part
    // -- so `before` needs to fit 3 digits, not the 1 SCIENTIFIC would need.
    assert_eq!(
        fmt("1.2e11", 9, Form::Engineering, Some(2), None, None, None),
        Err(FormatError::BeforeOversize)
    );
    assert_eq!(fmt("1.2e11", 9, Form::Engineering, Some(3), None, None, None).unwrap(), "120E+9");
}

// ---- FORMAT: carry re-derives the exponential form -------------------------

#[test]
fn after_rounding_carry_can_bump_the_exponent_itself() {
    // 9.996E+20 rounded to 0 decimals is 1E+21, not 10E+20: SCIENTIFIC
    // tolerates only one integer digit, so the carry must move the exponent.
    assert_eq!(fmt("9.996e20", 9, Form::Scientific, None, Some(0), None, None).unwrap(), "1E+21");
    assert_eq!(fmt("9.996e20", 9, Form::Scientific, None, Some(1), None, None).unwrap(), "1.0E+21");
}

#[test]
fn engineering_carry_stays_in_the_same_group_when_it_fits() {
    // 99.996E+20 rounded to 0 decimals carries to a 2-digit mantissa, which
    // still fits ENGINEERING's 1-3 digit budget at the same exponent.
    assert_eq!(fmt("99.996e20", 9, Form::Engineering, None, Some(0), None, None).unwrap(), "10E+21");
    // 999.996E+20 carries through all three digits, still fitting the group.
    assert_eq!(fmt("999.996e20", 9, Form::Engineering, None, Some(0), None, None).unwrap(), "100E+21");
}

#[test]
fn after_rounding_carry_can_cross_from_plain_into_exponential() {
    // At DIGITS 9 / expt 3, 999.9996 (adjusted exponent 2) is plain, but
    // rounding to 0 decimals carries it to 1000 (adjusted exponent 3), which
    // clears the expt-3 trigger -- so the *result* is exponential.
    assert_eq!(fmt("999.9996", 9, Form::Scientific, None, None, None, Some(3)).unwrap(), "999.9996");
    assert_eq!(
        fmt("999.9996", 9, Form::Scientific, None, Some(0), None, Some(3)).unwrap(),
        "1E+3"
    );
    assert_eq!(
        fmt("999.9996", 9, Form::Engineering, None, Some(0), None, Some(3)).unwrap(),
        "1E+3"
    );
}

// ---- TRUNC ------------------------------------------------------------------

#[test]
fn trunc_drops_digits_rather_than_rounding_them() {
    assert_eq!(trunc("3.99", 9, 0), "3");
    assert_eq!(trunc("3.99", 9, 1), "3.9");
    assert_eq!(trunc("-3.99", 9, 0), "-3");
    assert_eq!(trunc("-3.99", 9, 1), "-3.9");
    assert_eq!(trunc("12.345", 9, 2), "12.34");
}

#[test]
fn trunc_pads_with_zeros_when_places_exceeds_the_decimal_count() {
    assert_eq!(trunc("12.345", 9, 5), "12.34500");
    assert_eq!(trunc("100", 9, 2), "100.00");
    assert_eq!(trunc("0.001", 9, 5), "0.00100");
}

#[test]
fn trunc_underflow_still_shows_the_requested_decimal_places() {
    // 0.000012345 has no digit within the first 1 or 2 decimal places, so
    // every stored digit gets dropped -- confirmed against the interpreter
    // that this is a fixed-decimal `0.0`/`0.00`, not the canonical `0` that
    // `Number::zero()` would collapse it to if the exponent weren't
    // preserved separately from the (correctly sign-dropped) magnitude.
    assert_eq!(trunc("0.000012345", 5, 1), "0.0");
    assert_eq!(trunc("0.000012345", 5, 2), "0.00");
    assert_eq!(trunc("-0.000012345", 5, 1), "0.0");
    // `places == 0` lands on the canonical zero either way, so it isn't a
    // regression check by itself, but pins the boundary next to the above.
    assert_eq!(trunc("0.000012345", 5, 0), "0");
}

#[test]
fn trunc_never_produces_exponential_form() {
    assert_eq!(trunc("1e10", 9, 0), "10000000000");
    assert_eq!(trunc("1e-10", 9, 0), "0");
    assert_eq!(trunc("1e-10", 9, 20), "0.00000000010000000000");
}

#[test]
fn trunc_to_a_magnitude_that_vanishes_drops_the_sign() {
    assert_eq!(trunc("-0.5", 9, 0), "0");
    assert_eq!(trunc("0.5", 9, 0), "0");
    assert_eq!(trunc("-0.001", 9, 0), "0");
}

#[test]
fn trunc_rounds_to_current_digits_before_truncating_decimals() {
    // 15 significant digits at DIGITS 9 rounds to 9 first, *then* truncates
    // to the requested decimal places.
    assert_eq!(trunc("123456789012345", 9, 2), "123456789000000.00");
}

#[test]
fn trunc_defaults_places_to_zero() {
    assert_eq!(n("3.99").trunc(9, 0), trunc("3.99", 9, 0));
}

// ---- places/before beyond i32 range -----------------------------------
//
// `places` (TRUNC) and `before`/`after` (FORMAT) are `u32`, and the
// interpreter genuinely accepts values past `i32::MAX` -- it just returns a
// correspondingly huge string. These three each materialise a
// multi-gigabyte `String` (the whole point is proving the arithmetic
// doesn't panic/wrap at that scale), so they check only `.len()`, matching
// `length(...)` on the interpreter side rather than the differential
// harness, which the reviewer asked not to carry these magnitudes into.

#[test]
fn trunc_accepts_places_at_and_past_the_i32_negation_boundary() {
    // `-(places as i32)` overflows in debug ("attempt to negate with
    // overflow") and produces a `capacity overflow` panic in release once
    // `places >= 2^31` (2_147_483_648) -- confirmed against the interpreter
    // that this is a real, accepted input: `length(trunc(1, 2147483648))`
    // is `2147483650` (the digit, the point, and 2_147_483_648 zeros).
    assert_eq!(n("1").trunc(9, 2_147_483_648).len(), 2_147_483_650);
}

#[test]
fn format_before_survives_the_full_u32_range() {
    // `before as i32` wraps negative once `before >= 2^31`, which made
    // `available < needed` spuriously true and raised `BeforeOversize` for
    // an input the interpreter accepts outright: `length(format(1,
    // 3000000000))` is `3000000000` -- the digit plus 2_999_999_999 leading
    // spaces.
    let result = n("1")
        .format_with(9, Form::Scientific, Some(3_000_000_000), None, None, None)
        .unwrap();
    assert_eq!(result.len(), 3_000_000_000);
}

#[test]
fn format_after_survives_the_full_u32_range() {
    // `after` shares `round_to_places` with TRUNC's `places`, so it needed
    // the same fix: `length(format(1,,2147483648))` on the interpreter is
    // `2147483650`, the same shape as the TRUNC case above.
    let result = n("1")
        .format_with(9, Form::Scientific, None, Some(2_147_483_648), None, None)
        .unwrap();
    assert_eq!(result.len(), 2_147_483_650);
}

#[test]
fn expp_already_used_no_i32_cast_to_begin_with() {
    // Unlike `places`/`before`/`after`, `expp` never goes through `i32` --
    // every place it's used is `as usize`/`as u32` (confirmed by inspection
    // of `format.rs`, not just by this test). This is a cheap confirmation
    // rather than a repro: a moderately large `expp` still correctly pads
    // the exponent, with nothing here needing a multi-gigabyte allocation
    // to prove it (an `expp` anywhere near `u32::MAX` would itself demand
    // that many exponent digits, which is a real but uninteresting
    // consequence of the field width being that wide, not a bug).
    let result = fmt("1e10", 9, Form::Scientific, None, None, Some(20), None).unwrap();
    assert_eq!(result, "1E+00000000000000000010");
}
