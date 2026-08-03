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
use rexx_parse::{
    Call, CodeBody, Expr, ExprKind, Fragment, InstructionKind, Loop, LoopKind, Parse, ParseSource,
    Redirection, Signal, SymbolId, SymbolTable, Tail, Trace, Use, VariableRef, compound_parts,
};
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
/// **That accessor has since landed** (`SymbolId::index()`, `180875a9`), so
/// switching `by_symbol` to a `Vec` indexed by it is now a decision this
/// crate could make, not one blocked on `rexx-parse`. Not made in this fix
/// round, which is scoped to making `note`/`build` exhaustive rather than
/// to the representation: variable lookup is 8.1%/32.2% of runtime (the
/// realistic and stem-heavy benchmarks), so trading a `HashMap` for a `Vec`
/// is worth its own measurement, not a side effect of an unrelated change.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    pub(crate) names: HashMap<Box<[u8]>, usize>,
    pub(crate) by_symbol: HashMap<SymbolId, usize>,
}

impl Plan {
    /// Walks `body` once and returns a finished table (D16: "built by one
    /// upfront pass", not populated lazily one name at a time).
    ///
    /// **Exhaustive over `InstructionKind`, with no catch-all arm.** The
    /// original `_ => {}` here (and `note`'s own, below) was the actual
    /// defect this fix closes, not a placeholder: a body containing a
    /// `Stem` or `Compound` produced an *empty* plan, so every one of its
    /// names went through `grow_slots` one at a time on first touch --
    /// precisely the lazy algorithm D16 exists to replace, and worst on the
    /// stem-heavy code D16's own 32.2% figure measures. Matching every
    /// variant explicitly, even the ones that contribute nothing, is what
    /// makes a future omission a compile error instead of a silent one.
    ///
    /// Registers every name an instruction's fields *could* name, not only
    /// the ones this phase's `eval`/`run` already executes: most kinds
    /// below still fail loudly today and gain real behaviour only in later
    /// tasks, but pre-registering a name costs nothing when it is never
    /// read (an unread slot is simply unread), and it means neither this
    /// function nor a later task has to remember to revisit `plan.rs` the
    /// day one of them stops failing loudly.
    pub(crate) fn build(body: &CodeBody, symbols: &SymbolTable) -> Plan {
        let mut plan = Plan::default();
        for instruction in &body.instructions {
            plan.note_instruction(&instruction.kind, symbols);
        }
        plan
    }

    /// Registers every name one instruction's own fields could read -- the
    /// exhaustive match `build`'s doc comment describes, factored out to
    /// its own function because the match itself, covering all
    /// thirty-nine `InstructionKind` variants, does not fit inside a loop
    /// body and stay readable.
    fn note_instruction(&mut self, kind: &InstructionKind, symbols: &SymbolTable) {
        match kind {
            InstructionKind::Assignment { target, value } => {
                self.note(target, symbols);
                self.note(value, symbols);
            }
            InstructionKind::Message { term, value } => {
                self.note(term, symbols);
                self.note_opt(value, symbols);
            }
            InstructionKind::Command { expression }
            | InstructionKind::Push { expression }
            | InstructionKind::Queue { expression }
            | InstructionKind::Say { expression }
            | InstructionKind::Return { expression }
            | InstructionKind::Exit { expression }
            | InstructionKind::Reply { expression }
            | InstructionKind::Numeric { expression, .. } => self.note_opt(expression, symbols),
            InstructionKind::Interpret { expression } | InstructionKind::Options { expression } => {
                self.note(expression, symbols);
            }
            InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) => {
                self.note_loop(loop_, symbols);
            }
            InstructionKind::If { condition, .. } | InstructionKind::When { condition, .. } => {
                self.note(condition, symbols);
            }
            InstructionKind::WhenCase { values, .. } => {
                for value in values {
                    self.note(value, symbols);
                }
            }
            InstructionKind::Select { case, .. } => self.note_opt(case, symbols),
            // No expression and no data variable: `Select`'s own `label`
            // and `Leave`/`Iterate`/`End`'s `name` are *block* labels
            // matched against `LEAVE`/`ITERATE`/`END`, never read through
            // `slot_of` -- the same distinction `note_loop` draws for a
            // `DO`/`LOOP`'s own `label`, below.
            InstructionKind::Label { .. }
            | InstructionKind::Then
            | InstructionKind::Else { .. }
            | InstructionKind::Otherwise
            | InstructionKind::Leave { .. }
            | InstructionKind::Iterate { .. }
            | InstructionKind::End { .. }
            | InstructionKind::Nop => {}
            InstructionKind::Drop { variables }
            | InstructionKind::Expose { variables }
            | InstructionKind::Procedure { variables } => {
                for variable in variables {
                    self.note_variable_ref(variable, symbols);
                }
            }
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => self.note_parse(parse, symbols),
            InstructionKind::Call(call) => self.note_call(call, symbols),
            InstructionKind::Signal(signal) => {
                if let Signal::Value(expr) = &**signal {
                    self.note(expr, symbols);
                }
                // `Label`/`Trap` name a label or a condition, neither a
                // data variable.
            }
            InstructionKind::Guard(guard) => self.note_opt(&guard.condition, symbols),
            InstructionKind::Forward(forward) => {
                self.note_opt(&forward.to, symbols);
                self.note_opt(&forward.message, symbols);
                self.note_opt(&forward.class, symbols);
                self.note_opt(&forward.arguments, symbols);
                if let Some(items) = &forward.array {
                    self.note_args(items, symbols);
                }
            }
            InstructionKind::Raise(raise) => {
                self.note_opt(&raise.rc, symbols);
                self.note_opt(&raise.description, symbols);
                self.note_opt(&raise.additional, symbols);
                if let Some(items) = &raise.array {
                    self.note_args(items, symbols);
                }
                if let Some(result) = &raise.result {
                    self.note_opt(&result.value, symbols);
                }
            }
            InstructionKind::Use(use_) => match &**use_ {
                Use::Arg { targets, .. } => {
                    for target in targets.iter().flatten() {
                        self.note(&target.target, symbols);
                        self.note_opt(&target.default, symbols);
                    }
                }
                Use::Local { variables } => {
                    for variable in variables {
                        self.note_variable_ref(variable, symbols);
                    }
                }
            },
            InstructionKind::Address(address) => {
                self.note_opt(&address.dynamic, symbols);
                self.note_opt(&address.command, symbols);
                if let Some(io) = &address.io {
                    for redirection in [&io.input, &io.output, &io.error] {
                        match redirection {
                            // `STEM name.`: the same single, complete,
                            // interned symbol a bare `ExprKind::Stem` read
                            // is, so it gets the same treatment `note`
                            // gives one, id and all.
                            Redirection::Stem(id) => self.bind(*id, symbols.name(*id).as_bytes()),
                            Redirection::Stream(expr) | Redirection::Using(expr) => {
                                self.note(expr, symbols);
                            }
                            Redirection::Default | Redirection::Normal => {}
                        }
                    }
                }
            }
            InstructionKind::Trace(trace) => {
                if let Trace::Value(expr) = trace {
                    self.note(expr, symbols);
                }
            }
        }
    }

    /// A `DO`/`LOOP` header's own names: `COUNTER`/control variables (bound,
    /// since each is one complete, interned symbol with its own id, the
    /// same treatment `note` gives a bare `Variable`) and every expression
    /// its kind and its trailing `WHILE`/`UNTIL` carry.
    ///
    /// `label` is deliberately not bound: a loop label names the block for
    /// `LEAVE`/`ITERATE`/`END`, not a data variable, exactly like `Select`'s
    /// own `label` in `note_instruction`.
    fn note_loop(&mut self, loop_: &Loop, symbols: &SymbolTable) {
        if let Some(counter) = loop_.counter {
            self.bind(counter, symbols.name(counter).as_bytes());
        }
        match &loop_.kind {
            LoopKind::Simple | LoopKind::Forever => {}
            LoopKind::Count(count) => self.note_opt(count, symbols),
            LoopKind::Controlled(controlled) => {
                self.bind(
                    controlled.control,
                    symbols.name(controlled.control).as_bytes(),
                );
                self.note(&controlled.initial, symbols);
                self.note_opt(&controlled.to, symbols);
                self.note_opt(&controlled.by, symbols);
                self.note_opt(&controlled.for_count, symbols);
            }
            LoopKind::Over {
                control,
                target,
                for_count,
            } => {
                self.bind(*control, symbols.name(*control).as_bytes());
                self.note(target, symbols);
                self.note_opt(for_count, symbols);
            }
            LoopKind::With {
                index,
                item,
                target,
                for_count,
            } => {
                if let Some(index) = index {
                    self.bind(*index, symbols.name(*index).as_bytes());
                }
                if let Some(item) = item {
                    self.bind(*item, symbols.name(*item).as_bytes());
                }
                self.note(target, symbols);
                self.note_opt(for_count, symbols);
            }
        }
        if let Some(conditional) = &loop_.conditional {
            self.note(&conditional.condition, symbols);
        }
    }

    /// A `PARSE`/`ARG`/`PULL` instruction's own names: `PARSE VAR name`'s
    /// source variable (read from, so bound the same way any read is), any
    /// expression a `VALUE` source or a trigger's pattern carries, and
    /// every template target -- already `Expr`s (`ast.rs`'s own doc: a
    /// dropped `.` placeholder aside, a target is a `Variable`/`Stem`/
    /// `Compound` node), so `note` alone is enough for them.
    fn note_parse(&mut self, parse: &Parse, symbols: &SymbolTable) {
        match &parse.source {
            ParseSource::Var(id) => self.bind(*id, symbols.name(*id).as_bytes()),
            ParseSource::Value(expr) => self.note_opt(expr, symbols),
            ParseSource::Arg
            | ParseSource::LineIn
            | ParseSource::Pull
            | ParseSource::Source
            | ParseSource::Version => {}
        }
        for trigger in parse.template.iter().flatten() {
            self.note_opt(&trigger.value, symbols);
            self.note_args(&trigger.targets, symbols);
        }
    }

    /// A `CALL`'s own names: a dynamic target and every argument. `Named`'s
    /// own `name`/`literal` and `Qualified`'s `namespace`/`name` are a
    /// routine name and a namespace, resolved by their own search, never
    /// through `slot_of`; `Trap` names a condition, not a variable.
    fn note_call(&mut self, call: &Call, symbols: &SymbolTable) {
        match call {
            Call::Named { args, .. } | Call::Qualified { args, .. } => {
                self.note_args(args, symbols);
            }
            Call::Dynamic { target, args } => {
                self.note(target, symbols);
                self.note_args(args, symbols);
            }
            Call::Trap(_) => {}
        }
    }

    /// One `Drop`/`Expose`/`Procedure`/`Use Local` target.
    ///
    /// `Indirect(id)`'s `id` is the *wrapper* variable read at run time to
    /// learn the real target's name (`DROP (v)` reads `v` itself, the same
    /// as any ordinary read) -- the target `v` names is not knowable until
    /// then, so nothing more can be pre-registered for it, and this file's
    /// own `a_runtime_name_grows_the_frame` test is exactly this case,
    /// expected to keep falling through to `extra`/`grow_slots`.
    /// `Direct(id)`'s spelling can be a simple variable, a stem or a
    /// compound with no tag saying which (`VariableRef`'s own doc comment)
    /// -- dispatched here on whether it contains a `.` at all, the same
    /// condition `compound_parts` itself requires before it can be called.
    fn note_variable_ref(&mut self, var_ref: &VariableRef, symbols: &SymbolTable) {
        let (VariableRef::Direct(id) | VariableRef::Indirect(id)) = *var_ref;
        let name = symbols.name(id);
        if name.contains('.') {
            self.note_compound_name(name.as_bytes());
        } else {
            self.bind(id, name.as_bytes());
        }
    }

    /// Registers every name a compound-shaped spelling's decomposition
    /// touches: the stem itself, and any tail piece that is a variable
    /// (D15a) -- a constant piece (a bare digit run, or the empty piece a
    /// trailing period leaves) names nothing. `name` is either a `Compound`
    /// expression's own interned spelling, or a `Drop`/`Expose`/
    /// `Procedure`/`Use Local` target's, once `note_variable_ref` has
    /// already established it is compound- or stem-shaped.
    ///
    /// Registers by name alone, with no `SymbolId` to bind alongside:
    /// neither the stem prefix nor a tail piece has one of its own --
    /// `compound_parts` only ever hands back a borrowed slice of the one
    /// interned spelling a `Compound` id carries, and a piece was never a
    /// token the scanner saw (`ast.rs`'s own `Compound` doc comment says
    /// so). That is exactly why `stem.rs`'s `tail_key`/`read_by_name`
    /// resolve both of these purely by name, which is what makes `names`,
    /// not `by_symbol`, the correct table for them to land on.
    fn note_compound_name(&mut self, name: &[u8]) {
        // `compound_parts` needs `&str` and panics on a name with no period
        // at all; every caller here already guarantees one -- a `Compound`
        // expression's own spelling always has one (`ast.rs`), and
        // `note_variable_ref` checks first. A Rexx symbol's interned
        // spelling is always ASCII (the scanner classifies only ASCII
        // symbol characters as `Stem`/`Compound`), so the conversion itself
        // cannot fail either.
        let name = std::str::from_utf8(name).expect("an interned compound name is ASCII");
        let (stem, tails) = compound_parts(name);
        self.slot_for(stem.as_bytes());
        for tail in tails {
            if let Tail::Variable(piece) = tail {
                self.slot_for(piece.as_bytes());
            }
        }
    }

    /// Assigns slots to every variable `expr` names, in source order.
    ///
    /// Recursive, and that recursion is on the interpreter thread's stack
    /// budget alongside `eval`'s: a left-deep 100,000-term expression is
    /// walked here as deeply as it is later evaluated.
    ///
    /// **Exhaustive over `ExprKind`, with no catch-all arm** -- see
    /// `build`'s doc comment for why: the original `_ => {}` here was the
    /// actual defect, not a placeholder, and matching every variant
    /// explicitly is what turns a future omission into a compile error.
    /// Reimplements the same shape `ExprKind::for_each_child`
    /// (`rexx-parse`'s `ast.rs`) already walks, rather than calling it:
    /// that method is `pub(crate)` to `rexx-parse`, so nothing outside that
    /// crate can reach it -- `rexx-parse/tests/gate_walk/mod.rs`'s
    /// `children_of` reimplements the identical shape for the identical
    /// reason, one crate over.
    fn note(&mut self, expr: &Expr, symbols: &SymbolTable) {
        match &expr.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) => {
                self.bind(*id, symbols.name(*id).as_bytes());
            }
            ExprKind::Compound(id) => self.note_compound_name(symbols.name(*id).as_bytes()),
            // A literal, a constant (its value is its own spelling, never a
            // variable) and a `.name` environment symbol (resolved through
            // the class/environment lookup, never `slot_of`) name nothing.
            // A namespace:name class resolver names a class, not a
            // variable, either.
            ExprKind::Literal(_)
            | ExprKind::Constant(_)
            | ExprKind::DotVariable(_)
            | ExprKind::ClassResolver { .. } => {}
            ExprKind::Prefix { operand, .. } => self.note(operand, symbols),
            ExprKind::Binary { left, right, .. } => {
                self.note(left, symbols);
                self.note(right, symbols);
            }
            ExprKind::Call { args, .. } | ExprKind::QualifiedCall { args, .. } => {
                self.note_args(args, symbols);
            }
            ExprKind::Message {
                target,
                super_class,
                args,
                ..
            } => {
                self.note(target, symbols);
                if let Some(super_class) = super_class {
                    self.note(super_class, symbols);
                }
                self.note_args(args, symbols);
            }
            ExprKind::List(items) => self.note_args(items, symbols),
            ExprKind::Logical(items) => {
                for item in items {
                    self.note(item, symbols);
                }
            }
            ExprKind::VariableReference(inner) => self.note(inner, symbols),
        }
    }

    /// `note` for an optional expression: an omitted argument, a `Say` with
    /// nothing after it, and every other `Option<Expr>` field this module
    /// walks share this one shape rather than each writing its own `if let`.
    fn note_opt(&mut self, expr: &Option<Expr>, symbols: &SymbolTable) {
        if let Some(expr) = expr {
            self.note(expr, symbols);
        }
    }

    /// `note` for an argument list, an omitted position skipped -- the
    /// `Vec<Option<Expr>>` shape a call's arguments, a list's elements and a
    /// `PARSE` template's targets all share.
    fn note_args(&mut self, args: &[Option<Expr>], symbols: &SymbolTable) {
        for arg in args.iter().flatten() {
            self.note(arg, symbols);
        }
    }

    /// Binds `name` to a slot, and `id` to the same one.
    ///
    /// Both views are updated together because they are one assignment seen
    /// two ways: a second symbol spelling the same name must land on the slot
    /// the first one got, which is why the slot number comes from `names` and
    /// never from `by_symbol`'s length.
    fn bind(&mut self, id: SymbolId, name: &[u8]) {
        let slot = self.slot_for(name);
        self.by_symbol.insert(id, slot);
    }

    /// Assigns `name` a slot, reusing one already assigned to the identical
    /// spelling. The name-only half of `bind`, factored out because a
    /// compound's stem prefix and tail-piece variables have no `SymbolId`
    /// to bind alongside them (`note_compound_name`'s own doc comment says
    /// why) and so go through this directly.
    fn slot_for(&mut self, name: &[u8]) -> usize {
        let next = self.names.len();
        *self.names.entry(name.into()).or_insert(next)
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

    /// The measured bug this fix closes (Task 6 fix dispatch): before it,
    /// every one of the first four bodies below built an *empty* plan --
    /// `Plan::note`'s `_ => {}` dropped `Stem`/`Compound` outright, and
    /// `Plan::build`'s own `_ => {}` meant an instruction it did not
    /// explicitly list (a `DROP`, a controlled `DO`) never even reached
    /// `note` at all. This asserts the plan's actual *contents* -- which
    /// names ended up in `plan.names` -- not merely that resolution still
    /// works afterwards through the `extra`/`grow_slots` fallback, which is
    /// the exact bar the dispatch set: neutering `Plan::build` to return an
    /// empty plan unconditionally must fail this test. It does not fail the
    /// pre-existing tests below, whose own assertions run through
    /// `slot_of`, which still gives the right *answer* via that fallback
    /// even when the plan is empty -- only the plan's own contents catch
    /// the regression.
    #[test]
    fn build_registers_every_name_a_stem_or_compound_touches() {
        let cases: &[(&[u8], &[&str])] = &[
            // say a.b -- the stem "A." and the tail-piece variable "B",
            // neither of which `note`'s old match handled at all.
            (&b"say a.b"[..], &["A.", "B"][..]),
            // a.1 = 'x' -- a bare digit tail is a constant (D15a), so only
            // the stem itself needs a slot.
            (b"a.1 = 'x'", &["A."]),
            // q. = 1 -- a bare Stem assignment target, dropped by the same
            // `_ => {}` a Compound was.
            (b"q. = 1", &["Q."]),
            // say v -- unaffected by this fix, kept as the control case:
            // if this one broke, the fix broke something that already
            // worked, not just left something unfixed.
            (b"say v", &["V"]),
            // drop a.b.c -- `Drop`'s own `_ => {}` in the pre-fix `build`
            // meant this never reached `note` at all, compound or not.
            // Both tail pieces are letter-led, so both are variables.
            (b"drop a.b.c", &["A.", "B", "C"]),
            // do i = 1 to 5 / end -- a controlled loop's control variable,
            // which never went through `note` before either: `Do` was not
            // one of `build`'s three explicitly handled kinds.
            (b"do i = 1 to 5\nend", &["I"]),
        ];
        for (source, expected_names) in cases {
            let program = parse_program(source.to_vec()).expect("test program parses");
            let plan = Plan::build(&program.main, &program.symbols);
            assert!(
                !plan.names.is_empty(),
                "{:?} must build a non-empty plan",
                String::from_utf8_lossy(source)
            );
            for name in *expected_names {
                assert!(
                    plan.names.contains_key(name.as_bytes()),
                    "{:?}: expected {name:?} in the plan, got {:?}",
                    String::from_utf8_lossy(source),
                    plan.names.keys().collect::<Vec<_>>()
                );
            }
        }
    }

    /// The table above only asserts that each expected name is *present*,
    /// so an over-wide `build` -- one that registers every name in the
    /// program, or anything else besides the right set -- would still pass
    /// it. That is exactly the shape of gap that let the original `_ =>
    /// {}` catch-all through unnoticed: presence checks cannot fail on
    /// extra entries, only on missing ones.
    ///
    /// These two assert the plan's key set **exactly**, and both expect
    /// nothing at all: `LEAVE`'s target is a block label matched against
    /// `LEAVE`/`ITERATE`/`END`, never a variable read through `slot_of`
    /// (`note_instruction`'s own comment on this, above); `.nil` is one of
    /// `note`'s no-op `ExprKind` arms. An empty expected set cannot pass
    /// vacuously -- there is no name a broken `build` could accidentally
    /// omit and still match -- so these two carry the most weight per
    /// assertion of anything in this module.
    #[test]
    fn build_registers_exactly_the_expected_set_not_merely_a_superset() {
        let cases: &[(&[u8], &[&str])] = &[(b"leave lbl", &[]), (b"say .nil", &[])];
        for (source, expected_names) in cases {
            let program = parse_program(source.to_vec()).expect("test program parses");
            let plan = Plan::build(&program.main, &program.symbols);

            let mut actual: Vec<&[u8]> = plan.names.keys().map(|k| &**k).collect();
            actual.sort();
            let mut expected: Vec<&[u8]> = expected_names.iter().map(|n| n.as_bytes()).collect();
            expected.sort();

            assert_eq!(
                actual,
                expected,
                "{:?}: expected the plan's key set to be exactly {expected_names:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    #[test]
    fn a_tail_piece_and_a_plain_variable_share_one_slot() {
        // b = 2 ; say a.b -> A.2 ; a.2 = 'hit' ; say a.b -> hit
        // An implementer who gives tail pieces their own slots gets A.B.
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
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
        let mut interp = Interp::new();
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
