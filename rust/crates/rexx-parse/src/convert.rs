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

//! The text conversions both grammars need, ported from the runtime services
//! the C++ parser reaches into rather than from either parser.
//!
//! `TraceSetting::parseTraceSetting` decides whether an operand is a usable
//! `TRACE` option. `RexxString::numberString` decides whether one is a number at
//! all, and `requestNumber` converts one to an integer under a precision. The
//! instruction grammar needs all three for `TRACE` and the directive grammar
//! needs all three for `::CONSTANT`, `::ANNOTATE` and `::OPTIONS`, so they
//! belong to neither module.
//!
//! # Number ACCEPTANCE is not this crate's rule, and must not be
//!
//! `is_number` and the first step of `whole_number` both go through
//! `rexx_num::Number::parse`. That is the only implementation of "is this text a
//! Rexx number" in the workspace, and it must stay the only one.
//!
//! The C++ layers it the same way. `LanguageParser` and `DirectiveParser` never
//! implement number syntax, they call `RexxString::numberString()`, which is the
//! numeric runtime's. So the dependency edge from this crate to `rexx-num`
//! mirrors the interpreter's own.
//!
//! This is not a preference. An earlier version of this module carried its own
//! acceptance rule and reintroduced a defect `rexx-num` had already found, fixed
//! and pinned: blanks are legal between a sign and its digits, so
//! `trace "+ 9"` is a skip count of 9 and `::options digits "+ 9"` is rc 0, and
//! the private rule rejected both. `rexx-num`'s `signblank` case set, 2,320
//! cases, exists for exactly that defect, and `Number::parse` is verified across
//! 128,368 differential cases. Two ports of one rule drift. One does not.
//!
//! # Nothing about numbers is local any more
//!
//! The CONVERSION went the same way as the acceptance. `whole_number`'s second
//! half used to be a local port of `NumberString::numberValue`, which needed the
//! mantissa and exponent and so needed a second walk over the text to recover
//! them. It is now `Number::whole_value`, beside `Number::parse` and beside the
//! `round_to` it partly duplicates, so this module holds no number syntax and no
//! number arithmetic at all: `whole_number` is a call to each.
//!
//! `Number::format` would NOT have worked as a substitute for `whole_value`, and
//! measurably so: `::options digits "1e18"` is Error 26.5 at nineteen digits,
//! while Rexx's display rule puts an adjusted exponent equal to `DIGITS` in plain
//! notation, so a `format`-based check would accept it. The conversion is its own
//! rule, which is why it is a method rather than a caller's composition.

use rexx_num::Number;
// The precision `::OPTIONS DIGITS` and `::OPTIONS FUZZ` convert under, which is
// `Numerics::ARGUMENT_DIGITS` and belongs to the numeric layer along with
// everything else about numbers here.
pub(crate) use rexx_num::ARGUMENT_DIGITS;

/// `RexxString::numberString()`: the number `text` denotes, or `None` if it
/// denotes none.
///
/// Non-UTF-8 input is not a number, which is right rather than convenient: a
/// symbol cannot hold a non-ASCII byte at all, because
/// `LanguageParser::characterTable` is zero for every byte from 0x80 to 0xFF,
/// and a literal that holds one is not a number either.
fn number(text: &[u8]) -> Option<Number> {
    Number::parse(std::str::from_utf8(text).ok()?)
}

/// Whether `text` is a Rexx number at all, without asking what its value is.
///
/// `RexxString::numberString() != OREF_NULL`, which is the test the `::CONSTANT`
/// and `::ANNOTATE` signed forms make on the sign concatenated with the symbol
/// that followed it. Measured against `build/bin/rexxc`: `::CONSTANT c -.5`,
/// `-1e2` and `-5.` are all rc 0 while `-5x` and `-1e` are Error 19.916, so a
/// constant symbol is not necessarily a number.
///
/// The exponent limits come with the delegation rather than being restated here,
/// and they are observable in two independent places. Measured:
/// `::CONSTANT c -1e999999999` is rc 0 and `-1e1000000000` is 19.916, which is
/// the limit on the exponent AS WRITTEN; `-9e999999999` is rc 0 and
/// `-99e999999999` is 19.916, which is the limit on the ADJUSTED exponent. A
/// rule that checked only one of the two would pass one pair and fail the other.
pub(crate) fn is_number(text: &[u8]) -> bool {
    number(text).is_some()
}

/// The value of `text` as a whole number under `digits` precision, or `None` if
/// it has none.
///
/// `RexxString::requestNumber(result, digits)`: `numberString()` for the
/// acceptance and `NumberString::numberValue` for the conversion, which is
/// exactly the two calls below and nothing else. Both belong to the numeric
/// layer, and the rounding rule in particular is one a caller writing its own
/// will get wrong, as one did. `Number::whole_value` documents it with the
/// `TRACE` measurements that pin each step.
///
/// `digits` is the caller's precision, because the two callers convert under
/// different ones: `TRACE` uses the parse-time `NUMERIC DIGITS` and
/// `::OPTIONS DIGITS` uses `ARGUMENT_DIGITS`. Measured, the boundary really does
/// differ: `::options digits 123456789012345678` is rc 0 at eighteen digits
/// where `trace 1234567890` is already 24.1 at ten.
pub(crate) fn whole_number(text: &[u8], digits: usize) -> Option<i64> {
    number(text)?.whole_value(digits)
}

/// Whether `text` is a usable `TRACE` option string.
///
/// `TraceSetting::parseTraceSetting` (`TraceSetting.cpp:135`) reads any number
/// of leading `?` toggles and then exactly ONE more character, ignoring
/// everything after it, which is why `trace results` works: only the `R`
/// matters. An empty string is the normal setting. Measured: `trace r`,
/// `trace ?r`, `trace ??r`, `trace results` and `trace ''` are all rc 0, while
/// `trace zzz` is Error 24.1.
pub(crate) fn check_trace_setting(text: &[u8]) -> Result<(), ()> {
    for &byte in text {
        if byte == b'?' {
            continue;
        }
        return match byte.to_ascii_uppercase() {
            b'A' | b'C' | b'L' | b'E' | b'F' | b'N' | b'O' | b'R' | b'I' => Ok(()),
            _ => Err(()),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests;
