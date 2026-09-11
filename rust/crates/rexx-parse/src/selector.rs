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

//! Message selectors, interned while a source is parsed.

use std::collections::HashSet;
use std::sync::Arc;

/// One message name, as the parse that read it holds it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Selector(Arc<[u8]>);

impl Selector {
    /// The name's bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// Whether these two are the same interned name -- one allocation, which
    /// is what interning buys over comparing bytes.
    pub fn same(left: &Selector, right: &Selector) -> bool {
        Arc::ptr_eq(&left.0, &right.0)
    }
}

impl std::ops::Deref for Selector {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

/// One parse's pool of message names.
#[derive(Default)]
pub(crate) struct SelectorTable(HashSet<Arc<[u8]>>);

impl SelectorTable {
    pub(crate) fn new() -> SelectorTable {
        SelectorTable::default()
    }

    /// The pool's selector for `name`, adding it if the pool has no such
    /// spelling.
    pub(crate) fn intern(&mut self, name: &[u8]) -> Selector {
        if let Some(found) = self.0.get(name) {
            return Selector(Arc::clone(found));
        }
        let fresh: Arc<[u8]> = Arc::from(name);
        self.0.insert(Arc::clone(&fresh));
        Selector(fresh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The interning constraint's own test** (D24): one spelling is one
    /// allocation within a parse, so a resolution keyed on a selector's
    /// identity answers the same for every site that names the method.
    #[test]
    fn one_spelling_interns_to_one_allocation_and_two_spellings_do_not() {
        let mut pool = SelectorTable::new();
        let first = pool.intern(b"LENGTH");
        let second = pool.intern(b"LENGTH");
        let other = pool.intern(b"REVERSE");

        assert!(
            Selector::same(&first, &second),
            "the same spelling interned twice is two allocations"
        );
        assert_eq!(first, second);
        assert!(!Selector::same(&first, &other));
        assert_ne!(first, other);
        assert_eq!(first.bytes(), b"LENGTH");
    }

    /// Two pools are two parses, and a selector's identity is a property of
    /// the pool that minted it. Equality still holds across the two, which
    /// is what every byte-keyed reader depends on.
    #[test]
    fn two_pools_agree_by_value_and_not_by_identity() {
        let mut one = SelectorTable::new();
        let mut two = SelectorTable::new();
        let left = one.intern(b"LENGTH");
        let right = two.intern(b"LENGTH");

        assert_eq!(left, right);
        assert!(!Selector::same(&left, &right));
    }
}
