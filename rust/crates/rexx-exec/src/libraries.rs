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

//! What each library name has resolved to.
//!
//! A module of its own so that the map is private to the write rule below and
//! not to whichever module happens to resolve libraries.

use std::collections::HashMap;

use crate::LibraryLoad;

/// Every library name resolved so far, with the answer it resolved to.
///
/// A name's answer is written once and afterwards neither removed nor
/// replaced, so a library this hands an [`std::rc::Rc`] out for stays loaded
/// as long as the interpreter does. That is a correctness constraint:
/// dropping the last reference runs `dlclose`, and the mapping it takes away
/// holds the extension's `UNINIT`, the block a `CSELF` addresses and every
/// entry point resolved out of the package tables, so a library released
/// while an object of a class it contributed a method to is still reachable
/// leaves the finalizer sweep calling an unmapped address.
pub(crate) struct Libraries {
    held: HashMap<Vec<u8>, LibraryLoad>,
}

impl Libraries {
    pub(crate) fn new() -> Libraries {
        Libraries {
            held: HashMap::new(),
        }
    }

    /// What `name` has resolved to, or `None` for a name nothing has been
    /// held for.
    pub(crate) fn get(&self, name: &[u8]) -> Option<&LibraryLoad> {
        self.held.get(name)
    }

    /// Holds `load` for `name` where nothing is held yet, and answers what is
    /// held afterwards, which for a name already resolved is the earlier
    /// answer and not `load`.
    pub(crate) fn hold(&mut self, name: &[u8], load: LibraryLoad) -> LibraryLoad {
        self.held.entry(name.to_vec()).or_insert(load).clone()
    }
}
