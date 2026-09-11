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

use rexx_num::Number;
// The precision `::OPTIONS DIGITS` and `::OPTIONS FUZZ` convert under, which is
// `Numerics::ARGUMENT_DIGITS` and belongs to the numeric layer along with
// everything else about numbers here.
pub(crate) use rexx_num::ARGUMENT_DIGITS;

/// `RexxString::numberString()`: the number `text` denotes, or `None` if it
/// denotes none.
fn number(text: &[u8]) -> Option<Number> {
    Number::parse(std::str::from_utf8(text).ok()?)
}

/// Whether `text` is a Rexx number at all, without asking what its value is.
pub(crate) fn is_number(text: &[u8]) -> bool {
    number(text).is_some()
}

/// The value of `text` as a whole number under `digits` precision, or `None` if
/// it has none.
pub(crate) fn whole_number(text: &[u8], digits: usize) -> Option<i64> {
    number(text)?.whole_value(digits)
}

/// Whether `text` is a usable `TRACE` option string.
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
