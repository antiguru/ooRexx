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

//! Symbol interning: `SymbolId`, and the `SymbolTable` that hands them out.

use std::borrow::Cow;
use std::collections::HashMap;

/// A symbol's identity: the upcased spelling, interned. Two symbols with the
/// same `SymbolId` name the same variable, method or label.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SymbolId(u32);

impl SymbolId {
    /// This id as an index into the `SymbolTable` that interned it: **dense,
    /// zero-based, and assigned in interning order**, so `n` distinct symbols
    /// occupy exactly `0..n` and `SymbolTable::len` is the length a caller
    /// needs to size a `Vec`.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Interns upcased symbol spellings. Owned by `ProgramSource`'s parse, handed
/// to `Program` so Phase 4 can resolve a `SymbolId` back to text for error
/// messages and `SIGNAL`'s label lookup.
#[derive(Default, Debug)]
pub struct SymbolTable {
    by_name: HashMap<Box<str>, SymbolId>,
    names: Vec<Box<str>>,
}

impl SymbolTable {
    /// Intern `text`, upcasing it. Returns the same id for every spelling that
    /// differs only in case.
    pub fn intern(&mut self, text: &str) -> SymbolId {
        // Cow, not Box<str>, because `Box<str>: From<&str>` copies: building
        // the key eagerly would allocate on the lookup path even when the
        // symbol is already interned, which is the common case by an order of
        // magnitude. Borrow when the text is already upper, allocate only to
        // upcase, and allocate the owned key only on a genuine miss.
        let key: Cow<'_, str> = if text.bytes().any(|b| b.is_ascii_lowercase()) {
            Cow::Owned(text.to_ascii_uppercase())
        } else {
            Cow::Borrowed(text)
        };
        if let Some(&id) = self.by_name.get(key.as_ref()) {
            return id;
        }
        let id = SymbolId(u32::try_from(self.names.len()).expect("symbols fit u32"));
        let owned: Box<str> = key.into_owned().into();
        self.names.push(owned.clone());
        self.by_name.insert(owned, id);
        id
    }

    /// The upcased spelling. Panics on an id from a different table, which is
    /// a parser bug rather than a source error.
    pub fn name(&self, id: SymbolId) -> &str {
        &self.names[id.0 as usize]
    }

    /// How many distinct symbols are interned.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether nothing is interned yet.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}
