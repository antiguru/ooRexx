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
//!
//! 1. **A dropped tail is a tombstone**, present in the map and mapped to
//!    `None`, which does *not* fall back to the stem's default; an absent
//!    key does. `u.='d'; u.1='one'; drop u.1; say u.1` is `U.1`, the derived
//!    name, not `d`.
//! 2. **`stem. = expr` and `drop stem.` replace the whole `Stem` object and
//!    rebind the variable; a tail write mutates the existing one.**
//!    `a.=1; b.=a.; a.1=2; say b.1` is `2`: `b.` shares the very object `a.`
//!    was pointing at, so a tail mutation through either name is visible
//!    through both. Measured further: after that, `a. = 9` replaces `a.`'s
//!    object with a fresh one, and `b.` (still holding the old one) is
//!    unaffected -- `say b.` and `say b.1` still answer `1` and `2`.
//! 3. **A stem carries its own name**, because a bare stem read returns the
//!    object itself (or the derived name if unset), never a copy of the
//!    read site's spelling, and that object can outlive the variable that
//!    first produced it: `q.1='x'; y=q.; say y` is `Q.`, not the tail value
//!    and not anything derived from `y` -- by the time `y` is read, `q.` is
//!    gone from context and only the object's own `name` field still says
//!    "Q.".
//! 4. **A tail key is the resolved piece values, verbatim and case-
//!    sensitively, joined by `.`.** `i='abc'; v.i='val'; say v.i v.ABC` is
//!    `val V.ABC`: `I` resolves to `abc`, which is not upcased, while `ABC`
//!    written literally stands for itself.
//!
//! **What needs no code here at all.** Reading a *bare* stem
//! (`ExprKind::Stem`) is not a new operation: it goes through the exact same
//! slot read every variable uses, unset or not. Unset, it derives its own
//! name exactly as an uninitialised simple variable does -- no `Body::Stem`
//! is ever allocated for that case (measured: `say never_touched.5` needs no
//! stem object to answer `NEVER_TOUCHED.5`, and `drop x.` leaves `x.`
//! behaving exactly as if it had never been touched at all, so there is
//! nothing to distinguish). Set, the slot holds a `Body::Stem`, and
//! *rendering* it is the one thing this module adds to `value.rs`'s
//! `to_text`: the object's `default` if `Some`, else its own `name`.

use crate::{Code, Interp};
use rexx_core::{BehaviourId, Body, Decoded, ObjRef};
use rexx_parse::{SymbolId, Tail, compound_parts};
use std::collections::HashMap;

/// Names a `Body` variant without printing it.
///
/// The three `unreachable!` sites below want to say *what* was found in a
/// stem-named slot, and `{:?}` on the value answers that at unbounded length:
/// a `Body::Stem` formats its entire tails map, so a panic message could carry
/// a whole variable pool. Same defect the not-implemented messages had, where
/// one expression produced 373,332 bytes, and the same fix.
///
/// Exhaustive with no wildcard arm on purpose, mirroring `Body::trace` and
/// `form_name`: a new `Body` variant is a compile error here rather than a
/// silent "unknown".
fn body_variant_name(body: &Body) -> &'static str {
    match body {
        Body::Text { .. } => "Body::Text",
        Body::Num { .. } => "Body::Num",
        Body::Stem { .. } => "Body::Stem",
        Body::Array(_) => "Body::Array",
        Body::Instance(_) => "Body::Instance",
        Body::WeakRef(_) => "Body::WeakRef",
    }
}

impl Interp {
    /// Resolves a compound's tail pieces into the one key its tails map is
    /// keyed by (D15a): each piece verbatim and case-sensitively, joined by
    /// `.`. `Tail::Constant` stands for itself; `Tail::Variable` is a plain
    /// variable name whose *current value* supplies the piece, read through
    /// the ordinary variable path (deriving its own name if unset, the same
    /// as any read) and rendered with `to_text`.
    ///
    /// `id` is the *whole* compound's `SymbolId` (`ExprKind::Compound`'s
    /// own id, whose name is the full dotted spelling) -- there is no
    /// separate id for a piece, `compound_parts` only ever hands back a
    /// borrowed slice of that same interned name, which is why this takes
    /// `code: &Code<'_>` rather than the brief's `&Plan`/`frame`: turning
    /// `id` into text needs `code.symbols`, and a piece's value needs the
    /// ordinary slot machinery `self` already carries, neither of which a
    /// bare `Plan`/`SlotFrame` pair reaches on its own.
    pub(crate) fn tail_key(&mut self, code: &Code<'_>, id: SymbolId) -> Vec<u8> {
        let (_stem, tails) = compound_parts(code.symbols.name(id));
        let mut key = Vec::new();
        for (index, tail) in tails.iter().enumerate() {
            if index > 0 {
                key.push(b'.');
            }
            match tail {
                Tail::Constant(text) => key.extend_from_slice(text.as_bytes()),
                Tail::Variable(name) => {
                    let value = self.read_by_name(name.as_bytes());
                    key.extend_from_slice(&self.to_text(value));
                }
            }
        }
        key
    }

    /// Reads a variable by name alone, the same slot machinery `read` uses
    /// but with no `SymbolId` to try first: a tail piece from
    /// `compound_parts` is a borrowed `&str`, not a token, so there is no id
    /// for it (`ExprKind::Compound`'s doc comment on `ast.rs`). Unset
    /// derives the name's own (already upcased) spelling, same as any
    /// uninitialised read -- which is also exactly what a *bare* stem read
    /// needs (`ExprKind::Stem`'s id already carries its own trailing
    /// period), so this one function serves both a tail piece's value and a
    /// stem variable's raw, unconverted one.
    pub(crate) fn read_by_name(&mut self, name: &[u8]) -> ObjRef {
        let slot = self.slot_of(name);
        let frame = self.activation().frame;
        match self.roots.slot(frame, slot) {
            Some(value) => value,
            None => self.text(name),
        }
    }

    /// Reads a tail: `stem_name` is the **read site's** own name (used only
    /// to find the slot), including its trailing period; `key` is
    /// `tail_key`'s output.
    ///
    /// Unset stem -> the whole compound is uninitialised, derived name from
    /// `stem_name` (there is no object to ask), no `Body::Stem` needed
    /// (measured: `say never_touched.5` is `NEVER_TOUCHED.5`). Set: a
    /// tombstone (`Some(None)`) or an absent key with no default both
    /// derive the name -- but from the **object's own** `name` field, not
    /// `stem_name`, once an object exists. A tombstone does **not** fall
    /// back to the default, which is the rule an obvious "just check the
    /// map" implementation misses (D15a's `u.1`/`u.2` pair).
    ///
    /// **`stem_name` does two different jobs, and once aliasing exists they
    /// want different names.** Finding the slot needs the read site's
    /// spelling, always. Deriving an unresolved tail's name needs the
    /// object's own, the same rule `to_text` already applies to a bare
    /// stem read (D15a: "a stem carries its own name"). An earlier version
    /// used `stem_name` for both, which is right exactly when the two agree
    /// and wrong the moment they do not. Measured, and the aliased case no
    /// existing test reached because every read in the aliasing test
    /// *resolved* (through a shared default or a written tail), while
    /// `derived_tail_name` is reachable only when a tail does *not*
    /// resolve -- so the aliasing test and the tombstone test never met:
    ///
    /// ```text
    /// a.1='x'; b.=a.;             say b.2  ->  A.2   (not B.2)
    /// c.1='x'; d.=c.; drop d.1;   say d.1  ->  C.1   (not D.1)
    /// g.1='x'; h.=g.; k.=h.;      say k.9  ->  G.9   (two hops, still G)
    /// m.1='x'; n.2='y'; m.=n.;    say m.1  ->  N.1   (m.'s own object discarded)
    /// ```
    pub(crate) fn stem_get(&mut self, stem_name: &[u8], key: &[u8]) -> ObjRef {
        let slot = self.slot_of(stem_name);
        let frame = self.activation().frame;
        let stem_value = match self.roots.slot(frame, slot) {
            Some(v) => v,
            // No object at all: the read site's own spelling is all there
            // is to derive from.
            None => return self.derived_tail_name(stem_name, key),
        };

        // `resolved` and `object_name` are both computed, fully, before any
        // further `self` call, so the borrow on `self.heap` below never has
        // to overlap one. `object_name` is cloned (a small, boxed slice)
        // rather than borrowed, for the same reason.
        let (resolved, object_name) = {
            let object = self.heap.get(stem_value).expect("a live value");
            let Body::Stem {
                name,
                default,
                tails,
            } = &object.body
            else {
                unreachable!(
                    "a stem-named slot holds only Body::Stem, got {}",
                    body_variant_name(&object.body)
                );
            };
            let resolved = match tails.get(key) {
                Some(Some(value)) => Some(*value),
                Some(None) => None, // the tombstone: absent from the default too
                None => *default,   // an untouched tail falls back to the default
            };
            (resolved, name.clone())
        };

        match resolved {
            Some(value) => value,
            // An object exists but this key does not resolve: derive from
            // the OBJECT's own name, not the read site's -- see this
            // function's doc comment for why the two can differ.
            None => self.derived_tail_name(&object_name, key),
        }
    }

    /// Writes a tail: mutates the stem's existing object in place, or
    /// auto-vivifies one (`default: None`) if this is the first write the
    /// stem has ever seen (D15a: "a tail assignment mutates", and measured,
    /// `q.1='x'` with `q.` never itself assigned still leaves `q.2`
    /// deriving its own name rather than falling back to anything).
    pub(crate) fn stem_set(&mut self, stem_name: &[u8], key: &[u8], value: ObjRef) {
        let slot = self.slot_of(stem_name);
        let frame = self.activation().frame;
        match self.roots.slot(frame, slot) {
            Some(stem_value) => {
                let object = self.heap.get_mut(stem_value).expect("a live value");
                let Body::Stem { tails, .. } = &mut object.body else {
                    unreachable!(
                        "a stem-named slot holds only Body::Stem, got {}",
                        body_variant_name(&object.body)
                    );
                };
                tails.insert(key.to_vec(), Some(value));
            }
            None => {
                let mut tails = HashMap::new();
                tails.insert(key.to_vec(), Some(value));
                let stem = self.alloc_with(
                    BehaviourId::STEM,
                    Body::Stem {
                        name: stem_name.into(),
                        default: None,
                        tails,
                    },
                );
                self.roots.set_slot(frame, slot, stem);
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
        let slot = self.slot_of(stem_name);
        let frame = self.activation().frame;
        if let Some(stem_value) = self.roots.slot(frame, slot) {
            let object = self.heap.get_mut(stem_value).expect("a live value");
            let Body::Stem { tails, .. } = &mut object.body else {
                unreachable!(
                    "a stem-named slot holds only Body::Stem, got {}",
                    body_variant_name(&object.body)
                );
            };
            tails.insert(key.to_vec(), None);
        }
    }

    /// `stem. = expr`: replaces the whole object and rebinds the variable
    /// (D15a).
    ///
    /// **Except when `value` is already a `Body::Stem`,** which is not a
    /// special case bolted on but what "replace and rebind" has to mean when
    /// the expression on the right is itself a bare stem read: assignment
    /// never clones an object in Rexx, so `b. = a.` makes `b.` reference the
    /// very object `a.` does, not a new object whose default happens to be
    /// `a.`'s. Measured, and the only hypothesis that survives it: `a.=1;
    /// b.=a.; a.1=2; say b.1` is `2` (a tail written through `a.` is visible
    /// through `b.`, which only holds if they are one object), and `a. = 9`
    /// afterward -- rebinding `a.`'s *own* slot to a fresh object -- leaves
    /// `b.` answering `1` and `b.1` answering `2`, unchanged, because `b.`
    /// was never holding a reference to `a.`'s *slot*, only to the object
    /// that used to be there. A "wrap the value as my new default" model
    /// would need `stem_get` to chase through a nested default to reproduce
    /// `b.1 -> 2`, which is a rule D15a never states and this measurement
    /// does not require: sharing the object makes it fall out for free.
    /// Assigning anything that is not already a stem still wraps it as this
    /// stem's new default, same as `a. = 1`, `w. = 'wd'`.
    pub(crate) fn stem_assign(&mut self, stem_name: &[u8], value: ObjRef) {
        if self.is_stem(value) {
            let slot = self.slot_of(stem_name);
            let frame = self.activation().frame;
            self.roots.set_slot(frame, slot, value);
        } else {
            self.replace_stem(stem_name, Some(value));
        }
    }

    /// `drop stem.`: replaces the whole object with a fresh, empty one
    /// (`default: None`, no tails) and rebinds the variable -- the same
    /// "replace and rebind" `stem_assign` uses, with nothing to wrap.
    ///
    /// Not `RootSet::clear_slot`, even though one exists (added, after this
    /// was first written, for plain `DROP` on a simple variable, whose read
    /// path has to tell "unset" apart from every other value for 4b's
    /// `NOVALUE`). A stem's slot is not "empty or not" the way a simple
    /// variable's is: replacing the object is the literal reading of
    /// D15a's own wording, and it is what makes `stem_assign`'s wrap branch
    /// and this function one shared operation (`replace_stem`) rather than
    /// two independently-written ones that happen to agree today.
    ///
    /// Measured, and it is the only case this task found where "dropped"
    /// and "never touched" turn out to be indistinguishable rather than
    /// merely similar: `x.='d'; x.1='one'; drop x.; say x.1; say x.` gives
    /// `X.1` then `X.`, exactly what a stem named `X.` that had never been
    /// touched at all would give.
    ///
    /// **This does not generalise to tails, and reading it that way is the
    /// trap.** A dropped *tail* (`stem_drop_tail`) is a tombstone: present
    /// in the map, and it does *not* fall back to the default, which an
    /// absent key does. Here, one level up, "dropped" and "absent" collapse
    /// into the very same thing. The two rules sit one level apart -- a
    /// whole stem versus one of its tails -- precisely because they
    /// disagree.
    pub(crate) fn stem_drop(&mut self, stem_name: &[u8]) {
        self.replace_stem(stem_name, None);
    }

    /// The shared half of `stem_assign`'s "wrap" branch and `stem_drop`:
    /// build a fresh `Body::Stem` with the given default and rebind the
    /// variable to it, leaving any old object exactly where aliases into it
    /// already point (D15a's `r.`/`u` and `s.`/`t` transcripts, which need
    /// the *old* object left untouched rather than mutated).
    fn replace_stem(&mut self, stem_name: &[u8], default: Option<ObjRef>) {
        let slot = self.slot_of(stem_name);
        let frame = self.activation().frame;
        let stem = self.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: stem_name.into(),
                default,
                tails: HashMap::new(),
            },
        );
        self.roots.set_slot(frame, slot, stem);
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        interp
            .activations
            .push(Activation::new(Rc::clone(&program), plan, frame));
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
        let mut interp = Interp::new(false);
        activated(&mut interp);

        let d = interp.text(b"d");
        interp.stem_assign(b"U.", d);
        let one = interp.text(b"one");
        interp.stem_set(b"U.", b"1", one);
        interp.stem_drop_tail(b"U.", b"1");

        let u1 = interp.stem_get(b"U.", b"1");
        assert_eq!(&*interp.to_text(u1), b"U.1");
        let u2 = interp.stem_get(b"U.", b"2");
        assert_eq!(&*interp.to_text(u2), b"d");
    }

    #[test]
    fn bare_stem_assignment_shares_the_object_when_the_value_is_already_a_stem() {
        // a. = 1 ; b. = a. ; a.1 = 2 ; say b.1 -> 2
        // a. = 9 (afterward) ; say b. -> 1 ; say b.1 -> 2  (b. keeps the OLD object)
        let mut interp = Interp::new(false);
        activated(&mut interp);

        let one = interp.number(Number::parse("1").unwrap(), 9, Form::Scientific);
        interp.stem_assign(b"A.", one);
        let a_value = interp.read_by_name(b"A.");
        interp.stem_assign(b"B.", a_value);

        let two = interp.number(Number::parse("2").unwrap(), 9, Form::Scientific);
        interp.stem_set(b"A.", b"1", two);

        let b1 = interp.stem_get(b"B.", b"1");
        assert_eq!(&*interp.to_text(b1), b"2");

        let nine = interp.number(Number::parse("9").unwrap(), 9, Form::Scientific);
        interp.stem_assign(b"A.", nine);

        let b_bare = interp.read_by_name(b"B.");
        assert_eq!(&*interp.to_text(b_bare), b"1");
        let b1_again = interp.stem_get(b"B.", b"1");
        assert_eq!(&*interp.to_text(b1_again), b"2");
    }

    #[test]
    fn dropping_the_whole_stem_leaves_an_old_alias_intact() {
        // r. = 'rd' ; u = r. ; drop r. ; say u -> rd
        let mut interp = Interp::new(false);
        activated(&mut interp);

        let rd = interp.text(b"rd");
        interp.stem_assign(b"R.", rd);
        let r_value = interp.read_by_name(b"R.");
        // `u = r.`: an ordinary simple-variable assignment, aliasing whatever
        // `r.`'s slot currently holds -- no stem function involved, because
        // the target is not a stem.
        let u_slot = interp.slot_of(b"U");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, u_slot, r_value);

        interp.stem_drop(b"R.");

        let u_value = interp.roots.slot(frame, u_slot).expect("u was set");
        assert_eq!(&*interp.to_text(u_value), b"rd");
    }

    #[test]
    fn reassigning_the_whole_stem_leaves_an_old_alias_intact() {
        // s. = 'def' ; t = s. ; s. = 'other' ; say t -> def
        let mut interp = Interp::new(false);
        activated(&mut interp);

        let def = interp.text(b"def");
        interp.stem_assign(b"S.", def);
        let s_value = interp.read_by_name(b"S.");
        let t_slot = interp.slot_of(b"T");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, t_slot, s_value);

        let other = interp.text(b"other");
        interp.stem_assign(b"S.", other);

        let t_value = interp.roots.slot(frame, t_slot).expect("t was set");
        assert_eq!(&*interp.to_text(t_value), b"def");
    }

    #[test]
    fn an_untouched_stem_derives_its_own_name_with_the_period() {
        // say q. -> Q.
        let mut interp = Interp::new(false);
        activated(&mut interp);
        let q = interp.read_by_name(b"Q.");
        assert_eq!(&*interp.to_text(q), b"Q.");
    }

    #[test]
    fn tail_keys_are_verbatim_and_case_sensitive() {
        // i = 'abc' ; v.i = 'val' ; say v.i v.ABC -> val V.ABC
        let mut interp = Interp::new(false);
        let (program, id) = compound_id(&mut interp, b"say v.i");
        let abc = interp.text(b"abc");
        let i_slot = interp.slot_of(b"I");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, i_slot, abc);

        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        let key = interp.tail_key(&code, id);
        assert_eq!(key, b"abc");

        let val = interp.text(b"val");
        interp.stem_set(b"V.", &key, val);

        let v_i = interp.stem_get(b"V.", &key);
        assert_eq!(&*interp.to_text(v_i), b"val");
        // The literal piece "ABC" stands for itself, verbatim -- a
        // different key from the resolved "abc", so this is genuinely a
        // different tail, not a case-insensitive hit on the same one.
        let v_abc = interp.stem_get(b"V.", b"ABC");
        assert_eq!(&*interp.to_text(v_abc), b"V.ABC");
    }

    #[test]
    fn a_multi_level_tail_joins_its_pieces_with_a_period() {
        // i = 1 ; j = 2 ; a.i.j = 'deep' ; say a.1.2 -> deep
        let mut interp = Interp::new(false);
        let (program, id) = compound_id(&mut interp, b"say a.i.j");

        let one = interp.text(b"1");
        let two = interp.text(b"2");
        let frame = interp.activation().frame;
        let i_slot = interp.slot_of(b"I");
        interp.roots.set_slot(frame, i_slot, one);
        let j_slot = interp.slot_of(b"J");
        interp.roots.set_slot(frame, j_slot, two);

        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        let key = interp.tail_key(&code, id);
        assert_eq!(key, b"1.2");

        let deep = interp.text(b"deep");
        interp.stem_set(b"A.", &key, deep);

        // The discriminating check: `a.1.2` (a literal key, resolved through
        // the SAME `tail_key` machinery on a second compound expression)
        // must land on the identical key `a.i.j` did, not a tuple of pieces.
        // Parsed only, not activated: activating a second program would push
        // a second frame, shadowing the one `i`/`j` were just bound in.
        let (program2, id2) = parse_compound(b"say a.1.2");
        let code2 = Code {
            body: &program2.main,
            symbols: &program2.symbols,
            slots: &HashMap::new(),
        };
        let key2 = interp.tail_key(&code2, id2);
        assert_eq!(key2, key);

        let value = interp.stem_get(b"A.", &key2);
        assert_eq!(&*interp.to_text(value), b"deep");
    }

    #[test]
    fn a_tail_on_a_completely_untouched_stem_derives_its_name() {
        // say never_touched.5 -> NEVER_TOUCHED.5
        let mut interp = Interp::new(false);
        activated(&mut interp);
        let value = interp.stem_get(b"NEVER_TOUCHED.", b"5");
        assert_eq!(&*interp.to_text(value), b"NEVER_TOUCHED.5");
    }
}
