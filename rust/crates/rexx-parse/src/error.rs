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
    /// The interpreter's message text for this error, rendered from the
    /// generated table.
    pub fn message(&self) -> String {
        // The sub-message row first, because it is the specific one, and the
        // major's row only when the specific one would leave a placeholder
        // visible. Filling the placeholder is not an option here -- see the
        // module's note on scope -- and a user-facing message reading
        // `found "&1"` would be worse than the generic one that is true.
        let specific = row(self.code, self.sub);
        if let Some(text) = specific.filter(|text| !has_placeholder(text)) {
            return text.to_string();
        }
        row(self.code, 0)
            .unwrap_or_else(|| panic!("no interpreter message for error {}", self.code))
            .to_string()
    }

    /// The 1-based physical line this error is reported on.
    pub fn line(&self, source: &ProgramSource) -> usize {
        source.line_of(self.byte)
    }
}

impl std::fmt::Display for ParseError {
    /// `13.1: Invalid character in program.`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}: {}", self.code, self.sub, self.message())
    }
}

impl std::error::Error for ParseError {}

/// The generated table's text for one `(major, sub)` pair, or `None` when the
/// table has no such row.
fn row(code: u16, sub: u16) -> Option<&'static str> {
    rexx_inventory::errors::lookup(code, sub).map(|message| message.text)
}

/// Whether `text` still holds a `&1`-style substitution placeholder.
fn has_placeholder(text: &str) -> bool {
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        rest = &rest[at + 1..];
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests;
