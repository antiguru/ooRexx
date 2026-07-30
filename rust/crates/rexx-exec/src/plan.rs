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

//! The per-body resolution plan (D16): a name-to-slot table built by one
//! upfront walk over a body's AST at first execution, and cached on
//! `Interp` rather than on the body itself (an `Rc<Program>` gives shared
//! immutable access, so nothing could be written into a `CodeBody` reached
//! through one).
//!
//! `Plan`, `BodyKey` and `ProgramId` here, and the cache lookup
//! (`Interp::plan_for`), a fragment's own resolution
//! (`Interp::fragment_plan`) and the full name resolution order
//! (`Interp::slot_of`) all moved here from Task 3's spike, which built this
//! shape and proved why `extra` (on `Activation`, `activation.rs`) is not
//! optional: the plan is an `Rc`, shared and immutable, built by a pass
//! that never saw a name introduced at run time -- and such names exist in
//! 4a. `DROP (v)` names its target at run time, and an interpreted
//! fragment's bindings are visible to the enclosing body's own later
//! clauses (measured: `interpret "newvar = 7"` then `say newvar + 1`
//! prints 8).

use crate::Interp;
use rexx_parse::{CodeBody, Expr, ExprKind, Fragment, InstructionKind, SymbolId, SymbolTable};
use std::collections::HashMap;
use std::rc::Rc;

/// A loaded program's identity.
///
/// A small integer the loader hands out, never a pointer: D16 requires that
/// the plan cache's key cannot be reused by a different program, and an
/// address can be, once an `Rc` drops and the allocator reuses the block.
/// `Interp::programs` holds an `Rc` for every id it has issued, so an id
/// outlives every plan keyed against it by construction.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ProgramId(pub(crate) usize);

/// Which code body of which loaded program a cached plan belongs to (D16).
///
/// There is deliberately **no fragment arm**, and that is a finding rather
/// than an omission. D16 says a fragment's plan is keyed by `(enclosing body,
/// fragment id)`, but a fragment is re-parsed on every execution of its
/// `INTERPRET` and its text can differ per iteration, so a "fragment id" can
/// only be a counter handed out per parse. Every lookup against such a key
/// misses and every insert stays forever, so `do 1000000; interpret s; end`
/// would accumulate a million plans that are each read zero times. The
/// durable part of a fragment's resolution is not its plan but the
/// name-to-slot bindings it adds to the enclosing activation, and those live
/// on `Activation::extra`. So a fragment plan is built, used, and dropped with
/// the fragment. See `Interp::fragment_plan`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct BodyKey {
    pub(crate) program: ProgramId,
    /// `None` is the program's main body. 4b is the first to need
    /// `Some(index)` for `directives[index]`'s body.
    pub(crate) directive: Option<usize>,
}

/// One body's variable-resolution plan, built by one upfront pass at first
/// execution (D16).
///
/// Two views of the same assignment. `names` is the map D16 specifies, keyed
/// by upcased name, and it is what a name resolved at *run time* goes through:
/// `DROP (v)`, and a fragment's names. `by_symbol` is what evaluation goes
/// through, so the hot path is a lookup by the id the AST already carries
/// rather than a byte-string hash.
///
/// `by_symbol` is a `HashMap` where D16's shape wants an array index.
/// `SymbolId` is a newtype over a private `u32` with no accessor, so nothing
/// outside `rexx-parse` can turn one into a `Vec` index directly -- though
/// `SymbolTable::intern`/`name` (`token.rs`) already use that same `u32` as a
/// dense, table-local, zero-based index internally
/// (`SymbolId(u32::try_from(self.names.len())...)`, `self.names[id.0 as
/// usize]`), so exposing it as `SymbolId::index()` would cost nothing new.
/// Recorded as an open `rexx-parse` amendment rather than made here: it
/// changes a crate this task does not own, and variable lookup is
/// 8.1%/32.2% of runtime (the realistic and stem-heavy benchmarks), so the
/// choice is worth making deliberately rather than as a side effect of this
/// task. Kept as a hash for now, pending that decision.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    pub(crate) names: HashMap<Box<[u8]>, usize>,
    pub(crate) by_symbol: HashMap<SymbolId, usize>,
}

impl Plan {
    /// Walks `body` once and returns a finished table (D16: "built by one
    /// upfront pass", not populated lazily one name at a time).
    pub(crate) fn build(body: &CodeBody, symbols: &SymbolTable) -> Plan {
        let mut plan = Plan::default();
        for instruction in &body.instructions {
            match &instruction.kind {
                InstructionKind::Assignment { target, value } => {
                    plan.note(target, symbols);
                    plan.note(value, symbols);
                }
                InstructionKind::Say {
                    expression: Some(expression),
                } => plan.note(expression, symbols),
                InstructionKind::Interpret { expression } => plan.note(expression, symbols),
                // Every other instruction fails loudly before evaluation ever
                // reaches it, so a variable inside one can never be read and
                // needs no slot. Task 6 makes this pass exhaustive over
                // `InstructionKind`, which is the point at which the omission
                // would start to matter.
                _ => {}
            }
        }
        plan
    }

    /// Assigns slots to every variable `expr` names, in source order.
    ///
    /// Recursive, and that recursion is on the interpreter thread's stack
    /// budget alongside `eval`'s: a left-deep 100,000-term expression is
    /// walked here as deeply as it is later evaluated.
    fn note(&mut self, expr: &Expr, symbols: &SymbolTable) {
        match &expr.kind {
            ExprKind::Variable(id) => self.bind(*id, symbols.name(*id).as_bytes()),
            ExprKind::Prefix { operand, .. } => self.note(operand, symbols),
            ExprKind::Binary { left, right, .. } => {
                self.note(left, symbols);
                self.note(right, symbols);
            }
            // A literal and a constant name no variable; every remaining form
            // fails loudly in `eval` before its names could be read. Task 6
            // covers `Stem`, `Compound` and the rest, where D16's rule that a
            // tail piece lands on the *same* slot as a same-named variable is
            // what `names` exists for.
            _ => {}
        }
    }

    /// Binds `name` to a slot, and `id` to the same one.
    ///
    /// Both views are updated together because they are one assignment seen
    /// two ways: a second symbol spelling the same name must land on the slot
    /// the first one got, which is why the slot number comes from `names` and
    /// never from `by_symbol`'s length.
    fn bind(&mut self, id: SymbolId, name: &[u8]) {
        let next = self.names.len();
        let slot = *self.names.entry(name.into()).or_insert(next);
        self.by_symbol.insert(id, slot);
    }

    pub(crate) fn len(&self) -> usize {
        self.names.len()
    }

    /// Looks up `name` in this plan alone: the first of the two steps D16's
    /// own resolution order names, `plan.slot_of(name).or_else(|| extra.get(name))`.
    /// Consults neither `extra` nor growth -- `Interp::slot_of` is the full
    /// three-source resolution this is one third of.
    pub(crate) fn slot_of(&self, name: &[u8]) -> Option<usize> {
        self.names.get(name).copied()
    }
}

impl Interp {
    /// The plan for one body, from the cache or built and cached (D16:
    /// "cached on `Interp`, not on the body", because an `Rc<Program>` gives
    /// shared immutable access and nothing can be written into a `CodeBody`
    /// reached through one).
    pub(crate) fn plan_for(
        &mut self,
        key: BodyKey,
        body: &CodeBody,
        symbols: &SymbolTable,
    ) -> Rc<Plan> {
        if let Some(plan) = self.plans.get(&key) {
            return Rc::clone(plan);
        }
        let plan = Rc::new(Plan::build(body, symbols));
        self.plans.insert(key, Rc::clone(&plan));
        plan
    }

    /// The slot `name` resolves to in the current frame, allocating one if it
    /// resolves to none.
    ///
    /// Three sources in order, and the third is the one D16 leaves out. The
    /// plan's name map is the upfront pass's answer. `extra` is every binding
    /// made since, which is where a fragment's new names and `DROP (v)`'s
    /// run-time target land. Growth is what happens when neither has it:
    /// `RootSet::grow_slots` extends the frame, and the name is recorded
    /// **here**, because the plan is an `Rc` and cannot be extended.
    pub(crate) fn slot_of(&mut self, name: &[u8]) -> usize {
        let activation = self.activation();
        if let Some(slot) = activation.plan.slot_of(name) {
            return slot;
        }
        if let Some(slot) = activation.extra.get(name) {
            return *slot;
        }
        let frame = activation.frame;
        let slot = self.roots.grow_slots(frame);
        self.activation_mut().extra.insert(name.into(), slot);
        slot
    }

    /// Resolves a fragment's own `SymbolId`s to slots in the **enclosing**
    /// frame.
    ///
    /// This is D16's "its plan is built against the enclosing plan's name map"
    /// and it goes through `slot_of`, which is that name map plus the two
    /// things D16 does not mention: the activation's `extra` bindings, and
    /// growth for a name nobody has bound yet. A fragment's ids are its own,
    /// and `parse_interpret` builds a fresh `SymbolTable` every call, so id 7
    /// in the fragment and id 7 in the program name unrelated symbols -- the
    /// join has to be through the text, `fragment.symbols.name(id)`, and this
    /// is the only place that matters.
    ///
    /// The result is returned rather than cached, for the reason `BodyKey`
    /// gives.
    pub(crate) fn fragment_plan(&mut self, fragment: &Fragment) -> HashMap<SymbolId, usize> {
        // The same upfront pass, run against the fragment's own body, which
        // numbers its names 0..n in walk order. Those numbers are local to the
        // fragment and mean nothing to the enclosing frame; the loop below is
        // what translates them.
        let local = Plan::build(&fragment.body, &fragment.symbols);

        // Walk order, recovered from the local numbering rather than from
        // iterating the map, because a `HashMap`'s order varies run to run and
        // the enclosing frame's slots would then be allocated in a different
        // order each time. Nothing observable depends on that order today,
        // which is the reason to fix it now rather than after something does.
        let mut by_local: Vec<&[u8]> = vec![b""; local.len()];
        for (name, slot) in &local.names {
            by_local[*slot] = name;
        }
        let enclosing: Vec<usize> = by_local.iter().map(|name| self.slot_of(name)).collect();

        local
            .by_symbol
            .iter()
            .map(|(id, local_slot)| (*id, enclosing[*local_slot]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Code;
    use rexx_parse::{Program, parse_interpret, parse_program};

    /// Pushes a fresh top-level activation for `program`, the same setup
    /// `Interp::run` does, so these tests can drive `slot_of`/`Plan` through
    /// a live activation without running the whole instruction loop.
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
            .push(crate::Activation::new(Rc::clone(&program), plan, frame));
        program
    }

    #[test]
    fn a_tail_piece_and_a_plain_variable_share_one_slot() {
        // b = 2 ; say a.b -> A.2 ; a.2 = 'hit' ; say a.b -> hit
        // An implementer who gives tail pieces their own slots gets A.B.
        let mut interp = Interp::new(false);
        let program = parse_program(b"say a.b".to_vec()).expect("test program parses");
        let id = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => match expr.kind {
                ExprKind::Compound(id) => id,
                ref other => panic!("expected a compound expression, got {other:?}"),
            },
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        let program = activate(&mut interp, program);

        let two = interp.text(b"2");
        let b_slot = interp.slot_of(b"B");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, b_slot, two);

        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        let key = interp.tail_key(&code, id);
        assert_eq!(key, b"2");
        // `A.` shares its "2" slot with the plain variable `B`'s value,
        // through `a.2`'s own key, which is exactly what `key` resolved to.
        let a2_slot = interp.slot_of(b"A.2");
        assert_ne!(
            a2_slot, b_slot,
            "A.2 is a different variable name from B, so a different slot -- \
             what must share a slot is the tail KEY '2' with the variable B's VALUE, \
             not the names themselves"
        );

        let uninit = interp.stem_get(b"A.", &key);
        assert_eq!(&*interp.to_text(uninit), b"A.2");

        let hit = interp.text(b"hit");
        interp.stem_set(b"A.", &key, hit);
        let after = interp.stem_get(b"A.", &key);
        assert_eq!(&*interp.to_text(after), b"hit");
    }

    #[test]
    fn a_runtime_name_grows_the_frame() {
        // v = 'X' ; x = 1 ; drop (v) ; say x  ->  X
        // X may not appear in the body at all, so the plan cannot have a
        // slot for it: this program never mentions X in its own text.
        let mut interp = Interp::new(false);
        let program = parse_program(b"nop".to_vec()).expect("test program parses");
        activate(&mut interp, program);

        assert_eq!(
            interp.activation().plan.len(),
            0,
            "a body with no variables at all has an empty plan"
        );

        let one = interp.number(
            rexx_num::Number::parse("1").unwrap(),
            9,
            rexx_num::Form::Scientific,
        );
        let x_slot = interp.slot_of(b"X");
        let frame = interp.activation().frame;
        interp.roots.set_slot(frame, x_slot, one);

        // `drop (v)` resolves its target *by the current value of v*, not by
        // the literal text "v" -- here that value is "X", so this names the
        // same slot `x_slot` does, exactly like `DROP (v)` would at run time.
        let dynamic_slot = interp.slot_of(b"X");
        assert_eq!(dynamic_slot, x_slot);
        assert!(
            interp.activation().extra.contains_key(b"X".as_slice()),
            "a name the plan never saw must grow into extra, not silently \
             miss or panic"
        );
    }

    #[test]
    fn names_are_keyed_upcased_but_tail_values_are_not() {
        // The two rules live in different decision blocks (D16 vs D15a) and
        // are easy to swap. A NAME is upcased before a `SymbolId` even
        // exists for it -- the *tokenizer* does that (`SymbolTable::intern`),
        // not `Plan` or `slot_of`, which never see a lowercase spelling to
        // begin with: `v.i`, however the source writes it, interns as
        // "V.I", and `compound_parts` decomposes the piece as "I", already
        // upcase. A tail VALUE is different: whatever the piece variable's
        // current *value* renders as, verbatim and case-sensitively.
        let mut interp = Interp::new(false);
        let program = parse_program(b"say v.i".to_vec()).expect("test program parses");
        let id = match &program.main.instructions[0].kind {
            InstructionKind::Say {
                expression: Some(expr),
            } => match expr.kind {
                ExprKind::Compound(id) => id,
                ref other => panic!("expected a compound expression, got {other:?}"),
            },
            other => panic!("expected a SAY with an expression, got {other:?}"),
        };
        // The compound's own interned spelling is already upcased regardless
        // of how the source wrote it -- the fact D16's "keyed by upcased
        // name" rests on, checked here rather than assumed.
        assert_eq!(program.symbols.name(id), "V.I");

        let program = activate(&mut interp, program);
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
        // The tail VALUE "abc" survives verbatim, lowercase and all -- not
        // upcased to "ABC", which is the distinct rule D15a states.
        assert_eq!(key, b"abc");
    }

    #[test]
    fn a_fragments_plan_resolves_against_the_enclosing_frame() {
        // interpret "newvar = 7" ; say newvar + 1  ->  8 (measured on the
        // oracle). A fragment's plan is never cached (BodyKey has no
        // fragment arm) and its bindings land in the enclosing frame via
        // `extra`, not in a frame of its own.
        let mut interp = Interp::new(false);
        let program = parse_program(b"nop".to_vec()).expect("test program parses");
        activate(&mut interp, program);

        let fragment = parse_interpret(b"newvar = 7".to_vec()).expect("fragment parses");
        let slots = interp.fragment_plan(&fragment);
        assert_eq!(slots.len(), 1, "the fragment names exactly one variable");

        let enclosing_slot = interp.slot_of(b"NEWVAR");
        let (_id, fragment_slot) = slots.iter().next().expect("one entry");
        assert_eq!(
            *fragment_slot, enclosing_slot,
            "the fragment's own id must resolve to the SAME slot the \
             enclosing body would use for the same name"
        );
    }
}
