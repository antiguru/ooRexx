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

//! The string builtins.

use rexx_core::ObjRef;

use crate::Interp;
use crate::error::Failure;

/// `LENGTH(string)`: how many bytes the argument renders as.
///
/// **The result is a plain integer whose rendering does not depend on
/// `NUMERIC DIGITS`, so it is created as text and not through
/// `Interp::number`.** Measured: `numeric digits 1 ; say
/// length('abcdefghij')` prints `10`, where a value carrying `DIGITS 1` as
/// its created pair would render `1E+1`. It is a *value*, not a *number*
/// whose precision was captured -- and D15 is still visible from the other
/// side, measured on the same value: built under `numeric digits 3` and read
/// back under `numeric digits 1`, `say n` is still `16` while `say n + 0` is
/// `2E+1`, because the addition is a new operation creating a new number
/// under the digits then in force. `set_sigl` (`run.rs`) creates a line
/// number the same way and for the same reason.
///
/// Bytes, not characters: measured, `say length('1.50')` is 4 and `say
/// length('')` is 0. `to_text` is what the oracle's own `REQUIRED_STRING`
/// conversion corresponds to, so a number argument is measured by its
/// rendering -- `say length(1.50)` is 4, not 3.
pub(crate) fn length(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let value = args[0].expect("check_arity admitted LENGTH's one required argument");
    // The borrow of `interp` ends with this statement, which is what lets the
    // allocation below happen at all.
    let bytes = interp.to_text(value).len();
    Ok(interp.text(bytes.to_string().as_bytes()))
}
