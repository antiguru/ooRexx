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

use rexx_core::{MethodId, NameMap, ObjRef};
use std::collections::BTreeSet;

/// One entry under one name. Several of these can share a name when more
/// than one class in the ancestor chain defines it -- that is exactly the
/// case a scope-override (`SUPER`) send has to distinguish.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MethodSlot {
    /// A real, invokable entry: the scope it was defined at and the method
    /// it names.
    Defined { scope: ObjRef, method: MethodId },
    /// The oracle's `.nil` tombstone. It carries neither scope nor method,
    /// which is not an omission here but the shape of `hideMethod`'s own
    /// `put(TheNilObject, name)`: the two dictionary walks that read scopes,
    /// `setMethodScope` (`MethodDictionary.cpp:486`) and `findSuperMethod`
    /// (`:453`), each test for `.nil` and skip it rather than asking it for
    /// one.
    Hidden,
}

impl MethodSlot {
    /// The scope this entry was defined at, or `None` for a tombstone.
    fn scope(self) -> Option<ObjRef> {
        match self {
            MethodSlot::Defined { scope, .. } => Some(scope),
            MethodSlot::Hidden => None,
        }
    }
}

/// A flat name-to-method(s) map, plus the scope ordering a scope-override
/// send needs to resolve correctly.
#[derive(Clone, Default)]
pub struct MethodDict {
    entries: NameMap<String, Vec<MethodSlot>>,
    scope_list: Vec<ObjRef>,
    scope_orders: NameMap<ObjRef, Vec<ObjRef>>,
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
        if let Some(existing) = list.iter_mut().find(|e| e.scope() == Some(scope)) {
            *existing = MethodSlot::Defined { scope, method };
            return;
        }
        list.insert(0, MethodSlot::Defined { scope, method });
    }

    /// Replace everything under `name` with one entry -- the oracle's
    /// `MethodDictionary::replaceMethod`, which is a bare `put` and whose own
    /// comment says why: "We might be replacing a method inherited from
    /// another class, so we want to ensure that any existing method by that
    /// name is completely removed" (`MethodDictionary.cpp:200`-`:202`, over
    /// the `put` at `:211`).
    pub fn replace_method(&mut self, name: &str, scope: ObjRef, method: MethodId) {
        let key = name.to_ascii_uppercase();
        self.entries
            .insert(key, vec![MethodSlot::Defined { scope, method }]);
    }

    /// Add a tombstone under `name` as its highest-priority entry, leaving
    /// whatever was already there behind it -- `addMethod`'s own no-method
    /// branch, which is an unconditional `addFront(TheNilObject, name)`
    /// (`MethodDictionary.cpp:166`-`:171`).
    fn add_hidden(&mut self, name: &str) {
        let key = name.to_ascii_uppercase();
        self.entries
            .entry(key)
            .or_default()
            .insert(0, MethodSlot::Hidden);
    }

    /// Replace everything under `name` with a tombstone -- `hideMethod`'s
    /// `put(TheNilObject, name)` (`MethodDictionary.cpp:350`), and the
    /// omitted-second-argument arm of `RexxClass::defineMethod`
    /// (`ClassClass.cpp:836`-`:843`), which reaches `replaceMethod`'s `put`
    /// at `:864` with the same `.nil`.
    pub fn hide_method(&mut self, name: &str) {
        let key = name.to_ascii_uppercase();
        self.entries.insert(key, vec![MethodSlot::Hidden]);
    }

    /// Take `name` out of the dictionary entirely -- the oracle's
    /// `MethodDictionary::removeMethod`, whose `bool` says whether there was
    /// anything to remove. `RexxClass::deleteMethod` reads that answer: it
    /// propagates to the subclasses only when the dictionary actually
    /// changed (`ClassClass.cpp:966`).
    pub fn remove_method(&mut self, name: &str) -> bool {
        let key = name.to_ascii_uppercase();
        self.entries
            .remove(&key)
            .is_some_and(|list| !list.is_empty())
    }

    /// The highest-priority entry under `name`, tombstone included -- the
    /// oracle's `getMethod`, which `RexxClass::method` reads and
    /// `methodLookup` filters.
    pub fn slot(&self, name: &str) -> Option<MethodSlot> {
        // Every send arrives here, and the uppercase copy was an allocation
        // per send. `to_ascii_uppercase` changes a name exactly when it
        // holds an ASCII lower-case byte, so the two arms answer alike and
        // the common one -- a name the scanner already upcased -- borrows.
        let list = if name.bytes().any(|b| b.is_ascii_lowercase()) {
            self.entries.get(&name.to_ascii_uppercase())
        } else {
            self.entries.get(name)
        };
        list.and_then(|l| l.first()).copied()
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
                match *entry {
                    MethodSlot::Defined { scope, method } => {
                        self.add_method(name, scope, method);
                    }
                    MethodSlot::Hidden => self.add_hidden(name),
                }
            }
        }
    }

    /// Overlay `source`'s methods **and its already-folded scope order** on
    /// top of this dictionary -- the oracle's `RexxBehaviour::merge`
    /// (`RexxBehaviour.cpp:649`), which calls `MethodDictionary::merge`
    /// (`:666`): `mergeMethods` (identical to [`Self::merge_methods`]) followed
    /// by `mergeScopes`, which folds every scope already in `source`'s own
    /// `scope_list`, in `source`'s order, into this one via [`Self::add_scope`].
    /// This is distinct from [`Self::merge_methods`] alone: that merges methods
    /// only, and is what an ordinary `INHERIT` mixin cascade uses
    /// (`class_graph.rs`'s `cascade_build`) because a mixin's own ancestors are
    /// walked independently by that same cascade. This method is for the one
    /// case where the source's scope history has to come along wholesale
    /// instead of being re-derived: `createClassBehaviour`'s metaclass merge
    /// (D44, `ClassClass.cpp:1123-1127`), where a class's class-behaviour
    /// absorbs its metaclass's already-built instance behaviour verbatim.
    pub fn merge(&mut self, source: &MethodDict) {
        self.merge_methods(source);
        for &scope in &source.scope_list {
            self.add_scope(scope);
        }
    }

    /// Copy `source`'s entries **defined at `filter`** into this dictionary
    /// under `scope`, leaving `source` untouched -- the oracle's
    /// `MethodDictionary::replaceMethods(source, filterScope, scope)`
    /// (`behaviour/MethodDictionary.cpp:251`).
    pub fn replace_methods_from(&mut self, source: &MethodDict, filter: ObjRef, scope: ObjRef) {
        for (name, list) in &source.entries {
            for entry in list.iter().rev() {
                match *entry {
                    MethodSlot::Defined {
                        scope: defined_at,
                        method,
                    } if defined_at == filter => self.add_method(name, scope, method),
                    MethodSlot::Defined { .. } => {}
                    MethodSlot::Hidden => self.add_hidden(name),
                }
            }
        }
    }

    /// Rewrite every entry's scope to `scope` -- the oracle's
    /// `MethodDictionary::setMethodScope`, what `Class~inheritInstanceMethods`
    /// uses to make a donor's methods present under the recipient's own
    /// identity rather than the donor's.
    pub fn set_method_scope(&mut self, scope: ObjRef) {
        for list in self.entries.values_mut() {
            for entry in list.iter_mut() {
                if let MethodSlot::Defined { method, .. } = *entry {
                    *entry = MethodSlot::Defined { scope, method };
                }
            }
        }
    }

    /// True if `name` resolves -- `RexxBehaviour::hasMethod`, which is
    /// `methodLookup(name) != OREF_NULL` (`RexxBehaviour.hpp:82`) and so
    /// answers false for a name whose highest-priority entry is a tombstone.
    pub fn has_method(&self, name: &str) -> bool {
        matches!(self.slot(name), Some(MethodSlot::Defined { .. }))
    }

    /// Every name this dictionary answers to, regardless of which scope
    /// supplies it. A name whose highest-priority entry is a tombstone is
    /// not one of them, matching [`Self::has_method`]. This is what a **method-set** assertion reads --
    /// `superClasses`/`ancestors` cannot see a donation that added no
    /// scope, but this can.
    pub fn method_names(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .filter(|(_, l)| matches!(l.first(), Some(MethodSlot::Defined { .. })))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// The highest-priority method for an ordinary (unscoped) send --
    /// index 0 of `name`'s entry list.
    pub fn lookup(&self, name: &str) -> Option<(ObjRef, MethodId)> {
        match self.slot(name) {
            Some(MethodSlot::Defined { scope, method }) => Some((scope, method)),
            Some(MethodSlot::Hidden) | None => None,
        }
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
            .filter_map(|e| match *e {
                MethodSlot::Defined { scope, method } => Some((scope, method)),
                MethodSlot::Hidden => None,
            })
            .find(|(scope, _)| *scope == start_scope || visible.contains(scope))
    }

    /// The immediate superscope for `scope`: the last entry of `scope`'s
    /// own snapshot, or `None` for the topmost scope (the oracle's
    /// `resolveSuperScope`, which returns `.nil` for the same case).
    pub fn resolve_super_scope(&self, scope: ObjRef) -> Option<ObjRef> {
        self.scope_orders.get(&scope)?.last().copied()
    }
}
