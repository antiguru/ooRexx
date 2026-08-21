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
//!
//! `LanguageParser::parseMessage` interns the upcased message name into the
//! parse's own string pool -- `messagename = commonString(messagename->
//! upper())` (`parser/LanguageParser.cpp:3391`) -- and `commonString` hands
//! back the pool's copy when the spelling is already there
//! (`parser/LanguageParser.cpp:2269`-`2280`). The pool is a field of the
//! parser (`parser/LanguageParser.hpp:481`), so its scope is one parse, which
//! is [`SelectorTable`]'s scope here.

use std::collections::HashSet;
use std::sync::Arc;

/// One message name, as the parse that read it holds it.
///
/// **Identity, not just equality.** Two occurrences of the same spelling in
/// one parse are one `Selector` sharing one allocation, so
/// [`Selector::same`] answers the question the oracle answers by comparing
/// two `RexxString *`. Two spellings that differ are never the same
/// allocation.
///
/// `Arc` rather than `Rc`, and the requirement is the tree's and not this
/// type's: `crates/rexx-parse/tests/deep.rs` parses on a thread of its own
/// for the stack, which needs `Program` to be `Send`.
///
/// Derefs to the bytes, so a reader that wants the name reads it here rather
/// than through a table: the name is already upcased and already carries the
/// bracket spelling `[]` for the collection form.
///
/// **Comparable across parses by value and not by identity.** A `Selector`
/// from one [`SelectorTable`] and one from another can hold equal bytes at
/// different allocations, so `==` answers for both and [`Selector::same`]
/// answers only within the parse that interned them.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Selector(Arc<[u8]>);

impl Selector {
    /// The name's bytes.
    ///
    /// Spelled as a method as well as through [`Deref`](std::ops::Deref) so
    /// that a struct field or a call argument of type `&[u8]` can be written
    /// without relying on a coercion site.
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
///
/// A set rather than a name-to-index map: the [`Selector`] a caller gets back
/// carries the bytes, so nothing has to index back into the pool, and the
/// pool's only job is to keep the one allocation per spelling alive and
/// findable while the parse runs.
#[derive(Default)]
pub(crate) struct SelectorTable(HashSet<Arc<[u8]>>);

impl SelectorTable {
    pub(crate) fn new() -> SelectorTable {
        SelectorTable::default()
    }

    /// The pool's selector for `name`, adding it if the pool has no such
    /// spelling.
    ///
    /// The parser hands in an already-upcased name, which is where the
    /// oracle upcases it too (`parser/LanguageParser.cpp:3391`): interning
    /// the raw spelling would put `length` and `LENGTH` in the pool as two
    /// entries for one method.
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
    ///
    /// A pool that returned a fresh allocation per call would satisfy `==`
    /// on both rows and fail the first.
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
