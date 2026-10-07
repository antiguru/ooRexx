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

//! Variable storage and reads: a slot's own storage, what an `EXPOSE` bound it
//! to, and the scope pools.

use crate::activation::{Activation, InstanceVar};
use crate::run::{NameShape, shape_of};
use crate::{Code, Interp, Novalue};
use rexx_core::{Body, ObjRef, SlotFrame};
use rexx_parse::SymbolId;

impl Interp {
    /// The variable slot `slot` of `frame` names: the frame's own storage,
    /// unless an `EXPOSE` in this activation bound that slot to a pool on the
    /// receiving object.
    #[inline(always)]
    pub(super) fn variable(&self, frame: SlotFrame, slot: usize) -> Option<ObjRef> {
        if self.activation_exposes(frame) {
            return self.exposed_variable(frame, slot);
        }
        self.roots.frame_slot(frame, slot)
    }

    /// Assigns the variable slot `slot` of `frame` names.
    #[inline(always)]
    /// `pub(crate)` for `crate::ir::Op::Store`'s own fast path, which writes
    /// a simple target's slot without going through `assign_evaluated`.
    pub(crate) fn set_variable(&mut self, frame: SlotFrame, slot: usize, value: ObjRef) {
        if self.activation_exposes(frame) {
            return self.set_exposed_variable(frame, slot, value);
        }
        self.roots.set_frame_slot(frame, slot, value);
    }

    /// Returns the variable slot `slot` of `frame` names to the uninitialised
    /// state, which is what `DROP` does. Measured on the oracle: a class
    /// method that exposes `v`, assigns it and drops it leaves a later `expose
    /// v` reading the derived name `V`.
    #[inline(always)]
    pub(super) fn clear_variable(&mut self, frame: SlotFrame, slot: usize) {
        if self.activation_exposes(frame) {
            return self.clear_exposed_variable(frame, slot);
        }
        self.roots.clear_frame_slot(frame, slot);
    }

    /// Whether the running activation has bound any name at all to a scope
    /// pool over `frame` -- the whole of what the three accessors above test
    /// before taking the frame, and the reason each of them is two functions.
    #[inline(always)]
    fn activation_exposes(&self, frame: SlotFrame) -> bool {
        let activation = self.activation();
        !activation.exposed.is_empty() && activation.frame == frame
    }

    /// [`Interp::variable`] for a **named** activation rather than the running
    /// one -- what `RexxContext~variables` reads a suspended context's pool
    /// through.
    pub(crate) fn variable_in(&self, activation: &Activation, slot: usize) -> Option<ObjRef> {
        let frame = activation.frame;
        match Interp::exposure_in(activation, frame, slot) {
            Some(var) => self
                .pools_of(var.owner)
                .and_then(|pools| pools.get(var.scope, &var.name)),
            None => self.roots.frame_slot_of(frame, slot),
        }
    }

    /// [`Interp::variable`]'s exposed half. A slot the list does not name is
    /// still an ordinary local, so this falls back rather than answering
    /// unset.
    #[cold]
    #[inline(never)]
    fn exposed_variable(&self, frame: SlotFrame, slot: usize) -> Option<ObjRef> {
        match self.exposure(frame, slot) {
            Some(var) => self
                .pools_of(var.owner)
                .and_then(|pools| pools.get(var.scope, &var.name)),
            None => self.roots.frame_slot(frame, slot),
        }
    }

    /// [`Interp::set_variable`]'s exposed half.
    #[cold]
    #[inline(never)]
    fn set_exposed_variable(&mut self, frame: SlotFrame, slot: usize, value: ObjRef) {
        // Field by field rather than through `Interp::exposure`, because the
        // name borrowed out of the activation has to stay live across the
        // `&mut self.heap` below; disjoint fields borrow independently where a
        // method taking `&self` would not.
        let activation = self
            .activity
            .running
            .as_deref()
            .expect("an activation is running");
        let Some(var) = Interp::exposure_in(activation, frame, slot) else {
            self.roots.set_frame_slot(frame, slot, value);
            return;
        };
        let object = Interp::pool_object(&mut self.heap, var.owner);
        if object.watched() {
            return self.store_watched_exposed(frame, slot, Some(value));
        }
        Interp::pools_in(object).set(var.scope, &var.name, value);
    }

    /// [`Interp::clear_variable`]'s exposed half.
    #[cold]
    #[inline(never)]
    fn clear_exposed_variable(&mut self, frame: SlotFrame, slot: usize) {
        let activation = self
            .activity
            .running
            .as_deref()
            .expect("an activation is running");
        let Some(var) = Interp::exposure_in(activation, frame, slot) else {
            self.roots.clear_frame_slot(frame, slot);
            return;
        };
        let object = Interp::pool_object(&mut self.heap, var.owner);
        if object.watched() {
            return self.store_watched_exposed(frame, slot, None);
        }
        Interp::pools_in(object).clear(var.scope, &var.name);
    }

    /// Assigns one name in one scope's pool on `owner`, outside any
    /// activation's exposure list -- what a generated `::ATTRIBUTE` setter
    /// writes through.
    pub(super) fn set_pool_variable(
        &mut self,
        owner: ObjRef,
        scope: ObjRef,
        name: &[u8],
        value: ObjRef,
    ) {
        let object = Interp::pool_object(&mut self.heap, owner);
        if object.watched() {
            return self.store_watched(owner, scope, name, Some(value));
        }
        Interp::pools_in(object).set(scope, name, value);
    }

    /// The value an accessor's or `DELEGATE`'s name holds in `scope`'s pool on
    /// `owner` (`RexxVariableBase::getValue(VariableDictionary *)`): a simple
    /// variable's value or derived name, the stem (made if the pool has none),
    /// or one tail of that stem.
    pub(crate) fn pool_value(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8]) -> ObjRef {
        match shape_of(name) {
            NameShape::Simple => match self
                .pools_of(owner)
                .and_then(|pools| pools.get(scope, name))
            {
                Some(value) => value,
                None => self.text(name),
            },
            NameShape::Stem => self.pool_stem(owner, scope, name),
            NameShape::Compound => {
                let (stem_name, key) = direct_compound(name);
                let stem = self.pool_stem(owner, scope, stem_name);
                self.stem_object_compound(stem, key)
            }
        }
    }

    /// Assigns an accessor's name in `scope`'s pool on `owner`
    /// (`RexxVariableBase::set(VariableDictionary *, ...)`): a stem takes a
    /// stem as it is and wraps anything else as a new stem's default, and a
    /// compound writes one tail of the pool's stem.
    pub(crate) fn set_pool_value(
        &mut self,
        owner: ObjRef,
        scope: ObjRef,
        name: &[u8],
        value: ObjRef,
    ) {
        match shape_of(name) {
            NameShape::Simple => self.set_pool_variable(owner, scope, name, value),
            NameShape::Stem => {
                let stem = self.stem_assignment_value(name, value);
                self.set_pool_variable(owner, scope, name, stem);
            }
            NameShape::Compound => {
                let (stem_name, key) = direct_compound(name);
                let stem = self.pool_stem(owner, scope, stem_name);
                self.stem_object_write(stem, key, value);
            }
        }
    }

    /// The stem `scope`'s pool on `owner` holds under `stem_name`, made empty
    /// if it holds none (`VariableDictionary::getStem`).
    fn pool_stem(&mut self, owner: ObjRef, scope: ObjRef, stem_name: &[u8]) -> ObjRef {
        if let Some(stem) = self
            .pools_of(owner)
            .and_then(|pools| pools.get(scope, stem_name))
        {
            return stem;
        }
        let stem = self.empty_stem(stem_name);
        self.set_pool_variable(owner, scope, stem_name, stem);
        stem
    }

    /// [`Interp::set_pool_variable`]'s other half: the name returns to the
    /// uninitialised state.
    pub(super) fn clear_pool_variable(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8]) {
        let object = Interp::pool_object(&mut self.heap, owner);
        if object.watched() {
            return self.store_watched(owner, scope, name, None);
        }
        Interp::pools_in(object).clear(scope, name);
    }

    /// `owner`, whose pools a variable lives in.
    ///
    /// # Panics
    ///
    /// Where `owner` is not live.
    fn pool_object(heap: &mut rexx_core::Heap, owner: ObjRef) -> &mut rexx_core::Object {
        heap.get_mut(owner)
            .expect("an object variable's owner is rooted")
    }

    /// `object`'s pools.
    ///
    /// # Panics
    ///
    /// Where `object` is not a `Body::Instance`.
    fn pools_in(object: &mut rexx_core::Object) -> &mut rexx_core::ScopePools {
        match &mut object.body {
            Body::Instance { pools, .. } => pools,
            _ => panic!("an object variable's owner is a Body::Instance"),
        }
    }

    /// A store of `value`, or a `DROP` where it is `None`, to the exposed
    /// variable of slot `slot` of `frame`, on an object a `GUARD WHEN` has
    /// watched.
    #[cold]
    #[inline(never)]
    fn store_watched_exposed(&mut self, frame: SlotFrame, slot: usize, value: Option<ObjRef>) {
        let Some(var) = self.exposure(frame, slot) else {
            return;
        };
        let (owner, scope, name) = (var.owner, var.scope, var.name.clone());
        self.store_watched(owner, scope, &name, value);
    }

    /// A store of `value`, or a `DROP` where it is `None`, of `name` in
    /// `scope`'s pool on `owner`, an object a `GUARD WHEN` has watched: every
    /// watcher of the variable is posted (`RexxVariable::set`, `::drop`,
    /// `::notify`, `execution/RexxVariable.cpp:158-194`). The oracle's
    /// notifier then yields, which hands its kernel lock only to an activity
    /// already queued for it; a woken watcher measured not to be one runs
    /// at the next switch, so no switch is asked for here.
    #[cold]
    #[inline(never)]
    fn store_watched(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8], value: Option<ObjRef>) {
        let pools = Interp::pools_in(Interp::pool_object(&mut self.heap, owner));
        match value {
            Some(value) => pools.set(scope, name, value),
            None => pools.clear(scope, name),
        }
        let watchers = self.activities.guards.watchers(owner, scope, name).to_vec();
        for watcher in watchers {
            self.post_guard(watcher);
        }
    }

    /// What an `EXPOSE` bound slot `slot` of `frame` to, if anything.
    pub(super) fn exposure(&self, frame: SlotFrame, slot: usize) -> Option<&InstanceVar> {
        Interp::exposure_in(self.activation(), frame, slot)
    }

    /// [`Interp::exposure`] over one activation, so that a caller holding a
    /// `&mut` borrow of another `Interp` field can still ask.
    fn exposure_in(activation: &Activation, frame: SlotFrame, slot: usize) -> Option<&InstanceVar> {
        if activation.exposed.is_empty() || activation.frame != frame {
            return None;
        }
        activation
            .exposed
            .iter()
            .find(|(at, _)| *at == slot)
            .map(|(_, var)| var)
    }

    /// The scope pools `owner` holds, for a reader.
    pub(crate) fn pools_of(&self, owner: ObjRef) -> Option<&rexx_core::ScopePools> {
        match self.heap.get(owner).map(|object| &object.body) {
            Some(Body::Instance { pools, .. }) => Some(pools),
            _ => None,
        }
    }

    /// Reads a variable, resolving its slot here.
    pub(super) fn read(&mut self, code: &Code<'_>, id: SymbolId) -> (ObjRef, Novalue) {
        self.read_at(code, id, None)
    }

    /// Reads a variable, by the slot the plan already resolved its id to, or
    /// by `at` when a compiler resolved the same thing earlier.
    pub(crate) fn read_at(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        at: Option<usize>,
    ) -> (ObjRef, Novalue) {
        let slot = match at {
            Some(slot) => slot,
            None => match code.slot_for(id) {
                Some(slot) => slot,
                None => self.slot_of(code.symbols.name(id).as_bytes()),
            },
        };
        let frame = self.activation().frame;
        match self.variable(frame, slot) {
            Some(value) => (value, Novalue::Set),
            None => (self.derived_name(code, id), Novalue::Unset),
        }
    }

    /// [`Interp::read_at`] for a slot already resolved, inlined into its
    /// caller.
    #[inline(always)]
    pub(crate) fn read_slot(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        slot: usize,
    ) -> (ObjRef, Novalue) {
        let frame = self.activation().frame;
        match self.variable(frame, slot) {
            Some(value) => (value, Novalue::Set),
            None => (self.derived_name(code, id), Novalue::Unset),
        }
    }

    /// What an uninitialised read yields: the derived name, which for a
    /// simple variable is its own upcased spelling.
    #[cold]
    #[inline(never)]
    fn derived_name(&mut self, code: &Code<'_>, id: SymbolId) -> ObjRef {
        let derived = code.symbols.name(id).as_bytes();
        self.text(derived)
    }
}

/// A compound name's stem, its period included, and its tail: everything
/// after the first period, taken literally (`buildCompoundVariable` with
/// `direct`, which `LanguageParser::getRetriever` builds for an attribute or
/// `DELEGATE` name).
fn direct_compound(name: &[u8]) -> (&[u8], &[u8]) {
    let dot = name
        .iter()
        .position(|&b| b == b'.')
        .expect("NameShape::Compound guarantees at least one period");
    name.split_at(dot + 1)
}
