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
            None => self.roots.frame_slot(frame, slot),
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
        let activation = self.running.as_deref().expect("an activation is running");
        let Some(var) = Interp::exposure_in(activation, frame, slot) else {
            self.roots.set_frame_slot(frame, slot, value);
            return;
        };
        let pools = self
            .heap
            .get_mut(var.owner)
            .map(|object| &mut object.body)
            .and_then(|body| match body {
                Body::Instance { pools, .. } => Some(pools),
                _ => None,
            })
            .expect("an exposed variable's owner is a rooted Body::Instance");
        pools.set(var.scope, &var.name, value);
    }

    /// [`Interp::clear_variable`]'s exposed half.
    #[cold]
    #[inline(never)]
    fn clear_exposed_variable(&mut self, frame: SlotFrame, slot: usize) {
        let activation = self.running.as_deref().expect("an activation is running");
        let Some(var) = Interp::exposure_in(activation, frame, slot) else {
            self.roots.clear_frame_slot(frame, slot);
            return;
        };
        let pools = self
            .heap
            .get_mut(var.owner)
            .map(|object| &mut object.body)
            .and_then(|body| match body {
                Body::Instance { pools, .. } => Some(pools),
                _ => None,
            })
            .expect("an exposed variable's owner is a rooted Body::Instance");
        pools.clear(var.scope, &var.name);
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
        let pools = self
            .heap
            .get_mut(owner)
            .map(|object| &mut object.body)
            .and_then(|body| match body {
                Body::Instance { pools, .. } => Some(pools),
                _ => None,
            })
            .expect("Interp::pool_owner answers a rooted Body::Instance");
        pools.set(scope, name, value);
    }

    /// [`Interp::set_pool_variable`]'s other half: the name returns to the
    /// uninitialised state.
    pub(super) fn clear_pool_variable(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8]) {
        let pools = self
            .heap
            .get_mut(owner)
            .map(|object| &mut object.body)
            .and_then(|body| match body {
                Body::Instance { pools, .. } => Some(pools),
                _ => None,
            })
            .expect("Interp::pool_owner answers a rooted Body::Instance");
        pools.clear(scope, name);
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
