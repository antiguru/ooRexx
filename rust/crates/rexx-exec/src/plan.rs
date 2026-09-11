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

use crate::Interp;
use crate::run::{NameShape, shape_of};
use crate::trace::ChunkTrace;
use rexx_parse::{
    Call, CallTarget, CodeBody, DirectiveKind, Expr, ExprKind, Fragment, Instruction,
    InstructionKind, Loop, LoopKind, Parse, ParseSource, ProgramSource, Redirection, Signal,
    SymbolId, SymbolTable, Tail, Trace, Use, VariableRef, compound_parts,
};
use std::rc::Rc;

/// A loaded program's identity.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ProgramId(pub(crate) usize);

/// Which package something belongs to.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Package {
    /// The package the interpreter's own classes belong to. `PackageClass::
    /// getProgramName` answers `REXX` for it (`classes/PackageClass.hpp:147`).
    Rexx,
    /// The package a loaded program's own directives install into.
    Program(ProgramId),
}

/// What a class object's own `package` field holds -- `RexxClass::setPackage`,
/// the write `RexxClass::subclass` makes before it builds anything
/// (`classes/ClassClass.cpp:1582`).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ClassPackage {
    /// `ClassDirective::install` passes the installing package
    /// (`instructions/ClassDirective.cpp:200`, `:205`).
    Program(ProgramId),
    /// `OREF_NULL`, which is what `subclassRexx` and `mixinClassRexx` forward
    /// (`classes/ClassClass.cpp:1546`, `:1496`).
    Null,
}

/// Which code body of which loaded program a cached plan belongs to (D16).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct BodyKey {
    pub(crate) program: ProgramId,
    /// `None` is the program's main body, `Some(index)` is
    /// `directives[index]`'s.
    pub(crate) directive: Option<usize>,
}

/// Whether a body is entered as a method, which is the one thing
/// [`Plan::build`] needs to know about its caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BodyKind {
    /// A program's main body or a `::ROUTINE`: entering it binds neither name.
    Plain,
    /// A `::METHOD` or `::ATTRIBUTE` body.
    Method,
}

/// One tail piece of a compound's name, owned.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TailPiece {
    /// A piece that is empty or starts with a digit, so it can never be a
    /// variable name and stands for itself.
    Constant(Box<[u8]>),
    /// A simple variable whose value supplies this piece.
    Variable { name: Box<[u8]>, at: Option<usize> },
}

/// A compound's name, split into the pieces `Interp::tail_key` joins.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CompoundName {
    /// The stem half, its trailing period included (`compound_parts`' own
    /// convention, matching `addCompound`, `LanguageParser.cpp:2153`).
    pub(crate) stem: Box<[u8]>,
    /// The slot the plan holding this entry bound `stem` to, so that finding
    /// the stem object at a reference costs an index rather than a hash of the
    /// stem's bytes.
    pub(crate) stem_at: Option<usize>,
    /// The tail pieces in source order.
    pub(crate) tails: Box<[TailPiece]>,
}

impl CompoundName {
    /// Splits one compound-shaped interned spelling.
    pub(crate) fn split(name: &str) -> CompoundName {
        let (stem, tails) = compound_parts(name);
        CompoundName {
            stem: stem.as_bytes().into(),
            stem_at: None,
            tails: tails
                .into_iter()
                .map(|tail| match tail {
                    Tail::Constant(piece) => TailPiece::Constant(piece.as_bytes().into()),
                    Tail::Variable(piece) => TailPiece::Variable {
                        name: piece.as_bytes().into(),
                        at: None,
                    },
                })
                .collect(),
        }
    }
}

/// One body's variable-resolution plan, built by one upfront pass at first
/// execution (D16).
#[derive(Debug, Default)]
pub(crate) struct Plan {
    pub(crate) names: rexx_core::NameMap<Box<[u8]>, usize>,
    /// The slots `build` reserved for the names the interpreter assigns on
    /// its own -- `RESULT` after a call that returns a value, `SIGL` on a
    /// transfer. Cached because the alternative is `Interp::slot_of`, which
    /// hashes the name; measured, setting `RESULT` cost 236 user
    /// instructions per call against the oracle's 18.
    pub(crate) result_slot: Option<usize>,
    pub(crate) sigl_slot: Option<usize>,
    pub(crate) by_symbol: Vec<Option<usize>>,
    /// Whether nothing this body runs can change the `TRACE` setting in force
    /// while it runs, so that a compiled stream may decide at compile time
    /// which value echoes it carries rather than gating each one.
    never_retraces: bool,
    /// The static clause indent of every instruction in this body, by index.
    pub(crate) indents: Box<[usize]>,
    /// The 1-based source line every instruction in this body sits on, by
    /// index -- empty for a body built without a source, which is the
    /// fragment case `build`'s own parameter documents.
    pub(crate) lines: Box<[usize]>,
    /// How a compound-shaped symbol this body names splits, by
    /// `SymbolId::index`, `None` where this pass recorded nothing for it.
    pub(crate) compounds: Box<[Option<CompoundName>]>,
}

impl Plan {
    /// The static clause indent of `target`.
    pub(crate) fn indent_of(&self, instructions: &[Instruction], target: usize) -> usize {
        match self.indents.get(target) {
            Some(indent) => *indent,
            None => crate::run::static_indent(instructions, target),
        }
    }

    /// The 1-based source line `target`'s own clause starts on.
    pub(crate) fn line_at(
        &self,
        instruction: &Instruction,
        source: &ProgramSource,
        target: usize,
    ) -> usize {
        match self.lines.get(target) {
            Some(line) => {
                debug_assert_eq!(
                    *line,
                    source.line_of(instruction.clause_span.start),
                    "the plan's line table answers {line} for instruction {target}, and the \
                     source it is being read against puts that clause on another line -- so \
                     this plan was built from a different source"
                );
                *line
            }
            None => source.line_of(instruction.clause_span.start),
        }
    }

    /// How the compound `id` names splits, if this plan's pass saw it.
    pub(crate) fn never_retraces(&self) -> bool {
        self.never_retraces
    }

    pub(crate) fn compound(&self, id: SymbolId) -> Option<&CompoundName> {
        self.compounds.get(id.index())?.as_ref()
    }

    /// Walks `body` once and returns a finished table (D16: "built by one
    /// upfront pass", not populated lazily one name at a time).
    pub(crate) fn build(
        body: &CodeBody,
        symbols: &SymbolTable,
        source: Option<&ProgramSource>,
        kind: BodyKind,
    ) -> Plan {
        // `compounds` is sized here rather than filled as names arrive,
        // because `bind`/`note_compound_name` write into it by id and an id
        // is only an index into a table of that table's own length.
        let mut plan = Plan {
            compounds: std::iter::repeat_with(|| None)
                .take(symbols.len())
                .collect(),
            // Sized here for the reason `compounds` above is, and indexed the
            // same way: `bind` writes it by `SymbolId::index()`, and an id is
            // only an index into a table of that table's own length.
            by_symbol: std::iter::repeat_with(|| None)
                .take(symbols.len())
                .collect(),
            // Optimistic, and the walk below falsifies it; the field's own doc
            // has why the *default* is the other way round.
            never_retraces: true,
            ..Plan::default()
        };
        for instruction in &body.instructions {
            plan.note_instruction(&instruction.kind, symbols);
        }
        // **`RESULT`, `RC` and `SIGL` get a slot whether or not the body
        // mentions them**, which is what
        // `interpreter/execution/RexxLocalVariables.hpp` does with
        // `VARIABLE_RESULT`/`VARIABLE_RC`/`VARIABLE_SIGL`. Registered after
        // the body's own names rather than before them, because only being
        // *in* the plan matters here and taking the low indices would
        // renumber every slot this crate's tests pin.
        plan.result_slot = Some(plan.slot_for(b"RESULT"));
        plan.slot_for(b"RC");
        plan.sigl_slot = Some(plan.slot_for(b"SIGL"));
        // **`SELF` and `SUPER` for the same reason, in a method body only.**
        // `Interp::enter_method_body` binds both on every send through
        // `Interp::slot_of`, whose third source grows the frame and records
        // the name in `Activation::extra` -- so a method body that never
        // mentions either name paid two boxed keys and two map inserts per
        // send, which is exactly the cost the paragraph above describes for
        // `RESULT`. Measured on `bench-programs/dispatch.rex`, `slot_of`
        // reached from that binding was the largest single caller of `extra`.
        if kind == BodyKind::Method {
            plan.slot_for(b"SELF");
            plan.slot_for(b"SUPER");
        }
        plan.indents = crate::run::all_indents(&body.instructions);
        if let Some(source) = source {
            plan.lines = body
                .instructions
                .iter()
                .map(|instruction| source.line_of(instruction.clause_span.start))
                .collect();
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
                self.never_retraces = false;
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
                            Redirection::Stem(id) => self.bind(*id, symbols.name(*id)),
                            Redirection::Stream(expr) | Redirection::Using(expr) => {
                                self.note(expr, symbols);
                            }
                            Redirection::Default | Redirection::Normal => {}
                        }
                    }
                }
            }
            InstructionKind::Trace(trace) => {
                self.never_retraces = false;
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
    fn note_loop(&mut self, loop_: &Loop, symbols: &SymbolTable) {
        if let Some(counter) = loop_.counter {
            self.bind(counter, symbols.name(counter));
        }
        match &loop_.kind {
            LoopKind::Simple | LoopKind::Forever => {}
            LoopKind::Count(count) => self.note_opt(count, symbols),
            LoopKind::Controlled(controlled) => {
                self.bind(controlled.control, symbols.name(controlled.control));
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
                self.bind(*control, symbols.name(*control));
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
                    self.bind(*index, symbols.name(*index));
                }
                if let Some(item) = item {
                    self.bind(*item, symbols.name(*item));
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
            ParseSource::Var(id) => self.bind(*id, symbols.name(*id)),
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
            Call::Named { name, args, .. } => {
                if names_trace(name) {
                    self.never_retraces = false;
                }
                self.note_args(args, symbols);
            }
            Call::Qualified { name, args, .. } => {
                if names_trace(symbols.name(*name).as_bytes()) {
                    self.never_retraces = false;
                }
                self.note_args(args, symbols);
            }
            Call::Dynamic { target, args } => {
                // The target is a run-time value, so this cannot be read here.
                self.never_retraces = false;
                self.note(target, symbols);
                self.note_args(args, symbols);
            }
            Call::Trap(_) => {}
        }
    }

    /// One `Drop`/`Expose`/`Procedure`/`Use Local` target.
    fn note_variable_ref(&mut self, var_ref: &VariableRef, symbols: &SymbolTable) {
        let (VariableRef::Direct(id) | VariableRef::Indirect(id)) = *var_ref;
        let name = symbols.name(id);
        if name.contains('.') {
            self.note_compound_name(id, name);
        } else {
            self.bind(id, name);
        }
    }

    /// Registers every name a compound-shaped spelling's decomposition
    /// touches: the stem itself, and any tail piece that is a variable
    /// (D15a) -- a constant piece (a bare digit run, or the empty piece a
    /// trailing period leaves) names nothing. `name` is either a `Compound`
    /// expression's own interned spelling, or a `Drop`/`Expose`/
    /// `Procedure`/`Use Local` target's, once `note_variable_ref` has
    /// already established it is compound- or stem-shaped.
    fn note_compound_name(&mut self, id: SymbolId, name: &str) {
        let mut entry = CompoundName::split(name);
        // The stem's slot, kept for the same reason each piece's is: a
        // compound reference reads or writes the stem object through it, so
        // finding it costs an index into the frame instead of hashing the
        // stem's bytes at every reference.
        entry.stem_at = Some(self.slot_for(&entry.stem));
        for piece in &mut entry.tails {
            if let TailPiece::Variable { name, at } = piece {
                // The slot this pass was already computing and dropping. It
                // is kept on the piece so that resolving the piece at a
                // reference costs an index into the frame instead of hashing
                // the name: measured by `perf record` over
                // `samples/rexxcps.rex` at `5dc12a403`, before this work
                // (`REXX_ENGINE=ir`, `count=100`/`averaging=100`, 999 Hz),
                // hashing a `&[u8]` was 4.43% of self time with SipHash's own
                // `write` at a further 3.29% and `Interp::slot_of` at 1.72%.
                // The commit belongs with the figures: this function is what
                // takes that work out, so re-profiling at head will not find
                // them.
                *at = Some(self.slot_for(name));
            }
        }
        self.compounds[id.index()] = Some(entry);
    }

    /// Assigns slots to every variable `expr` names, in source order.
    fn note(&mut self, expr: &Expr, symbols: &SymbolTable) {
        match &expr.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) => {
                self.bind(*id, symbols.name(*id));
            }
            ExprKind::Compound(id) => self.note_compound_name(*id, symbols.name(*id)),
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
            ExprKind::Call { target, args } => {
                let named = match target {
                    CallTarget::Symbol(id) => names_trace(symbols.name(*id).as_bytes()),
                    CallTarget::Literal(bytes) => names_trace(bytes),
                };
                if named {
                    self.never_retraces = false;
                }
                self.note_args(args, symbols);
            }
            ExprKind::QualifiedCall { name, args, .. } => {
                if names_trace(symbols.name(*name).as_bytes()) {
                    self.never_retraces = false;
                }
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
    fn bind(&mut self, id: SymbolId, name: &str) {
        let slot = self.slot_for(name.as_bytes());
        self.by_symbol[id.index()] = Some(slot);
        if name.contains('.') {
            self.compounds[id.index()].get_or_insert_with(|| {
                let mut entry = CompoundName::split(name);
                if shape_of(name.as_bytes()) == NameShape::Stem {
                    entry.stem_at = Some(slot);
                }
                entry
            });
        }
    }

    /// Assigns `name` a slot, reusing one already assigned to the identical
    /// spelling. The name-only half of `bind`, factored out because a
    /// compound's stem prefix and tail-piece variables have no `SymbolId`
    /// to bind alongside them (`note_compound_name`'s own doc comment says
    /// why) and so go through this directly.
    /// The slot this pass bound `id` to, or `None` when it bound it none.
    pub(crate) fn slot_for_symbol(&self, id: SymbolId) -> Option<usize> {
        self.by_symbol.get(id.index()).copied().flatten()
    }

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
        source: &ProgramSource,
    ) -> Rc<Plan> {
        if let Some(plan) = self.plans.get(&key) {
            return Rc::clone(plan);
        }
        let plan = Rc::new(Plan::build(
            body,
            symbols,
            Some(source),
            self.body_kind(key),
        ));
        self.plans.insert(key, Rc::clone(&plan));
        plan
    }

    /// Whether the body `key` names is entered as a method.
    fn body_kind(&self, key: BodyKey) -> BodyKind {
        let Some(index) = key.directive else {
            return BodyKind::Plain;
        };
        let kind = self
            .programs
            .get(key.program.0)
            .and_then(|program| program.directives.get(index))
            .map(|directive| &directive.kind);
        match kind {
            Some(DirectiveKind::Method(_) | DirectiveKind::Attribute(_)) => BodyKind::Method,
            _ => BodyKind::Plain,
        }
    }

    /// The chunk for one body **under one trace setting**, from the cache or
    /// compiled and cached (D16's discipline, with the key D23 widens it by).
    pub(crate) fn chunk_for(
        &mut self,
        key: BodyKey,
        trace: ChunkTrace,
        body: &CodeBody,
        plan: &Plan,
    ) -> Option<Rc<crate::ir::Chunk>> {
        if let Some(chunk) = self.chunks.get(&(key, trace)) {
            return Some(Rc::clone(chunk));
        }
        match crate::ir::compile(body, plan, trace) {
            Ok(chunk) => {
                let chunk = Rc::new(chunk);
                self.chunks.insert((key, trace), Rc::clone(&chunk));
                Some(chunk)
            }
            Err(_) => {
                self.chunks_refused += 1;
                None
            }
        }
    }

    /// The slot `name` resolves to in the current frame, allocating one if it
    /// resolves to none.
    pub(crate) fn bound_slot_of(&self, name: &[u8]) -> Option<usize> {
        let activation = self.activation();
        activation
            .plan
            .slot_of(name)
            .or_else(|| activation.extra.get(name).copied())
    }

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
    pub(crate) fn fragment_plan(&mut self, fragment: &Fragment) -> (Vec<Option<usize>>, Plan) {
        // The same upfront pass, run against the fragment's own body, which
        // numbers its names 0..n in walk order. Those numbers are local to the
        // fragment and mean nothing to the enclosing frame; the loop below is
        // what translates them.
        // `BodyKind::Plain` whatever body the `INTERPRET` sits in: this plan
        // is a numbering of the *fragment's* names, and every name in it is
        // resolved against the enclosing frame below. `SELF` and `SUPER` are
        // already bound there when that frame is a method's, so asking for
        // them here would allocate nothing and change nothing -- and when it
        // is not a method's, it would grow the enclosing frame by two slots
        // that nothing writes.
        let local = Plan::build(&fragment.body, &fragment.symbols, None, BodyKind::Plain);

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

        let translation: Vec<Option<usize>> = local
            .by_symbol
            .iter()
            .map(|entry| entry.map(|local_slot| enclosing[local_slot]))
            .collect();

        // **The same plan with every slot it names moved into the enclosing
        // frame**, which is what lets a fragment compile at all: a chunk's
        // `Op::Load` and `Op::Store` carry plan slots, and a fragment's own
        // numbering means nothing in the frame those ops write.
        let mut compiled = local;
        compiled.names = compiled
            .names
            .iter()
            .map(|(name, slot)| (name.clone(), enclosing[*slot]))
            .collect();
        compiled.by_symbol = translation.clone();
        compiled.result_slot = compiled.result_slot.map(|slot| enclosing[slot]);
        compiled.sigl_slot = compiled.sigl_slot.map(|slot| enclosing[slot]);
        for entry in compiled.compounds.iter_mut().flatten() {
            entry.stem_at = None;
            for tail in entry.tails.iter_mut() {
                if let TailPiece::Variable { at, .. } = tail {
                    *at = None;
                }
            }
        }

        (translation, compiled)
    }
}

/// Whether a call names the `TRACE()` builtin, which sets the running
/// activation's own `TRACE` setting where every other call cannot.
fn names_trace(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"TRACE")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Novalue, planned_code};
    use rexx_parse::{Program, parse_interpret, parse_program};

    /// `indent_of` answers what `static_indent` answers, for every index.
    #[test]
    fn indent_of_answers_what_static_indent_answers_at_every_index() {
        let source = b"if 1 = 1 then\n  do i = 1 to 2\n    say i\n  end\nelse\n  nop\nselect\n  when 1 = 0 then nop\n  otherwise\n    say 'o'\nend\n";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let instructions = &program.main.instructions;
        assert!(instructions.len() > 8, "the program lost its shape");

        let expected: Vec<usize> = (0..instructions.len())
            .map(|index| crate::run::static_indent(instructions, index))
            .collect();
        let first: Vec<usize> = (0..instructions.len())
            .map(|index| plan.indent_of(instructions, index))
            .collect();
        assert_eq!(first, expected);
        // The stored table itself, not only what the accessor answers: an
        // accessor that recomputed on every call would satisfy the two
        // assertions above while the table it reads from stayed empty.
        assert_eq!(
            plan.indents.as_ref(),
            expected.as_slice(),
            "what the table actually holds"
        );
        // Not every index the same value, or the assertions above hold for a
        // memo that always answers with slot zero.
        assert!(
            expected
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                > 2,
            "this program does not distinguish enough indent levels to test with: {expected:?}"
        );
    }

    /// `line_at` answers what `ProgramSource::line_of` answers, for every
    /// index.
    #[test]
    fn line_at_answers_what_line_of_answers_at_every_index() {
        let source = b"nop\nsay 1\nif 1 = 1 then\n  nop\nelse\n  nop\ndo i = 1 to 2\n  say i\nend\nsay 'done'\n";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let instructions = &program.main.instructions;
        assert!(instructions.len() > 8, "the program lost its shape");

        let expected: Vec<usize> = instructions
            .iter()
            .map(|instruction| program.source.line_of(instruction.clause_span.start))
            .collect();
        let answered: Vec<usize> = instructions
            .iter()
            .enumerate()
            .map(|(index, instruction)| plan.line_at(instruction, &program.source, index))
            .collect();
        assert_eq!(answered, expected);
        assert_eq!(
            plan.lines.as_ref(),
            expected.as_slice(),
            "what the table actually holds"
        );
        // Not one line repeated, or a table that answered its first entry
        // everywhere would pass the two assertions above.
        assert!(
            expected
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                > 5,
            "this program does not distinguish enough lines to test with: {expected:?}"
        );
    }

    /// A plan built with no source carries no table, and `line_at` still
    /// answers -- which is the fragment's case and the whole of why the
    /// accessor has a fallback arm.
    #[test]
    fn line_at_falls_back_when_the_plan_was_built_without_a_source() {
        let source = b"nop\nsay 1\nsay 2\n";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, None, BodyKind::Plain);
        assert!(plan.lines.is_empty(), "no source, so no table");
        for (index, instruction) in program.main.instructions.iter().enumerate() {
            assert_eq!(
                plan.line_at(instruction, &program.source, index),
                program.source.line_of(instruction.clause_span.start),
                "index {index}"
            );
        }
    }

    /// `clause_line_at` answers what `clause_line` answers, at every index,
    /// with the override unset and with it set.
    #[test]
    fn clause_line_at_answers_what_clause_line_answers_and_the_override_still_wins() {
        let source = b"nop\nsay 1\nif 1 = 1 then\n  nop\nelse\n  nop\ndo i = 1 to 2\n  say i\nend\nsay 'done'\n";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = planned_code(&program, &plan);
        let mut interp = Interp::new();

        for (index, instruction) in program.main.instructions.iter().enumerate() {
            assert_eq!(
                interp.clause_line_at(&code, index, instruction, Some(&program.source)),
                interp.clause_line(Some(&program.source), instruction),
                "index {index}"
            );
        }

        interp.clause_line_override = Some(4242);
        for (index, instruction) in program.main.instructions.iter().enumerate() {
            assert_eq!(
                interp.clause_line_at(&code, index, instruction, Some(&program.source)),
                Some(4242),
                "the override outranks the table at index {index}"
            );
        }

        // `source: None` answers `None` whatever the override says, exactly
        // as `clause_line` does -- the two must not come apart on that arm
        // either.
        interp.clause_line_override = None;
        for (index, instruction) in program.main.instructions.iter().enumerate() {
            assert_eq!(
                interp.clause_line_at(&code, index, instruction, None),
                None,
                "index {index}"
            );
        }
    }

    /// The table matches `ProgramSource::line_of` for every instruction of
    /// every corpus program that parses.
    #[test]
    fn build_fills_what_line_of_computes_for_every_corpus_program() {
        let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
        let mut compared = 0usize;
        let mut positions = 0usize;
        let mut directories = vec![corpus];
        while let Some(directory) = directories.pop() {
            for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
                let path = entry.expect("a readable directory entry").path();
                if path.is_dir() {
                    directories.push(path);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rex") {
                    continue;
                }
                let bytes = std::fs::read(&path).expect("a readable corpus program");
                let Ok(program) = parse_program(bytes) else {
                    continue;
                };
                let plan = Plan::build(
                    &program.main,
                    &program.symbols,
                    Some(&program.source),
                    BodyKind::Plain,
                );
                let expected: Vec<usize> = program
                    .main
                    .instructions
                    .iter()
                    .map(|instruction| program.source.line_of(instruction.clause_span.start))
                    .collect();
                assert_eq!(
                    plan.lines.as_ref(),
                    expected.as_slice(),
                    "{} disagrees",
                    path.display()
                );
                compared += 1;
                positions += program.main.instructions.len();
            }
        }
        assert!(
            compared > 40 && positions > 500,
            "only {compared} programs and {positions} positions were compared, \
             which is too little of the corpus to have tested anything"
        );
    }

    /// Pushes a fresh top-level activation for `program`, the same setup
    /// `Interp::run` does, so these tests can drive `slot_of`/`Plan` through
    /// a live activation without running the whole instruction loop.
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
        interp.push_activation(crate::Activation::new(
            id,
            Rc::clone(&program),
            program_id,
            plan,
            frame,
        ));
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
            let plan = Plan::build(
                &program.main,
                &program.symbols,
                Some(&program.source),
                BodyKind::Plain,
            );
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
    #[test]
    fn build_registers_exactly_the_expected_set_not_merely_a_superset() {
        let cases: &[(&[u8], &[&str])] = &[(b"leave lbl", &[]), (b"say .nil", &[])];
        for (source, expected_names) in cases {
            let program = parse_program(source.to_vec()).expect("test program parses");
            let plan = Plan::build(
                &program.main,
                &program.symbols,
                Some(&program.source),
                BodyKind::Plain,
            );

            // The reserved names are in every plan, so they are checked
            // for presence once and then set aside; what each case is about
            // is the body's *own* names, and that comparison stays exact.
            for reserved in [b"RESULT".as_slice(), b"RC".as_slice(), b"SIGL".as_slice()] {
                assert!(
                    plan.names.contains_key(reserved),
                    "{:?}: every plan holds {}",
                    String::from_utf8_lossy(source),
                    String::from_utf8_lossy(reserved)
                );
            }
            let mut actual: Vec<&[u8]> = plan
                .names
                .keys()
                .map(|k| &**k)
                .filter(|name| !matches!(*name, b"RESULT" | b"RC" | b"SIGL"))
                .collect();
            actual.sort();
            let mut expected: Vec<&[u8]> = expected_names.iter().map(|n| n.as_bytes()).collect();
            expected.sort();

            assert_eq!(
                actual,
                expected,
                "{:?}: expected the plan's own key set to be exactly {expected_names:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// A method body's plan holds `SELF` and `SUPER`, and no other body's
    /// does.
    #[test]
    fn only_a_method_body_holds_self_and_super() {
        let source = b"nop\n::routine r\n  nop\n::class k\n::method m\n  x = 1\n";
        let program = Rc::new(parse_program(source.to_vec()).expect("test program parses"));
        let mut interp = Interp::new();
        let program_id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));

        let position = |wanted: fn(&DirectiveKind) -> bool| {
            program
                .directives
                .iter()
                .position(|directive| wanted(&directive.kind))
                .expect("the program has this directive")
        };
        let routine = position(|kind| matches!(kind, DirectiveKind::Routine(_)));
        let method = position(|kind| matches!(kind, DirectiveKind::Method(_)));

        for (selector, what, expected) in [
            (None, "the main body", false),
            (Some(routine), "::ROUTINE r", false),
            (Some(method), "::METHOD m", true),
        ] {
            let body = crate::activation::body_of(&program, selector)
                .unwrap_or_else(|| panic!("{what} has a body"));
            let plan = interp.plan_for(
                BodyKey {
                    program: program_id,
                    directive: selector,
                },
                body,
                &program.symbols,
                &program.source,
            );
            for name in [b"SELF".as_slice(), b"SUPER".as_slice()] {
                assert_eq!(
                    plan.slot_of(name).is_some(),
                    expected,
                    "{what}: {} in the plan, which holds {:?}",
                    String::from_utf8_lossy(name),
                    plan.names.keys().collect::<Vec<_>>()
                );
            }
        }

        // Registered after the body's own names, so the method body's own `X`
        // is still slot 0 -- the property that let the two be added without
        // renumbering anything a plan already answered.
        let body =
            crate::activation::body_of(&program, Some(method)).expect("::METHOD m has a body");
        let plan = interp.plan_for(
            BodyKey {
                program: program_id,
                directive: Some(method),
            },
            body,
            &program.symbols,
            &program.source,
        );
        assert_eq!(plan.slot_of(b"X"), Some(0));
    }

    /// `build` records a compound's split under **the compound's own id**.
    #[test]
    fn build_records_a_compounds_split_under_the_compounds_own_id() {
        let source = b"drop dd.jj; do aa.ii = 1 to 2; say v.i.7; end";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );

        let variable = |text: &str, at: Option<usize>| TailPiece::Variable {
            name: text.as_bytes().into(),
            at,
        };
        let constant = |text: &str| TailPiece::Constant(text.as_bytes().into());

        let mut found: Vec<(&str, &CompoundName)> = Vec::new();
        for instruction in &program.main.instructions {
            match &instruction.kind {
                InstructionKind::Drop { variables } => {
                    let (VariableRef::Direct(id) | VariableRef::Indirect(id)) = variables[0];
                    found.push((symbols_name(&program, id), expect_entry(&plan, id)));
                }
                InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) => {
                    let LoopKind::Controlled(controlled) = &loop_.kind else {
                        panic!("expected a controlled loop, got {:?}", loop_.kind);
                    };
                    let id = controlled.control;
                    found.push((symbols_name(&program, id), expect_entry(&plan, id)));
                }
                InstructionKind::Say {
                    expression: Some(expr),
                } => {
                    let ExprKind::Compound(id) = expr.kind else {
                        panic!("expected a compound expression, got {:?}", expr.kind);
                    };
                    found.push((symbols_name(&program, id), expect_entry(&plan, id)));
                }
                _ => {}
            }
        }

        let expected: Vec<(&str, CompoundName)> = vec![
            (
                "DD.JJ",
                CompoundName {
                    stem: b"DD.".as_slice().into(),
                    stem_at: Some(0),
                    tails: vec![variable("JJ", Some(1))].into(),
                },
            ),
            (
                "AA.II",
                CompoundName {
                    stem: b"AA.".as_slice().into(),
                    stem_at: None,
                    tails: vec![variable("II", None)].into(),
                },
            ),
            (
                "V.I.7",
                CompoundName {
                    stem: b"V.".as_slice().into(),
                    stem_at: Some(3),
                    tails: vec![variable("I", Some(4)), constant("7")].into(),
                },
            ),
        ];
        // The slot numbers above are the pass's own, in the order it assigned
        // them: `DD.` then `JJ` for the `DROP`, `AA.II` whole for the control
        // variable, then `V.` and `I` for the `SAY`. Asserted here so that a
        // reader can see where 1 and 4 come from, and so that a change to the
        // assignment order fails on the map rather than only on the pieces.
        let mut names: Vec<(&[u8], usize)> = plan
            .names
            .iter()
            .map(|(name, slot)| (&**name, *slot))
            .collect();
        names.sort_by_key(|(_, slot)| *slot);
        assert_eq!(
            names,
            vec![
                (b"DD.".as_slice(), 0),
                (b"JJ".as_slice(), 1),
                (b"AA.II".as_slice(), 2),
                (b"V.".as_slice(), 3),
                (b"I".as_slice(), 4),
                // `Plan::build` registers these after the body's own names,
                // so they land at the end and the numbers above are unmoved.
                (b"RESULT".as_slice(), 5),
                (b"RC".as_slice(), 6),
                (b"SIGL".as_slice(), 7),
            ]
        );
        let expected: Vec<(&str, &CompoundName)> = expected
            .iter()
            .map(|(name, entry)| (*name, entry))
            .collect();
        assert_eq!(found, expected);
    }

    /// One symbol recorded by `note_compound_name` and by `bind` keeps the
    /// slots, whichever order the pass reaches them in.
    #[test]
    fn a_control_variable_does_not_take_the_slots_off_a_compound_already_seen() {
        let source = b"say v.i; do v.i = 1 to 2; end";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );

        let InstructionKind::Say {
            expression: Some(expr),
        } = &program.main.instructions[0].kind
        else {
            panic!(
                "expected a SAY first, got {:?}",
                program.main.instructions[0].kind
            );
        };
        let ExprKind::Compound(id) = expr.kind else {
            panic!("expected a compound expression, got {:?}", expr.kind);
        };
        // The same id in both positions is the whole premise, so it is
        // checked rather than assumed: `bind` addresses `compounds` by the
        // control variable's id, and if that were a different symbol from the
        // `SAY`'s there would be no overwrite to guard against.
        let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
            &program.main.instructions[1].kind
        else {
            panic!(
                "expected a DO second, got {:?}",
                program.main.instructions[1].kind
            );
        };
        let LoopKind::Controlled(controlled) = &loop_.kind else {
            panic!("expected a controlled loop, got {:?}", loop_.kind);
        };
        assert_eq!(controlled.control, id);

        assert_eq!(
            expect_entry(&plan, id).tails.as_ref(),
            [TailPiece::Variable {
                name: b"I".as_slice().into(),
                at: Some(1),
            }]
        );
    }

    /// The other build order for one symbol, which rests on the other
    /// filler's rule.
    #[test]
    fn a_compound_seen_after_the_control_variable_still_gets_its_slots() {
        let source = b"do v.i = 1 to 2; end; say v.i";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );

        let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
            &program.main.instructions[0].kind
        else {
            panic!(
                "expected a DO first, got {:?}",
                program.main.instructions[0].kind
            );
        };
        let LoopKind::Controlled(controlled) = &loop_.kind else {
            panic!("expected a controlled loop, got {:?}", loop_.kind);
        };
        let id = controlled.control;

        let last = program.main.instructions.last().expect("a body");
        let InstructionKind::Say {
            expression: Some(expr),
        } = &last.kind
        else {
            panic!("expected a SAY last, got {:?}", last.kind);
        };
        let ExprKind::Compound(say_id) = expr.kind else {
            panic!("expected a compound expression, got {:?}", expr.kind);
        };
        assert_eq!(say_id, id);

        // `V.I` whole takes slot 0 from `bind`, then `V.` and `I` take 1 and
        // 2 from `note_compound_name`. The piece's slot is 2, so the entry
        // that survived is the second one.
        assert_eq!(
            expect_entry(&plan, id).tails.as_ref(),
            [TailPiece::Variable {
                name: b"I".as_slice().into(),
                at: Some(2),
            }]
        );
    }

    /// **A compound tail piece can be bound in `extra` rather than in the
    /// plan, and this is the shape that reaches it.**
    #[test]
    fn a_tail_piece_with_no_plan_slot_binds_in_extra() {
        let mut interp = Interp::new();
        let program =
            parse_program(b"do za.zi = 1 to 3\nnop\nend".to_vec()).expect("test program parses");
        let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
            &program.main.instructions[0].kind
        else {
            panic!(
                "expected a DO first, got {:?}",
                program.main.instructions[0].kind
            );
        };
        let LoopKind::Controlled(controlled) = &loop_.kind else {
            panic!("expected a controlled loop, got {:?}", loop_.kind);
        };
        let id = controlled.control;

        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        assert_eq!(
            plan.slot_of(b"ZI"),
            None,
            "the plan binds the whole ZA.ZI and nothing named ZI"
        );
        assert_eq!(
            expect_entry(&plan, id).tails.as_ref(),
            [TailPiece::Variable {
                name: b"ZI".as_slice().into(),
                at: None,
            }]
        );

        let program = activate(&mut interp, program);
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = planned_code(&program, &plan);
        // Unset, so the piece derives its own spelling -- the ordinary
        // uninitialised read, reached here through `extra` and growth.
        let key = interp
            .tail_key(&code, id)
            .expect("the tail pieces are strings");
        assert_eq!(key, b"ZI");
        assert!(
            interp.activation().extra.contains_key(b"ZI".as_slice()),
            "the piece must be recorded in extra, which is the source a \
             precomputed slot would have skipped"
        );
    }

    /// **A compound's stem can be bound in `extra` rather than in the plan,
    /// and this is the shape that reaches it.**
    #[test]
    fn a_stem_with_no_plan_slot_binds_in_extra() {
        let mut interp = Interp::new();
        let program =
            parse_program(b"do za.zi = 1 to 3\nnop\nend".to_vec()).expect("test program parses");
        let (InstructionKind::Do(loop_) | InstructionKind::Loop(loop_)) =
            &program.main.instructions[0].kind
        else {
            panic!(
                "expected a DO first, got {:?}",
                program.main.instructions[0].kind
            );
        };
        let LoopKind::Controlled(controlled) = &loop_.kind else {
            panic!("expected a controlled loop, got {:?}", loop_.kind);
        };
        let id = controlled.control;

        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        assert_eq!(
            plan.slot_of(b"ZA."),
            None,
            "the plan binds the whole ZA.ZI and nothing named ZA."
        );
        assert_eq!(expect_entry(&plan, id).stem_at, None);

        let program = activate(&mut interp, program);
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = planned_code(&program, &plan);
        let (stem_name, stem_at) = code.stem(id);
        assert_eq!(stem_name, b"ZA.");
        assert_eq!(stem_at, None);

        // Nothing has been written, so the tail derives its own name from the
        // read site's spelling -- the ordinary uninitialised compound read,
        // reached here through `extra` and growth.
        let key = interp
            .tail_key(&code, id)
            .expect("the tail pieces are strings");
        let (value, novalue) = interp.stem_get_at(stem_name, stem_at, &key);
        assert_eq!(novalue, Novalue::Unset);
        assert_eq!(&*interp.to_text(value), b"ZA.ZI");
        assert!(
            interp.activation().extra.contains_key(b"ZA.".as_slice()),
            "the stem must be recorded in extra, which is the source a \
             precomputed slot would have skipped"
        );
    }

    fn symbols_name(program: &Program, id: SymbolId) -> &str {
        program.symbols.name(id)
    }

    fn expect_entry(plan: &Plan, id: SymbolId) -> &CompoundName {
        plan.compound(id)
            .expect("the pass recorded this compound's split under its own id")
    }

    /// **`Plan::bind` records the stem's slot for a stem-shaped name and not
    /// for a compound-shaped one**, and the pair is what makes that a decision
    /// rather than a coincidence of one spelling.
    #[test]
    fn bind_keeps_a_stem_shaped_names_own_slot_as_its_stems() {
        let source = b"zt. = 'v'\ndo zs. = 1 to 2\nnop\nend\ndo aa.ii = 1 to 2\nnop\nend";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );

        let mut found: Vec<(&str, Option<usize>)> = Vec::new();
        for instruction in &program.main.instructions {
            let id = match &instruction.kind {
                InstructionKind::Assignment { target, .. } => match target.kind {
                    ExprKind::Stem(id) => id,
                    ref other => panic!("expected a stem target, got {other:?}"),
                },
                InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) => {
                    let LoopKind::Controlled(controlled) = &loop_.kind else {
                        panic!("expected a controlled loop, got {:?}", loop_.kind);
                    };
                    controlled.control
                }
                _ => continue,
            };
            found.push((symbols_name(&program, id), expect_entry(&plan, id).stem_at));
        }

        assert_eq!(
            found,
            vec![("ZT.", Some(0)), ("ZS.", Some(1)), ("AA.II", None)]
        );
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
        interp.roots.set_frame_slot(frame, b_slot, two);

        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = planned_code(&program, &plan);
        let key = interp
            .tail_key(&code, id)
            .expect("the tail pieces are strings");
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

        let uninit = interp.stem_get(b"A.", &key).0;
        assert_eq!(&*interp.to_text(uninit), b"A.2");

        let hit = interp.text(b"hit");
        interp.stem_set(b"A.", &key, hit);
        let after = interp.stem_get(b"A.", &key).0;
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

        // `RESULT`, `RC` and `SIGL` are in every plan (see `Plan::build`), so
        // a body with no variables of its own holds exactly those and nothing
        // more -- which is still the precondition this test needs: `X` is not
        // among them, so reaching it has to grow the frame.
        assert_eq!(
            interp.activation().plan.len(),
            3,
            "a body with no variables of its own holds only the reserved names"
        );
        assert!(
            interp.activation().plan.slot_of(b"X").is_none(),
            "the name this test grows the frame for must not already have a slot"
        );

        let one = interp.number(
            rexx_num::Number::parse("1").unwrap(),
            9,
            rexx_num::Form::Scientific,
        );
        let x_slot = interp.slot_of(b"X");
        let frame = interp.activation().frame;
        interp.roots.set_frame_slot(frame, x_slot, one);

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
        interp.roots.set_frame_slot(frame, i_slot, abc);

        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            BodyKind::Plain,
        );
        let code = planned_code(&program, &plan);
        let key = interp
            .tail_key(&code, id)
            .expect("the tail pieces are strings");
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
        let (slots, plan) = interp.fragment_plan(&fragment);
        // The table is sized by the fragment's own symbol table and is mostly
        // `None`; what the fragment *names* is the count of bound entries.
        assert_eq!(
            slots.iter().flatten().count(),
            1,
            "the fragment names exactly one variable"
        );

        let enclosing_slot = interp.slot_of(b"NEWVAR");
        let fragment_slot = slots.iter().flatten().next().copied().expect("one entry");
        // **The remapped plan names the same slot**, which is the half the
        // compiled path depends on: a chunk's ops carry plan slots, and a plan
        // still numbering the fragment's own names would write into whatever
        // the enclosing frame happens to hold at that index.
        assert_eq!(
            plan.names.get(b"NEWVAR".as_slice()).copied(),
            Some(enclosing_slot),
            "the remapped plan's name map still holds the fragment's own \
             local numbering"
        );
        assert_eq!(
            plan.by_symbol.iter().flatten().copied().collect::<Vec<_>>(),
            vec![enclosing_slot],
            "the remapped plan's by_symbol disagrees with the translation \
             `Code::slots` is built from"
        );
        assert_eq!(
            fragment_slot, enclosing_slot,
            "the fragment's own id must resolve to the SAME slot the \
             enclosing body would use for the same name"
        );
    }
    /// **What may change the `TRACE` setting under a running chunk**, which is
    /// the whole of what lets `ir::compile` decide a body's value echoes once
    /// instead of gating each one.
    #[test]
    fn a_body_that_can_reach_the_trace_setting_is_the_one_that_says_so() {
        let retraces = |source: &[u8]| {
            let program = parse_program(source.to_vec()).expect("test program parses");
            !Plan::build(&program.main, &program.symbols, None, BodyKind::Plain).never_retraces()
        };

        // The instruction, in each of its forms.
        assert!(retraces(b"trace i"));
        assert!(retraces(b"zv = 'i'\ntrace value zv"));
        // The builtin, which sets the *running* activation's own setting where
        // every other call cannot reach it.
        assert!(retraces(b"zg = trace('i')"));
        assert!(retraces(b"call trace 'i'"));
        assert!(retraces(b"zg = TRACE('i')"));
        // Its argument list is no shelter: the walk reaches nested calls.
        assert!(retraces(b"zg = length(trace('i'))"));
        // Text this cannot read.
        assert!(retraces(b"interpret zv"));
        assert!(retraces(b"call (zv) 1"));

        // The adjacent successes: a call, an assignment and a loop that name
        // no route to the setting.
        assert!(!retraces(b"zg = length('ab')"));
        assert!(!retraces(b"call charout , 'x'"));
        assert!(!retraces(b"do zi = 1 to 3\nzs = zi + 1\nend"));
        assert!(!retraces(b"say 'x'"));
        // A *variable* spelled TRACE is not a call to it, but blocking one
        // costs only the optimisation -- so this row records which way the
        // guard falls rather than asserting it must not block.
        let _ = retraces(b"trace_var = 1");
    }
}
