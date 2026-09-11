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

//! Stems and compound variables (D15a): four measured behaviours an obvious
//! model gets wrong.

use crate::plan::{CompoundName, TailPiece};
use crate::{Code, Failure, Interp, Novalue};
use rexx_core::{BehaviourId, Body, Decoded, ObjRef};
use rexx_parse::SymbolId;

/// Names a `Body` variant without printing it.
fn body_variant_name(body: &Body) -> &'static str {
    match body {
        Body::Text { .. } => "Body::Text",
        Body::Num { .. } => "Body::Num",
        Body::Stem { .. } => "Body::Stem",
        Body::Array { .. } => "Body::Array",
        Body::Instance { .. } => "Body::Instance",
        Body::WeakRef(_) => "Body::WeakRef",
        Body::Class { .. } => "Body::Class",
        Body::Native(_) => "Body::Native",
        Body::VarRef(_) => "Body::VarRef",
    }
}

impl Interp {
    /// Resolves a compound's tail pieces into the one key its tails map is
    /// keyed by (D15a): each piece verbatim and case-sensitively, joined by
    /// `.`. `TailPiece::Constant` stands for itself; `TailPiece::Variable` is
    /// a plain variable name whose *current value* supplies the piece, read
    /// through the ordinary variable path (deriving its own name if unset,
    /// the same as any read) and rendered with `to_text`.
    pub(crate) fn take_key_buffer(&mut self) -> Vec<u8> {
        let mut buffer = std::mem::take(&mut self.key_buffer);
        buffer.clear();
        buffer
    }

    /// Hands the buffer back for the next builder.
    pub(crate) fn give_key_buffer(&mut self, buffer: Vec<u8>) {
        self.key_buffer = buffer;
    }

    /// [`Interp::tail_key`], building into a caller's buffer rather than a
    /// fresh one. The buffer is cleared first, so a caller may pass one that
    /// still holds an earlier key.
    pub(crate) fn tail_key_into(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        out: &mut Vec<u8>,
    ) -> Result<(), Failure> {
        out.clear();
        self.append_tail_key(code, id, out)
    }

    /// [`Interp::tail_key`]'s body, appending to a caller's buffer.
    fn append_tail_key(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        out: &mut Vec<u8>,
    ) -> Result<(), Failure> {
        match code.compound(id) {
            Some(entry) => self.append_tails(&entry.tails, out),
            None => self.append_tails(&CompoundName::split(code.symbols.name(id)).tails, out),
        }
    }

    pub(crate) fn tail_key(&mut self, code: &Code<'_>, id: SymbolId) -> Result<Vec<u8>, Failure> {
        match code.compound(id) {
            Some(entry) => self.join_tails(&entry.tails),
            None => self.join_tails(&CompoundName::split(code.symbols.name(id)).tails),
        }
    }

    /// Joins one compound's tail pieces into the key its stem's tails map is
    /// keyed by, resolving each `Variable` piece against the current frame.
    fn join_tails(&mut self, tails: &[TailPiece]) -> Result<Vec<u8>, Failure> {
        let mut key = Vec::new();
        self.append_tails(tails, &mut key)?;
        Ok(key)
    }

    /// [`Interp::join_tails`], appending to a caller's buffer.
    fn append_tails(&mut self, tails: &[TailPiece], key: &mut Vec<u8>) -> Result<(), Failure> {
        for (index, piece) in tails.iter().enumerate() {
            if index > 0 {
                key.push(b'.');
            }
            match piece {
                TailPiece::Constant(text) => key.extend_from_slice(text),
                TailPiece::Variable { name, at } => {
                    // **The tripwire for pieces resolved against a plan that
                    // is not the running activation's.** A slot lands on a
                    // piece because `Plan::slot_for` put its name in that
                    // plan's own name map, and `Interp::slot_of` reads that
                    // map before it reads `extra` -- so the two agree for as
                    // long as the entry and the activation come from one
                    // plan, and `Code::plan` is what pairs them. Mismatched,
                    // this reads whatever else lives at that index rather
                    // than failing, which is a wrong value found by chasing
                    // it.
                    debug_assert!(
                        at.is_none() || self.bound_slot_of(name) == *at,
                        "a compound tail piece names a slot this activation's plan does not \
                         give its name"
                    );
                    let value = self.read_by_name_at(name, *at);
                    // **A substituted tail is a `reqstr` context**, and its
                    // own one: `RexxInternalObject::copyIntoTail` converts
                    // with `requestString()` (`classes/ObjectClass.cpp:1204`)
                    // rather than rendering. Measured, oracle rc 0:
                    // `a.MS = 'hit'; i = .K; say a.i` with a class-side
                    // `makeString` returning `'MS'` prints `hit`.
                    let value = self.required_string_value(value)?;
                    self.write_text(value, key);
                }
            }
        }
        Ok(())
    }

    /// Reads a variable by name alone, the same slot machinery `read` uses
    /// but with no `SymbolId` to try first: a tail piece from
    /// `compound_parts` is a borrowed `&str`, not a token, so there is no id
    /// for it (`ExprKind::Compound`'s doc comment on `ast.rs`). Unset
    /// derives the name's own (already upcased) spelling, same as any
    /// uninitialised simple-variable read.
    pub(crate) fn read_by_name(&mut self, name: &[u8]) -> ObjRef {
        self.read_by_name_at(name, None)
    }

    /// [`Interp::read_by_name`] with the slot already in hand, which is what a
    /// tail piece carries when the plan that recorded it assigned one
    /// (`TailPiece::Variable`).
    fn read_by_name_at(&mut self, name: &[u8], at: Option<usize>) -> ObjRef {
        let slot = match at {
            Some(slot) => slot,
            None => self.slot_of(name),
        };
        let frame = self.activation().frame;
        match self.variable(frame, slot) {
            Some(value) => value,
            None => self.text(name),
        }
    }

    /// Reads a bare stem (`ExprKind::Stem`): D15a's own auto-vivification
    /// rule (branch review F4), mirroring the oracle's `createStemVariable`,
    /// which fires on any miss to the variable dictionary, reads included
    /// (`VariableDictionary::getStemVariable`, `VariableDictionary.hpp:
    /// 178-183`).
    pub(crate) fn read_stem(&mut self, name: &[u8]) -> ObjRef {
        self.read_stem_at(name, None)
    }

    /// [`Interp::read_stem`] with the slot already in hand. Two callers have
    /// one and they do not take it from the same place: `crate::ir::Op::Load`
    /// carries the slot the chunk was compiled with, and a controlled loop's
    /// re-test reads its stem control's off the entry `Code::compound` holds.
    /// A read reached from `eval.rs` has none.
    pub(crate) fn read_stem_at(&mut self, name: &[u8], at: Option<usize>) -> ObjRef {
        let slot = match at {
            Some(slot) => {
                // `Interp::stem_slot`'s tripwire, on the reading side, for the
                // same premise and against the same map: a slot reaches either
                // caller because `Plan::slot_for` put this name in the plan's
                // own name map, so a slot that plan cannot reproduce came from
                // somewhere else and would read another variable's value
                // rather than fail.
                debug_assert_eq!(
                    self.bound_slot_of(name),
                    Some(slot),
                    "a stem read names a slot this activation has not bound to its name"
                );
                slot
            }
            None => self.slot_of(name),
        };
        let frame = self.activation().frame;
        if let Some(value) = self.variable(frame, slot) {
            return value;
        }
        let stem = self.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: name.into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        self.set_variable(frame, slot, stem);
        stem
    }

    /// The frame slot a compound's stem is held in: the one the upfront pass
    /// recorded on the entry, or the full three-source resolution of
    /// `stem_name` when the entry carries none.
    fn stem_slot(&mut self, stem_name: &[u8], at: Option<usize>) -> usize {
        match at {
            Some(slot) => {
                // **The tripwire for an entry resolved against a plan that is
                // not the running activation's.** A slot lands on a
                // `CompoundName` because `Plan::slot_for` put its name in that
                // plan's own name map, and `Interp::slot_of` reads that map
                // before it reads `extra` -- so the two agree for as long as
                // the entry and the activation come from one plan, and
                // `Code::plan` is what pairs them. Mismatched, this reads or
                // writes whatever else lives at that index rather than
                // failing, which is a wrong value found by chasing it.
                debug_assert_eq!(
                    self.bound_slot_of(stem_name),
                    Some(slot),
                    "a stem names a slot this activation has not bound to its name"
                );
                slot
            }
            None => self.slot_of(stem_name),
        }
    }

    /// Reads a tail: `stem_name` is the **read site's** own name (used only
    /// to find the slot), including its trailing period; `key` is
    /// `tail_key`'s output.
    /// ```text
    /// a.1='x'; b.=a.;             say b.2  ->  A.2   (not B.2)
    /// c.1='x'; d.=c.; drop d.1;   say d.1  ->  C.1   (not D.1)
    /// g.1='x'; h.=g.; k.=h.;      say k.9  ->  G.9   (two hops, still G)
    /// m.1='x'; n.2='y'; m.=n.;    say m.1  ->  N.1   (m.'s own object discarded)
    /// ```
    /// **Returns the `Novalue` flag alongside the value since 4b's Task 7**,
    /// the same pair `Interp::read` has always returned and for the same
    /// reader: `SIGNAL ON NOVALUE`. `Novalue::Unset` is precisely the two
    /// derived-name exits below -- no object at all, or an object with
    /// nothing at this key -- and measured against the oracle both count,
    /// since `signal on novalue` traps on `say zunset.1` whether or not the
    /// stem itself has ever been assigned.
    pub(crate) fn stem_get(&mut self, stem_name: &[u8], key: &[u8]) -> (ObjRef, Novalue) {
        self.stem_get_at(stem_name, None, key)
    }

    /// [`Interp::stem_get`] with the stem's slot already in hand, which is
    /// what `CompoundName::stem_at` carries when the plan that recorded the
    /// entry assigned one. [`Interp::stem_slot`] is what `at` means here.
    pub(crate) fn stem_get_at(
        &mut self,
        stem_name: &[u8],
        at: Option<usize>,
        key: &[u8],
    ) -> (ObjRef, Novalue) {
        let slot = self.stem_slot(stem_name, at);
        let frame = self.activation().frame;
        let stem_value = match self.variable(frame, slot) {
            Some(v) => v,
            // No object at all: the read site's own spelling is all there
            // is to derive from.
            None => return (self.derived_tail_name(stem_name, key), Novalue::Unset),
        };

        // `resolved` is computed, fully, before any further `self` call, so
        // the borrow on `self.heap` never has to overlap one.
        let resolved = {
            let object = self.heap.get(stem_value).expect("a live value");
            let Body::Stem { default, tails, .. } = &object.body else {
                unreachable!(
                    "a stem-named slot holds only Body::Stem, got {}",
                    body_variant_name(&object.body)
                );
            };
            match tails.get(key) {
                Some((_, Some(value))) => Some(*value),
                Some((_, None)) => None, // the tombstone: absent from the default too
                None => *default,        // an untouched tail falls back to the default
            }
        };

        match resolved {
            Some(value) => (value, Novalue::Set),
            // An object exists but this key does not resolve: derive from
            // the OBJECT's own name, not the read site's -- see this
            // function's doc comment for why the two can differ.
            None => {
                let object_name = {
                    let object = self.heap.get(stem_value).expect("a live value");
                    let Body::Stem { name, .. } = &object.body else {
                        unreachable!("the same object matched Body::Stem a moment ago");
                    };
                    name.clone()
                };
                (self.derived_tail_name(&object_name, key), Novalue::Unset)
            }
        }
    }

    /// Writes a tail: mutates the stem's existing object in place, or
    /// auto-vivifies one (`default: None`) if this is the first write the
    /// stem has ever seen (D15a: "a tail assignment mutates", and measured,
    /// `q.1='x'` with `q.` never itself assigned still leaves `q.2`
    /// deriving its own name rather than falling back to anything).
    pub(crate) fn stem_set(&mut self, stem_name: &[u8], key: &[u8], value: ObjRef) {
        self.stem_set_at(stem_name, None, key, value);
    }

    /// [`Interp::stem_set`] with the stem's slot already in hand, which is
    /// what `CompoundName::stem_at` carries when the plan that recorded the
    /// entry assigned one. [`Interp::stem_slot`] is what `at` means here.
    pub(crate) fn stem_set_at(
        &mut self,
        stem_name: &[u8],
        at: Option<usize>,
        key: &[u8],
        value: ObjRef,
    ) {
        let slot = self.stem_slot(stem_name, at);
        let frame = self.activation().frame;
        match self.variable(frame, slot) {
            Some(stem_value) => {
                let object = self.heap.get_mut(stem_value).expect("a live value");
                let Body::Stem { tails, .. } = &mut object.body else {
                    unreachable!(
                        "a stem-named slot holds only Body::Stem, got {}",
                        body_variant_name(&object.body)
                    );
                };
                // **Looked up before it is inserted, because the key is
                // already there on all but the first write to a tail.**
                // `insert` needs an owned key whether or not it keeps it, so
                // an unconditional one allocates a copy on every assignment
                // and drops it again the moment the map finds the key it
                // already holds. Measured with `heaptrack` on
                // `bench-programs/compound.rex`, which writes 500 tails five
                // million times: this was one allocation per iteration and
                // essentially the whole of what that program allocated.
                let next = tails.len();
                match tails.get_mut(key) {
                    Some((_, existing)) => *existing = Some(value),
                    None => {
                        tails.insert(key.to_vec(), (next, Some(value)));
                    }
                }
            }
            None => {
                let mut tails = rexx_core::NameMap::default();
                tails.insert(key.to_vec(), (0, Some(value)));
                let stem = self.alloc_with(
                    BehaviourId::STEM,
                    Body::Stem {
                        name: stem_name.into(),
                        default: None,
                        tails,
                    },
                );
                self.set_variable(frame, slot, stem);
            }
        }
    }

    /// Drops one tail: a tombstone (`Some(key) -> None`) in the stem's
    /// existing object, which does not take the default. If the stem has
    /// never been touched there is nothing to record: an absent key and a
    /// tombstone with no default already render identically (both derive
    /// the name), so this is a genuine no-op rather than an auto-vivified
    /// object nobody would ever observe the difference from.
    pub(crate) fn stem_drop_tail(&mut self, stem_name: &[u8], key: &[u8]) {
        self.stem_drop_tail_at(stem_name, None, key);
    }

    /// [`Interp::stem_drop_tail`] with the stem's slot already in hand, which
    /// is what `CompoundName::stem_at` carries when the plan that recorded
    /// the entry assigned one. [`Interp::stem_slot`] is what `at` means here.
    pub(crate) fn stem_drop_tail_at(&mut self, stem_name: &[u8], at: Option<usize>, key: &[u8]) {
        let slot = self.stem_slot(stem_name, at);
        let frame = self.activation().frame;
        if let Some(stem_value) = self.variable(frame, slot) {
            let object = self.heap.get_mut(stem_value).expect("a live value");
            let Body::Stem { tails, .. } = &mut object.body else {
                unreachable!(
                    "a stem-named slot holds only Body::Stem, got {}",
                    body_variant_name(&object.body)
                );
            };
            // A drop keeps the tail's place, so the ordinal survives it.
            let next = tails.len();
            let ordinal = tails.get(key).map_or(next, |(at, _)| *at);
            tails.insert(key.to_vec(), (ordinal, None));
        }
    }

    /// `stem. = expr`: replaces the whole object and rebinds the variable
    /// (D15a).
    pub(crate) fn stem_assign(&mut self, stem_name: &[u8], value: ObjRef) {
        self.stem_assign_at(stem_name, None, value);
    }

    /// [`Interp::stem_assign`] with the stem's slot already in hand, which is
    /// what an `ExprKind::Stem` assignment target carries: the symbol whose
    /// spelling `stem_name` is was bound to that slot by `Plan::bind`.
    /// [`Interp::stem_slot`] is what `at` means here.
    pub(crate) fn stem_assign_at(&mut self, stem_name: &[u8], at: Option<usize>, value: ObjRef) {
        if self.is_stem(value) {
            let slot = self.stem_slot(stem_name, at);
            let frame = self.activation().frame;
            self.set_variable(frame, slot, value);
        } else {
            self.replace_stem(stem_name, at, Some(value));
        }
    }

    /// `drop stem.`: replaces the whole object with a fresh, empty one
    /// (`default: None`, no tails) and rebinds the variable -- the same
    /// "replace and rebind" `stem_assign` uses, with nothing to wrap.
    pub(crate) fn stem_drop(&mut self, stem_name: &[u8]) {
        // `None`: every caller reaches this from a plain string naming a
        // variable -- `drop_by_name`, which serves an already-upcased `DROP`
        // target and every word of an indirect subsidiary list alike -- and
        // has no slot to hand over.
        self.replace_stem(stem_name, None, None);
    }

    /// The shared half of `stem_assign`'s "wrap" branch and `stem_drop`:
    /// build a fresh `Body::Stem` with the given default and rebind the
    /// variable to it, leaving any old object exactly where aliases into it
    /// already point (D15a's `r.`/`u` and `s.`/`t` transcripts, which need
    /// the *old* object left untouched rather than mutated).
    fn replace_stem(&mut self, stem_name: &[u8], at: Option<usize>, default: Option<ObjRef>) {
        let slot = self.stem_slot(stem_name, at);
        let frame = self.activation().frame;
        let stem = self.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: stem_name.into(),
                default,
                tails: rexx_core::NameMap::default(),
            },
        );
        self.set_variable(frame, slot, stem);
    }

    /// The derived name for a tail with no value to answer: the stem's own
    /// name (which already carries its trailing period) with the key
    /// appended, e.g. `NEVER_TOUCHED.` + `5` -> `NEVER_TOUCHED.5`. Never
    /// called with an empty `key` -- a *bare* stem's derived name is the
    /// ordinary uninitialised-variable path in `read`, not this function.
    fn derived_tail_name(&mut self, stem_name: &[u8], key: &[u8]) -> ObjRef {
        let mut name = stem_name.to_vec();
        name.extend_from_slice(key);
        self.text(&name)
    }

    /// What a stem-shaped name is *worth* when `value` is assigned to it:
    /// `value` itself when it is already a stem, and otherwise a fresh stem
    /// object carrying it as the new default.
    pub(crate) fn stem_assignment_value(&mut self, stem_name: &[u8], value: ObjRef) -> ObjRef {
        if self.is_stem(value) {
            return value;
        }
        self.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: stem_name.into(),
                default: Some(value),
                tails: rexx_core::NameMap::default(),
            },
        )
    }

    /// Whether `value` is currently a heap `Body::Stem`, which is what
    /// `stem_assign` needs to decide "share the same object" from "wrap as
    /// my new default" (see its doc comment). `.nil` and `SmallInt` are
    /// never stems, and are rejected before the only heap lookup this makes.
    fn is_stem(&self, value: ObjRef) -> bool {
        match value.decode() {
            Decoded::Heap { .. } => matches!(
                self.heap.get(value).map(|object| &object.body),
                Some(Body::Stem { .. })
            ),
            _ => false,
        }
    }

    /// Whether `value` is a stem holding no content: no default, and no tail
    /// that still has a value.
    /// ```text
    /// (nothing)                    -> referenceable   (slot is None)
    /// say q.                       -> referenceable   (read_stem vivifies an empty stem)
    /// q.1='x'; drop q.             -> referenceable   (stem_drop leaves an empty stem)
    /// q.1='x'; drop q.1            -> referenceable   (a tombstone is not content)
    /// q.1='x'; q.2='y'; drop q.1;
    ///          drop q.2            -> referenceable   (all tails tombstoned)
    /// q.1='local'                  -> 98.995
    /// q.='dflt'                    -> 98.995
    /// q.1='x'; q.2='y'; drop q.1   -> 98.995          (one live tail left)
    /// q.='dflt'; drop q.1          -> 98.995          (the default is content)
    /// ```
    pub(crate) fn is_uninitialised_stem(&self, value: ObjRef) -> bool {
        match value.decode() {
            Decoded::Heap { .. } => matches!(
                self.heap.get(value).map(|object| &object.body),
                Some(Body::Stem {
                    default: None,
                    tails,
                    ..
                }) if tails.values().all(|(_, tail)| tail.is_none())
            ),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{BodyKind, Plan};
    use crate::{Activation, BodyKey, ProgramId};
    use rexx_num::{Form, Number};
    use rexx_parse::{ExprKind, InstructionKind, Program, parse_program};
    use std::rc::Rc;

    /// Pushes a fresh top-level activation for `program`, the same setup
    /// `Interp::run` does inline, but stopping short of calling
    /// `run_activation` -- these tests call stem functions directly rather
    /// than through the instruction loop, which does not yet dispatch
    /// `ExprKind::Stem`/`Compound` at all (that wiring is a later task's).
    /// Never popped: the whole `Interp` drops at the end of the test, and
    /// nothing here needs the frame released early.
    fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
        let program = Rc::new(program);
        let program_id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: program_id,
                directive: None,
            },
            &program.main,
            &program.symbols,
            &program.source,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp.push_activation(Activation::new(
            id,
            Rc::clone(&program),
            program_id,
            plan,
            frame,
        ));
        program
    }

    /// A trivial activated program, for tests that only need `stem_get`,
    /// `stem_set`, `stem_drop_tail`, `stem_assign` and `stem_drop` -- none
    /// of which need a real `SymbolId`, only the stem's name and a key as
    /// bytes.
    fn activated(interp: &mut Interp) {
        activate(
            interp,
            parse_program(b"nop".to_vec()).expect("a trivial program parses"),
        );
    }

    /// Parses `text` (one `SAY` of a compound expression) and returns its
    /// `Compound` `SymbolId`, with no activation pushed -- for a test that
    /// needs a second compound's `SymbolId` while keeping the *first*
    /// activation (and its frame, where a piece variable's value lives) on
    /// top, rather than shadowing it with a fresh, empty one.
    fn parse_compound(text: &[u8]) -> (Program, SymbolId) {
        let program = parse_program(text.to_vec()).expect("test program parses");
        let id = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => match expr.kind {
                ExprKind::Compound(id) => id,
                ref other => panic!("expected a compound expression, got {other:?}"),
            },
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        (program, id)
    }

    /// `parse_compound`, activated: for a test that only ever needs one
    /// compound expression and is happy for its program to also be the
    /// activation's own.
    fn compound_id(interp: &mut Interp, text: &[u8]) -> (Rc<Program>, SymbolId) {
        let (program, id) = parse_compound(text);
        (activate(interp, program), id)
    }

    // The seven transcripts, each re-verified against the oracle in the task
    // report before being encoded here.

    #[test]
    fn a_dropped_tail_is_a_tombstone_that_does_not_take_the_default() {
        // u. = 'd' ; u.1 = 'one' ; drop u.1 ; say u.1 -> U.1 ; say u.2 -> d
        let mut interp = Interp::new();
        activated(&mut interp);

        let d = interp.text(b"d");
        interp.stem_assign(b"U.", d);
        let one = interp.text(b"one");
        interp.stem_set(b"U.", b"1", one);
        interp.stem_drop_tail(b"U.", b"1");

        let u1 = interp.stem_get(b"U.", b"1").0;
        assert_eq!(&*interp.to_text(u1), b"U.1");
        let u2 = interp.stem_get(b"U.", b"2").0;
        assert_eq!(&*interp.to_text(u2), b"d");
    }

    #[test]
    fn bare_stem_assignment_shares_the_object_when_the_value_is_already_a_stem() {
        // a. = 1 ; b. = a. ; a.1 = 2 ; say b.1 -> 2
        // a. = 9 (afterward) ; say b. -> 1 ; say b.1 -> 2  (b. keeps the OLD object)
        let mut interp = Interp::new();
        activated(&mut interp);

        let one = interp.number(Number::parse("1").unwrap(), 9, Form::Scientific);
        interp.stem_assign(b"A.", one);
        let a_value = interp.read_stem(b"A.");
        interp.stem_assign(b"B.", a_value);

        let two = interp.number(Number::parse("2").unwrap(), 9, Form::Scientific);
        interp.stem_set(b"A.", b"1", two);

        let b1 = interp.stem_get(b"B.", b"1").0;
        assert_eq!(&*interp.to_text(b1), b"2");

        let nine = interp.number(Number::parse("9").unwrap(), 9, Form::Scientific);
        interp.stem_assign(b"A.", nine);

        let b_bare = interp.read_stem(b"B.");
        assert_eq!(&*interp.to_text(b_bare), b"1");
        let b1_again = interp.stem_get(b"B.", b"1").0;
        assert_eq!(&*interp.to_text(b1_again), b"2");
    }

    #[test]
    fn dropping_the_whole_stem_leaves_an_old_alias_intact() {
        // r. = 'rd' ; u = r. ; drop r. ; say u -> rd
        let mut interp = Interp::new();
        activated(&mut interp);

        let rd = interp.text(b"rd");
        interp.stem_assign(b"R.", rd);
        let r_value = interp.read_stem(b"R.");
        // `u = r.`: an ordinary simple-variable assignment, aliasing whatever
        // `r.`'s slot currently holds -- no stem function involved, because
        // the target is not a stem.
        let u_slot = interp.slot_of(b"U");
        let frame = interp.activation().frame;
        interp.roots.set_frame_slot(frame, u_slot, r_value);

        interp.stem_drop(b"R.");

        let u_value = interp.roots.frame_slot(frame, u_slot).expect("u was set");
        assert_eq!(&*interp.to_text(u_value), b"rd");
    }

    #[test]
    fn reassigning_the_whole_stem_leaves_an_old_alias_intact() {
        // s. = 'def' ; t = s. ; s. = 'other' ; say t -> def
        let mut interp = Interp::new();
        activated(&mut interp);

        let def = interp.text(b"def");
        interp.stem_assign(b"S.", def);
        let s_value = interp.read_stem(b"S.");
        let t_slot = interp.slot_of(b"T");
        let frame = interp.activation().frame;
        interp.roots.set_frame_slot(frame, t_slot, s_value);

        let other = interp.text(b"other");
        interp.stem_assign(b"S.", other);

        let t_value = interp.roots.frame_slot(frame, t_slot).expect("t was set");
        assert_eq!(&*interp.to_text(t_value), b"def");
    }

    #[test]
    fn an_untouched_stem_derives_its_own_name_with_the_period() {
        // say q. -> Q.
        let mut interp = Interp::new();
        activated(&mut interp);
        let q = interp.read_stem(b"Q.");
        assert_eq!(&*interp.to_text(q), b"Q.");
    }

    #[test]
    fn reading_an_untouched_stem_auto_vivifies_a_shared_object() {
        // b. = a. (a. never touched) ; a.1 = 5 ; say b.1 -> 5 ; say b.7 ->
        // A.7 (measured against the oracle; branch review F4).
        let mut interp = Interp::new();
        activated(&mut interp);

        let a_value = interp.read_stem(b"A.");
        interp.stem_assign(b"B.", a_value);

        let five = interp.number(Number::parse("5").unwrap(), 9, Form::Scientific);
        interp.stem_set(b"A.", b"1", five);

        let b1 = interp.stem_get(b"B.", b"1").0;
        assert_eq!(&*interp.to_text(b1), b"5");
        let b7 = interp.stem_get(b"B.", b"7").0;
        assert_eq!(&*interp.to_text(b7), b"A.7");
    }

    #[test]
    fn an_unset_tail_piece_variable_still_derives_its_own_name_not_a_stem() {
        // i = (unset) ; v.i = 'x' ; say v.I -> x (measured: `I` is v.'s
        // upcased own spelling, exactly what an uninitialised simple
        // variable derives -- never a stem object, since a tail piece is
        // always a plain variable, `compound_parts`/`Tail::Variable`'s own
        // contract). `read_by_name` -- not `read_stem` -- is what
        // `tail_key` calls for a piece, and this pins that it still takes
        // the plain, non-vivifying path after F4 split the two functions
        // apart: a mutation that merged them back into one auto-vivifying
        // function would make `i`'s unset read allocate a `Body::Stem`
        // named `I` instead of deriving the text `I`, which this test
        // cannot observe going wrong through rendering alone -- so it
        // additionally checks that no slot was bound for `I` at all.
        let mut interp = Interp::new();
        let (program, id) = compound_id(&mut interp, b"say v.i");

        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = crate::planned_code(&program, &plan);
        let key = interp
            .tail_key(&code, id)
            .expect("the tail pieces are strings");
        assert_eq!(key, b"I");

        let x = interp.text(b"x");
        interp.stem_set(b"V.", &key, x);
        let v_i = interp.stem_get(b"V.", b"I").0;
        assert_eq!(&*interp.to_text(v_i), b"x");

        let i_slot = interp.slot_of(b"I");
        let frame = interp.activation().frame;
        assert!(
            interp.roots.frame_slot(frame, i_slot).is_none(),
            "an unset tail piece must not bind anything into its own slot"
        );
    }

    #[test]
    fn tail_keys_are_verbatim_and_case_sensitive() {
        // i = 'abc' ; v.i = 'val' ; say v.i v.ABC -> val V.ABC
        let mut interp = Interp::new();
        let (program, id) = compound_id(&mut interp, b"say v.i");
        let abc = interp.text(b"abc");
        let i_slot = interp.slot_of(b"I");
        let frame = interp.activation().frame;
        interp.roots.set_frame_slot(frame, i_slot, abc);

        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = crate::planned_code(&program, &plan);
        let key = interp
            .tail_key(&code, id)
            .expect("the tail pieces are strings");
        assert_eq!(key, b"abc");

        let val = interp.text(b"val");
        interp.stem_set(b"V.", &key, val);

        let v_i = interp.stem_get(b"V.", &key).0;
        assert_eq!(&*interp.to_text(v_i), b"val");
        // The literal piece "ABC" stands for itself, verbatim -- a
        // different key from the resolved "abc", so this is genuinely a
        // different tail, not a case-insensitive hit on the same one.
        let v_abc = interp.stem_get(b"V.", b"ABC").0;
        assert_eq!(&*interp.to_text(v_abc), b"V.ABC");
    }

    #[test]
    fn a_multi_level_tail_joins_its_pieces_with_a_period() {
        // i = 1 ; j = 2 ; a.i.j = 'deep' ; say a.1.2 -> deep
        let mut interp = Interp::new();
        let (program, id) = compound_id(&mut interp, b"say a.i.j");

        let one = interp.text(b"1");
        let two = interp.text(b"2");
        let frame = interp.activation().frame;
        let i_slot = interp.slot_of(b"I");
        interp.roots.set_frame_slot(frame, i_slot, one);
        let j_slot = interp.slot_of(b"J");
        interp.roots.set_frame_slot(frame, j_slot, two);

        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = crate::planned_code(&program, &plan);
        let key = interp
            .tail_key(&code, id)
            .expect("the tail pieces are strings");
        assert_eq!(key, b"1.2");

        let deep = interp.text(b"deep");
        interp.stem_set(b"A.", &key, deep);

        // The discriminating check: `a.1.2` (a literal key, resolved through
        // the SAME `tail_key` machinery on a second compound expression)
        // must land on the identical key `a.i.j` did, not a tuple of pieces.
        // Parsed only, not activated: activating a second program would push
        // a second frame, shadowing the one `i`/`j` were just bound in.
        let (program2, id2) = parse_compound(b"say a.1.2");
        let plan2 = Plan::build(
            &program2.main,
            &program2.symbols,
            Some(&program2.source),
            BodyKind::Plain,
        );
        let code2 = crate::planned_code(&program2, &plan2);
        let key2 = interp
            .tail_key(&code2, id2)
            .expect("the tail pieces are strings");
        assert_eq!(key2, key);

        let value = interp.stem_get(b"A.", &key2).0;
        assert_eq!(&*interp.to_text(value), b"deep");
    }

    #[test]
    fn a_tail_on_a_completely_untouched_stem_derives_its_name() {
        // say never_touched.5 -> NEVER_TOUCHED.5
        let mut interp = Interp::new();
        activated(&mut interp);
        let value = interp.stem_get(b"NEVER_TOUCHED.", b"5").0;
        assert_eq!(&*interp.to_text(value), b"NEVER_TOUCHED.5");
    }
}
