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
use std::rc::Rc;

use rexx_api::load::Library;

/// Every library name that has loaded, with the library it loaded.
///
/// Only a library is held, never a miss: `PackageManager::loadLibrary`
/// (`interpreter/package/PackageManager.cpp:238-244`) removes a package whose
/// load answered false, so the next ask looks again.
///
/// A name's library is written once and afterwards neither removed nor
/// replaced, so a library this hands an [`Rc`] out for stays loaded as long as
/// the interpreter does. That is a correctness constraint:
/// dropping the last reference runs `dlclose`, and the mapping it takes away
/// holds the extension's `UNINIT`, the block a `CSELF` addresses and every
/// entry point resolved out of the package tables, so a library released
/// while an object of a class it contributed a method to is still reachable
/// leaves the finalizer sweep calling an unmapped address.
pub(crate) struct Libraries {
    held: HashMap<Vec<u8>, Rc<Library>>,
    order: PackageTable,
    /// The names held, in the order they loaded.
    loaded: Vec<Vec<u8>>,
}

impl Libraries {
    pub(crate) fn new() -> Libraries {
        Libraries {
            held: HashMap::new(),
            order: PackageTable::new(),
            loaded: Vec::new(),
        }
    }

    /// The library `name` loaded, or `None` for a name nothing is held for.
    pub(crate) fn get(&self, name: &[u8]) -> Option<&Rc<Library>> {
        self.held.get(name)
    }

    /// Holds `library` for `name` where nothing is held yet, and answers what
    /// is held afterwards, which for a name already held is the earlier
    /// library and not `library`.
    pub(crate) fn hold(&mut self, name: &[u8], library: Rc<Library>) -> Rc<Library> {
        let (order, loaded) = (&mut self.order, &mut self.loaded);
        Rc::clone(self.held.entry(name.to_vec()).or_insert_with(|| {
            order.put(name);
            loaded.push(name.to_vec());
            library
        }))
    }

    /// Records an ask for `name` that loaded nothing, which the oracle's
    /// table sees as a put and then a remove.
    pub(crate) fn missed(&mut self, name: &[u8]) {
        if !self.order.contains(name) {
            self.order.put(name);
            self.order.remove(name);
        }
    }

    /// Every held library with its name, in the order
    /// `PackageManager::unload` runs their unloaders
    /// (`interpreter/package/PackageManager.cpp:645-650`).
    pub(crate) fn in_unload_order(&self) -> Vec<(Vec<u8>, Rc<Library>)> {
        self.order
            .names()
            .filter_map(|name| Some((name.to_vec(), Rc::clone(self.held.get(name)?))))
            .collect()
    }
}

/// Closes every library termination left open, in the order they loaded,
/// which is the order the oracle's process exit runs their destructors in:
/// measured with forged extensions, after an unloader raised.
impl Drop for Libraries {
    fn drop(&mut self) {
        for name in &self.loaded {
            if let Some(library) = self.held.get(name) {
                library.close();
            }
        }
    }
}

/// The names in the oracle's `PackageManager::packages` string table, laid
/// out as `HashContents` lays them out, which is what fixes the order the
/// table iterates in: bucket by bucket, each bucket's chain in the order it
/// was built (`interpreter/classes/support/HashContents.cpp:226-300`).
struct PackageTable {
    buckets: usize,
    names: Vec<Option<Vec<u8>>>,
    next: Vec<Option<usize>>,
    /// The head of the free chain, which runs through the overflow half.
    free: Option<usize>,
}

impl PackageTable {
    /// The table `PackageManager::initialize` builds
    /// (`interpreter/package/PackageManager.cpp:81-88`): the default size,
    /// holding the two internal packages.
    fn new() -> PackageTable {
        let mut table = PackageTable::with_buckets(bucket_size(DEFAULT_TABLE_SIZE));
        table.put(b"REXX");
        table.put(b"REXXUTIL");
        table
    }

    fn with_buckets(buckets: usize) -> PackageTable {
        let total = buckets * 2;
        PackageTable {
            buckets,
            names: vec![None; total],
            next: (0..total)
                .map(|slot| (slot >= buckets && slot + 1 < total).then_some(slot + 1))
                .collect(),
            free: Some(buckets),
        }
    }

    fn bucket(&self, name: &[u8]) -> usize {
        let buckets = u64::try_from(self.buckets).expect("a bucket count fits a u64");
        usize::try_from(string_hash(name) % buckets).expect("a bucket fits a usize")
    }

    fn contains(&self, name: &[u8]) -> bool {
        self.names().any(|held| held == name)
    }

    /// `HashCollection::put`: expands first where the free chain is empty
    /// (`HashCollection.cpp:455-460`), then appends to the end of the chain.
    fn put(&mut self, name: &[u8]) {
        if self.free.is_none() {
            // `expandContents(capacity() * 2)`, whose capacity is the total.
            let mut larger = PackageTable::with_buckets(bucket_size(self.buckets * 4));
            for held in self.names() {
                larger.put(held);
            }
            *self = larger;
        }
        let mut at = self.bucket(name);
        if self.names[at].is_none() {
            self.names[at] = Some(name.to_vec());
            self.next[at] = None;
            return;
        }
        loop {
            if self.names[at].as_deref() == Some(name) {
                return;
            }
            match self.next[at] {
                Some(next) => at = next,
                None => break,
            }
        }
        let slot = self.free.expect("an expanded table has a free slot");
        self.free = self.next[slot];
        self.names[slot] = Some(name.to_vec());
        self.next[slot] = None;
        self.next[at] = Some(slot);
    }

    /// `HashContents::remove` and `removeChainLink` (`HashContents.cpp:350-406`).
    fn remove(&mut self, name: &[u8]) {
        let mut at = self.bucket(name);
        let mut previous = None;
        loop {
            match self.names[at].as_deref() {
                None => return,
                Some(held) if held == name => break,
                Some(_) => {}
            }
            previous = Some(at);
            match self.next[at] {
                Some(next) => at = next,
                None => return,
            }
        }
        match previous {
            None => match self.next[at] {
                None => self.names[at] = None,
                Some(next) => {
                    self.names[at] = self.names[next].take();
                    self.next[at] = self.next[next];
                    self.release(next);
                }
            },
            Some(previous) => {
                self.next[previous] = self.next[at];
                self.names[at] = None;
                self.release(at);
            }
        }
    }

    /// `returnToFreeChain`: the slot becomes the head of the free chain.
    fn release(&mut self, slot: usize) {
        self.next[slot] = self.free;
        self.free = Some(slot);
    }

    fn names(&self) -> impl Iterator<Item = &[u8]> {
        (0..self.buckets).flat_map(move |bucket| {
            let mut at = Some(bucket);
            std::iter::from_fn(move || {
                let slot = at?;
                let name = self.names[slot].as_deref()?;
                at = self.next[slot];
                Some(name)
            })
        })
    }
}

/// `HashCollection::DefaultTableSize` (`classes/support/HashCollection.hpp:130`).
const DEFAULT_TABLE_SIZE: usize = 17;

/// `HashCollection::MinimumBucketSize` (`classes/support/HashCollection.hpp:126`).
const MINIMUM_BUCKET_SIZE: usize = 17;

/// `HashCollection::calculateBucketSize` (`classes/support/HashCollection.cpp:150`),
/// below the table sizes this can reach.
fn bucket_size(capacity: usize) -> usize {
    let capacity = capacity.max(MINIMUM_BUCKET_SIZE);
    if capacity.is_multiple_of(2) {
        capacity + 1
    } else {
        capacity
    }
}

/// `RexxString::getStringHash` (`classes/StringClass.hpp:328-346`), over
/// `char`, which is signed on this platform.
fn string_hash(name: &[u8]) -> u64 {
    name.iter().fold(0, |hash: u64, byte| {
        hash.wrapping_mul(31)
            .wrapping_add_signed(i64::from(i8::from_ne_bytes([*byte])))
    })
}
