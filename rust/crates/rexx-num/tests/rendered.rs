//! `Number::rendered_integer`, the shape predicate that replaces a render.

use rexx_num::{Form, Number};

/// The predicate written the way it used to be: render at this precision,
/// reject anything that is not a plain run of digits, and read the rest back.
/// This is the oracle for every assertion in this file.
fn by_rendering(value: &Number, digits: u64, form: Form) -> Option<i64> {
    let text = value.format_form(digits, form);
    if text.contains('.') || text.contains('E') {
        return None;
    }
    text.parse().ok()
}

/// Mantissas chosen for the shapes the two implementations can disagree
/// about, not for their values: every rounding case (a dropped digit below
/// and above five, an all-nines carry that grows the number, a carry that
/// stops at the first digit), every zero spelling, and both sides of the
/// `i64` and tag boundaries.
const MANTISSAS: &[&str] = &[
    "0",
    "0.0",
    "0.00",
    "0.5",
    "0.05",
    "0.9999999999",
    "1",
    "5",
    "9",
    "10",
    "15",
    "19",
    "99",
    "99.6",
    "99.96",
    "100",
    "123",
    "150",
    "999",
    "999.5",
    "1000",
    "1.50",
    "12.0",
    "12.4",
    "12.5",
    "9.996",
    "1234567890",
    "123456789012345678",
    // The tag's own limits, and one either side of each.
    "2305843009213693951",
    "2305843009213693952",
    "2305843009213693950",
    // `i64::MAX` and `i64::MIN`'s magnitude, and one either side.
    "9223372036854775806",
    "9223372036854775807",
    "9223372036854775808",
    "9999999999999999999",
    "10000000000000000000",
    "12345678901234567890123456",
];

/// Precisions from 1 to 25 as the brief asks, plus `0` -- which is
/// `round_to`'s no-op sentinel and reaches a branch no ordinary `DIGITS`
/// can -- and two settings large enough to exercise the saturating
/// comparison.
fn precisions() -> Vec<u64> {
    let mut out: Vec<u64> = (0..=25).collect();
    out.push(1000);
    out.push(u64::MAX);
    out
}

#[test]
fn the_shape_predicate_answers_what_the_rendering_says() {
    let mut cases = 0usize;
    let mut accepted = 0usize;
    // Accepted values that `plain_integer` declines are the whole reason the
    // predicate is not simply that function: they are the set a narrower
    // replacement would turn into heap objects.
    let mut accepted_beyond_plain_integer = 0usize;

    for mantissa in MANTISSAS {
        for exponent in -24i32..=24 {
            for sign in ["", "-"] {
                let spelling = format!("{sign}{mantissa}E{exponent}");
                let Some(value) = Number::parse(&spelling) else {
                    continue;
                };
                for digits in precisions() {
                    cases += 1;
                    let expected = by_rendering(&value, digits, Form::Scientific);
                    assert_eq!(
                        value.rendered_integer(digits),
                        expected,
                        "{spelling} at DIGITS {digits}"
                    );
                    let Some(whole) = expected else { continue };
                    accepted += 1;
                    if value.plain_integer(digits).is_none() {
                        accepted_beyond_plain_integer += 1;
                    }
                    // What licenses a caller deciding a representation
                    // without knowing which `FORM` is in force: an accepted
                    // value renders as the same digits under both.
                    assert_eq!(
                        by_rendering(&value, digits, Form::Engineering),
                        Some(whole),
                        "{spelling} at DIGITS {digits}, FORM ENGINEERING"
                    );
                }
            }
        }
    }

    // Floors, because every assertion above is inside a loop that a broken
    // generator could leave empty, and the third is the one that matters:
    // a predicate equal to `plain_integer` would satisfy the first two.
    assert!(cases > 20_000, "only {cases} cases were generated");
    assert!(accepted > 1_000, "only {accepted} cases were accepted");
    assert!(
        accepted_beyond_plain_integer > 100,
        "only {accepted_beyond_plain_integer} accepted cases are outside `plain_integer`, \
         so this asserts almost nothing about the part that is new"
    );
}

/// The rounding cases in their own right, with the value spelled out, because
/// the property test above proves agreement without saying what either side
/// answers -- an implementation that rejected everything would fail it, but a
/// reader cannot see from it that `12.4` at `DIGITS 2` is `12`.
#[test]
fn rounding_to_the_precision_happens_before_the_shape_is_read() {
    let n = |text: &str| Number::parse(text).expect("test literal parses");

    // Rounded down, rounded up, and the all-nines carry that grows the value
    // by a digit -- which moves the exponent and so leaves plain form.
    assert_eq!(n("12.4").rendered_integer(2), Some(12));
    assert_eq!(n("12.5").rendered_integer(2), Some(13));
    assert_eq!(n("9.996").rendered_integer(2), Some(10));
    assert_eq!(n("99.6").rendered_integer(2), None); // `1.0E+2`
    // The adjacent successes: the same values at a precision that rounds
    // nothing are not integers at all.
    assert_eq!(n("12.4").rendered_integer(9), None);
    assert_eq!(n("9.996").rendered_integer(9), None);
    // A value already written as an integer, which `plain_integer` also
    // answers, and one that is an integer but renders exponentially.
    assert_eq!(n("150").rendered_integer(9), Some(150));
    assert_eq!(n("1.50E+2").rendered_integer(9), Some(150));
    assert_eq!(n("150").rendered_integer(2), None); // `1.5E+2`
    // Zero, which `parse` collapses to one canonical spelling, so every way
    // of writing it renders `0` and is accepted -- including the negative
    // one, which does not come back as `-0`.
    assert_eq!(n("0").rendered_integer(9), Some(0));
    assert_eq!(n("-0").rendered_integer(9), Some(0));
    assert_eq!(n("0.00").rendered_integer(9), Some(0));
    assert_eq!(n("0E9").rendered_integer(9), Some(0));
    // A fraction that survives the precision keeps its point, which is the
    // adjacent case to the zeros above and the common answer on `arith`.
    assert_eq!(n("0.05").rendered_integer(9), None);
    // Too wide for an `i64`, which the parse this replaces also declined.
    assert_eq!(
        n("9223372036854775807").rendered_integer(19),
        Some(i64::MAX)
    );
    assert_eq!(
        n("-9223372036854775808").rendered_integer(19),
        Some(i64::MIN)
    );
    assert_eq!(n("9223372036854775808").rendered_integer(19), None);
}
