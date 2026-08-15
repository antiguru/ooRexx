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

//! `MethodDict`: a flat name-to-method map plus scope ordering.
//!
//! This is the oracle's `MethodDictionary` (`interpreter/behaviour/MethodDictionary.cpp`).
//! One `MethodDict` is used two ways in this crate: as a class's own,
//! unflattened dictionary (the oracle's `instanceMethodDictionary` /
//! `classMethodDictionary`, holding only what that one class defines
//! directly, always at its own scope) and as a behaviour's flattened
//! dictionary (the oracle's `RexxBehaviour::methodDictionary`, built by
//! merging a chain of the former). `ClassGraph` (`class_graph.rs`) is what
//! tells the two apart and drives the merge between them.
//!
//! Method removal (`~delete`, `hideMethod`'s `.nil` tombstone) is not
//! modelled: no probe this task specifies needs it, and every method
//! present here is a real, invokable entry.

use rexx_core::ObjRef;
use std::collections::{BTreeSet, HashMap};

/// Identifies a method body. The bodies themselves are a later task's
/// concern (dispatch, Task 3); this crate only tracks which name resolves
/// to which identity, and through which scope.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct MethodId(pub u32);

/// One name's entry for one scope. Several of these can share a name when
/// more than one class in the ancestor chain defines it -- that is exactly
/// the case a scope-override (`SUPER`) send has to distinguish.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct MethodEntry {
    scope: ObjRef,
    method: MethodId,
}

/// A flat name-to-method(s) map, plus the scope ordering a scope-override
/// send needs to resolve correctly.
///
/// `entries[name]` keeps every scope that defines `name`, front to back in
/// **priority order**: index 0 is what an ordinary (unscoped) lookup
/// returns, matching the oracle's `MethodDictionary::addMethod`, which
/// always prepends a genuinely new (name, scope) pair
/// (`HashContents::addFront`) and replaces in place when the scope repeats.
///
/// `scope_list` is the oracle's `scopeList`: every scope folded into this
/// dictionary, in the order it was folded in. `scope_orders` is the
/// oracle's `scopeOrders`: for each scope, a **snapshot of `scope_list`
/// taken just before that scope was added** (`MethodDictionary::addScope`).
/// That snapshot is what a scope-override send searches -- `findSuperMethod`
/// only considers a method whose scope is the starting scope itself or
/// appears in that scope's own snapshot -- and it is also where
/// `resolveSuperScope`'s "immediate superscope" comes from: the snapshot's
/// last entry.
#[derive(Clone, Default)]
pub struct MethodDict {
    entries: HashMap<String, Vec<MethodEntry>>,
    scope_list: Vec<ObjRef>,
    scope_orders: HashMap<ObjRef, Vec<ObjRef>>,
}

impl MethodDict {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discard everything -- the oracle's `clearMethodDictionary`, the step
    /// every cascade takes before rebuilding a behaviour from scratch.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// True once `scope` has been folded into this dictionary. This is the
    /// cascade's own de-duplication guard: `createInstanceBehaviour` and
    /// `createClassBehaviour` both skip a superclass already merged, which
    /// is what makes a diamond ancestor safe to reach by more than one path.
    pub fn has_scope(&self, scope: ObjRef) -> bool {
        self.scope_orders.contains_key(&scope)
    }

    /// Fold a new scope into the ordering, unconditionally taking a
    /// snapshot of everything folded in so far -- the oracle's
    /// `MethodDictionary::addScope`. A repeat is a no-op, matching the
    /// oracle's own guard.
    pub fn add_scope(&mut self, scope: ObjRef) {
        if self.scope_orders.contains_key(&scope) {
            return;
        }
        self.scope_orders.insert(scope, self.scope_list.clone());
        self.scope_list.push(scope);
    }

    /// Add one method under `name`. A pre-existing entry from the same
    /// `scope` is replaced in place (matching `addMethod`'s same-scope
    /// branch); otherwise the new entry becomes the highest-priority entry
    /// for `name`, matching `addFront`.
    pub fn add_method(&mut self, name: &str, scope: ObjRef, method: MethodId) {
        let key = name.to_ascii_uppercase();
        let list = self.entries.entry(key).or_default();
        if let Some(existing) = list.iter_mut().find(|e| e.scope == scope) {
            existing.method = method;
            return;
        }
        list.insert(0, MethodEntry { scope, method });
    }

    /// Overlay `source`'s methods on top of this dictionary's own -- the
    /// oracle's `MethodDictionary::mergeMethods`. Iterating `source` in
    /// reverse priority order and folding each entry in through
    /// [`Self::add_method`] is what preserves `source`'s own relative priority
    /// after the merge: the oracle does the identical reversal
    /// (`ReverseTableIterator`) for the identical reason, stated in its own
    /// comment -- "so the methods get added to our directory in the same
    /// relative order that they appear in the source directory".
    pub fn merge_methods(&mut self, source: &MethodDict) {
        for (name, list) in &source.entries {
            for entry in list.iter().rev() {
                self.add_method(name, entry.scope, entry.method);
            }
        }
    }

    /// Overlay `source`'s scope ordering on top of this dictionary's own --
    /// the oracle's `MethodDictionary::mergeScopes`.
    pub fn merge_scopes(&mut self, source: &MethodDict) {
        for &scope in &source.scope_list {
            self.add_scope(scope);
        }
    }

    /// Both methods and scopes -- the oracle's `MethodDictionary::merge`.
    pub fn merge(&mut self, source: &MethodDict) {
        self.merge_methods(source);
        self.merge_scopes(source);
    }

    /// Rewrite every entry's scope to `scope` -- the oracle's
    /// `MethodDictionary::setMethodScope`, what `inheritInstanceMethods`
    /// uses to make a donor's methods present under the recipient's own
    /// identity rather than the donor's.
    pub fn set_method_scope(&mut self, scope: ObjRef) {
        for list in self.entries.values_mut() {
            for entry in list.iter_mut() {
                entry.scope = scope;
            }
        }
    }

    /// True if any scope defines `name`.
    pub fn has_method(&self, name: &str) -> bool {
        self.entries
            .get(&name.to_ascii_uppercase())
            .is_some_and(|l| !l.is_empty())
    }

    /// Every name this dictionary answers to, regardless of which scope
    /// supplies it. This is what a **method-set** assertion reads --
    /// `superClasses`/`ancestors` cannot see a donation that added no
    /// scope, but this can.
    pub fn method_names(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .filter(|(_, l)| !l.is_empty())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// The highest-priority method for an ordinary (unscoped) send --
    /// index 0 of `name`'s entry list.
    pub fn lookup(&self, name: &str) -> Option<(ObjRef, MethodId)> {
        self.entries
            .get(&name.to_ascii_uppercase())
            .and_then(|l| l.first())
            .map(|e| (e.scope, e.method))
    }

    /// Resolve `name` as a scope-override send starting at `start_scope`,
    /// matching `MethodDictionary::findSuperMethod`: the first (highest
    /// priority) entry whose scope is `start_scope` itself, or appears in
    /// `start_scope`'s own snapshot. Returns `None` if `start_scope` was
    /// never folded into this dictionary, exactly like the oracle's
    /// belt-and-braces `scopeOrders->get` miss.
    pub fn lookup_from_scope(&self, name: &str, start_scope: ObjRef) -> Option<(ObjRef, MethodId)> {
        let visible = self.scope_orders.get(&start_scope)?;
        let list = self.entries.get(&name.to_ascii_uppercase())?;
        list.iter()
            .find(|e| e.scope == start_scope || visible.contains(&e.scope))
            .map(|e| (e.scope, e.method))
    }

    /// The immediate superscope for `scope`: the last entry of `scope`'s
    /// own snapshot, or `None` for the topmost scope (the oracle's
    /// `resolveSuperScope`, which returns `.nil` for the same case).
    pub fn resolve_super_scope(&self, scope: ObjRef) -> Option<ObjRef> {
        self.scope_orders.get(&scope)?.last().copied()
    }
}
