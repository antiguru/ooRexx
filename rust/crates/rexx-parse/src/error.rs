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

//! What a `ParseError` reports: its message, from the generated table, and the
//! physical line it is reported on.
//! ```text
//! Error 7 running select.rex line 3:  WHEN or OTHERWISE expected.
//! Error 7.1:  SELECT on line 3 requires WHEN.
//! ```

use crate::ProgramSource;
use crate::token::ParseError;

impl ParseError {
    /// The interpreter's secondary message for this error, its `&1`, `&2`, ...
    /// replaced by [`ParseError::inserts`] as bytes.
    ///
    /// # Panics
    ///
    /// If the catalogue has no row for this error's (major, sub) pair.
    pub fn message(&self) -> Vec<u8> {
        let text = rexx_inventory::errors::lookup(self.code, self.sub)
            .unwrap_or_else(|| {
                panic!(
                    "no interpreter message for error {}.{}",
                    self.code, self.sub
                )
            })
            .text;
        substitute(text, self.inserts())
    }

    /// The 1-based physical line this error is reported on.
    pub fn line(&self, source: &ProgramSource) -> usize {
        source.line_of(self.byte)
    }
}

impl std::fmt::Display for ParseError {
    /// `8.2: ELSE has no corresponding THEN clause.`, the message lossily decoded.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}: {}",
            self.code,
            self.sub,
            String::from_utf8_lossy(&self.message())
        )
    }
}

impl std::error::Error for ParseError {}

/// `text` with each `&k` replaced by `values[k - 1]`, and a `&k` with no value
/// left as it is.
fn substitute(text: &str, values: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut bytes = text.bytes().peekable();
    while let Some(byte) = bytes.next() {
        let value = match (byte, bytes.peek()) {
            (b'&', Some(digit @ b'1'..=b'9')) => values.get(usize::from(digit - b'1')),
            _ => None,
        };
        match value {
            Some(value) => {
                bytes.next();
                out.extend_from_slice(value);
            }
            None => out.push(byte),
        }
    }
    out
}

#[cfg(test)]
mod tests;
