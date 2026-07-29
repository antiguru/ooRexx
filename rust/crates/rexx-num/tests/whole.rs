//! `Number::whole_value`, the `requestNumber` conversion.
//!
//! Every case is a `build/bin/rexxc` measurement, reached through `TRACE` for the
//! nine-digit precision and through `::OPTIONS DIGITS` and `::OPTIONS FUZZ` for
//! the eighteen-digit one. Those are the two callers, and they are the only way
//! to observe this conversion from a Rexx program, which is why the cases are
//! recorded with the spelling that produced them.
//!
//! The twelve curated differential sets do not cover this function: they exercise
//! the operators and the display conversions, not `requestNumber`. So the cases
//! here are the whole of its differential evidence and are kept exhaustive at the
//! boundaries rather than illustrative.

use rexx_num::{ARGUMENT_DIGITS, Number};

/// The precision a `TRACE` skip count converts under, `number_digits()` at its
/// default.
const TRACE_DIGITS: usize = 9;

fn whole(text: &str, digits: usize) -> Option<i64> {
    Number::parse(text)?.whole_value(digits)
}

#[test]
fn a_whole_number_may_be_written_any_way_rexx_writes_one() {
    // Measured through `::options digits`, all rc 0.
    assert_eq!(whole("12", ARGUMENT_DIGITS), Some(12));
    assert_eq!(whole("1e2", ARGUMENT_DIGITS), Some(100));
    assert_eq!(whole("0009", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole("9.0", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole("9.", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole("+9", ARGUMENT_DIGITS), Some(9));
    // A blank between the sign and the digits is legal, and that is
    // `Number::parse`'s rule rather than this one. Measured: `trace "+ 9"` and
    // `::options digits "+ 9"` are both rc 0.
    assert_eq!(whole("+ 9", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole("  +   9  ", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole("- 9", ARGUMENT_DIGITS), Some(-9));
}

#[test]
fn a_number_that_is_not_whole_has_no_whole_value() {
    // Measured through `::options digits`, all Error 26.5.
    assert_eq!(whole("9.5", ARGUMENT_DIGITS), None);
    assert_eq!(whole("1e-2", ARGUMENT_DIGITS), None);
    assert_eq!(whole(".9", ARGUMENT_DIGITS), None);
    assert_eq!(whole("0.4", ARGUMENT_DIGITS), None);
    assert_eq!(whole("-.5", ARGUMENT_DIGITS), None);
    // Measured through `trace`: 24.1 for both.
    assert_eq!(whole("1.5", TRACE_DIGITS), None);
    assert_eq!(whole("1.4", TRACE_DIGITS), None);
}

/// The conversion ROUNDS to the precision and only then asks whether the result
/// is an integer, so a fraction can survive it. This is the rule a caller
/// reimplementing `requestNumber` gets wrong, so both directions are pinned at
/// each boundary.
#[test]
fn the_conversion_rounds_to_the_precision_before_asking_if_it_is_whole() {
    // rc 0: ten digits truncate to nine, the dropped 4 does not carry.
    assert_eq!(whole("999999999.4", TRACE_DIGITS), Some(999_999_999));
    // rc 0: eleven digits truncate to nine and every surviving decimal is zero.
    assert_eq!(whole("1.0000000001", TRACE_DIGITS), Some(1));
    // rc 0: the dropped digit carries, and a carry over all-nine decimals is 1.
    assert_eq!(whole("0.9999999999", TRACE_DIGITS), Some(1));
    // 24.1: that carry makes the value ten digits wide.
    assert_eq!(whole("999999999.6", TRACE_DIGITS), None);
    // 24.1: nine digits do not exceed the precision, so nothing is rounded and
    // the 6 simply is not whole. This is the control that separates rounding
    // from truncation, and without it a conversion that rounded unconditionally
    // would pass every other row here.
    assert_eq!(whole("99999999.6", TRACE_DIGITS), None);
    // The same shapes under eighteen digits, both measured rc 0 through
    // `::options digits`.
    assert_eq!(
        whole("999999999999999999.4", ARGUMENT_DIGITS),
        Some(999_999_999_999_999_999)
    );
    assert_eq!(whole("1.0000000000000000001", ARGUMENT_DIGITS), Some(1));
    // THE CARRY RULE, IN WORDS, because the values alone do not state it and an
    // earlier version of this comment stated it wrongly.
    //
    // The digits have two separate jobs and it is easy to give them one. The
    // FIRST DROPPED digit -- the tenth, under nine digits -- decides only whether
    // there is a carry, and nothing else: `checkIntegerDigits` sets `carry` from
    // `numberDigits[numDigits] >= 5`. The NINE KEPT digits then decide whether the
    // value is a whole number, and what they must equal depends on that carry:
    // every surviving decimal must be a `0` normally, but a `9` when the carry
    // set, because only an all-nines tail can absorb the +1 and leave zeros.
    //
    // So the dropped digit never appears in the wholeness test, and the kept
    // digits never decide whether there is a carry. The decisive pair is two
    // inputs with IDENTICAL kept digits and different dropped ones, which come out
    // opposite ways. Measured:
    //
    //   trace "0.9999999994"   24.1   dropped 4, no carry, so kept nines must be
    //                                 zeros and are not
    //   trace "0.99999999999"  rc 0   dropped 9, carry, so kept nines must be
    //                                 nines and are
    //
    // and the other direction, a carry that cannot help because a kept digit is
    // not a nine:
    //
    //   trace "0.99999999899"  24.1   dropped 9, carry, ninth kept digit is 8
    //   trace "0.4999999999"   24.1   dropped 9, carry, FIRST kept digit is 4.
    //                                 The carry does happen here. An earlier
    //                                 comment said it did not, which was the
    //                                 wrong rule attached to the right value.
    //
    // And the no-carry branch reaching a whole number at all, so the `compare == 0`
    // arm is covered in both directions too:
    //
    //   trace "1.0000000004"   rc 0   dropped 0, no carry, kept decimals all zero
    assert_eq!(whole("0.99999999999", TRACE_DIGITS), Some(1));
    assert_eq!(whole("0.99999999989", TRACE_DIGITS), Some(1));
    assert_eq!(whole("0.9999999994", TRACE_DIGITS), None);
    assert_eq!(whole("0.99999999899", TRACE_DIGITS), None);
    assert_eq!(whole("0.4999999999", TRACE_DIGITS), None);
    assert_eq!(whole("1.0000000004", TRACE_DIGITS), Some(1));
}

/// The precision is the caller's, and the two callers really do differ.
#[test]
fn the_digit_limit_is_the_precision_the_caller_passed() {
    // Measured: `::options digits 123456789012345678` is rc 0 at eighteen digits
    // and `1234567890123456789` is Error 26.5 at nineteen.
    assert_eq!(
        whole("123456789012345678", ARGUMENT_DIGITS),
        Some(123_456_789_012_345_678)
    );
    assert_eq!(whole("1234567890123456789", ARGUMENT_DIGITS), None);
    assert_eq!(whole("1000000000000000000", ARGUMENT_DIGITS), None);
    // Measured: `trace 123456789` is rc 0 and `trace 1234567890` is Error 24.1.
    assert_eq!(whole("123456789", TRACE_DIGITS), Some(123_456_789));
    assert_eq!(whole("1234567890", TRACE_DIGITS), None);
    // The limit is on the VALUE's width and not the text's. Measured:
    // `::options digits 1E18` is 26.5 at nineteen digits while `1e17` is rc 0 at
    // eighteen, and `trace 1e9` is 24.1 at ten while `trace 1e8` is rc 0.
    assert_eq!(whole("1E18", ARGUMENT_DIGITS), None);
    assert_eq!(
        whole("1e17", ARGUMENT_DIGITS),
        Some(100_000_000_000_000_000)
    );
    assert_eq!(whole("1e9", TRACE_DIGITS), None);
    assert_eq!(whole("1e8", TRACE_DIGITS), Some(100_000_000));
}

#[test]
fn zero_is_zero_however_it_is_spelled() {
    assert_eq!(whole("0", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole("0.000", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole("-0", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole("0e9999", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole("- 0", ARGUMENT_DIGITS), Some(0));
}

#[test]
fn a_negative_whole_number_keeps_its_sign() {
    assert_eq!(whole("-9", TRACE_DIGITS), Some(-9));
    assert_eq!(whole("-1e2", TRACE_DIGITS), Some(-100));
    assert_eq!(whole("-999999999.4", TRACE_DIGITS), Some(-999_999_999));
}

/// The one path that does NOT apply the sign, reproduced from the C++ as written.
///
/// `numberValue` returns `carry ? 1 : 0` with no `* numberSign`, so a negative
/// pure fraction that rounds up converts to +1. This is unobservable from a Rexx
/// program rather than merely unmeasured: the only caller that reaches it is
/// `TRACE`, and a numeric `TRACE` is rejected at RUN time with error 24.901,
/// "Numeric TRACE requests are valid only from interactive debugging", whatever
/// value the parse produced. Pinned so the asymmetry cannot be "tidied" by
/// accident, and flagged as a suspected upstream defect rather than as intended
/// behaviour.
#[test]
fn the_carry_only_path_drops_the_sign_as_the_cpp_does() {
    assert_eq!(whole("0.9999999999", TRACE_DIGITS), Some(1));
    assert_eq!(whole("-0.9999999999", TRACE_DIGITS), Some(1));
    // Its companion, where the carry did not happen, is zero either way, so the
    // sign is invisible there and this row is not evidence of anything but
    // consistency.
    assert_eq!(whole("0.0000000000", TRACE_DIGITS), Some(0));
}

/// A number too wide for any precision is still a number, so acceptance and
/// conversion really are two questions.
#[test]
fn acceptance_and_conversion_are_separate_questions() {
    assert!(Number::parse("1234567890123456789012345678901234567890").is_some());
    assert_eq!(
        whole("1234567890123456789012345678901234567890", ARGUMENT_DIGITS),
        None
    );
    // And text that is not a number at all never reaches the conversion.
    assert!(Number::parse("abc").is_none());
    assert!(Number::parse("-1e").is_none());
    assert!(Number::parse("9 5").is_none());
}
