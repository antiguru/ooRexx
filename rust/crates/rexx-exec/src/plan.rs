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
use crate::trace::{ChunkTrace, TraceEvent};
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
    /// The walk's own flag, and scratch: `note_instruction` clears it when the
    /// instruction it is walking can change the `TRACE` setting in force, and
    /// [`Plan::build`] sets it again in front of each one. It means nothing
    /// once that walk has finished.
    keeps_the_trace_setting: bool,
    /// What each instruction does to the `TRACE` setting in force, by index --
    /// the optimizing function `crate::ir::trace_flow` runs its forward
    /// analysis over, and the whole of what lets a compiled stream decide at
    /// compile time which value echoes it carries rather than gating each one.
    trace_events: Box<[TraceEvent]>,
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

    /// [`Plan::trace_events`], or an empty slice for a plan built without the
    /// walk that fills it.
    pub(crate) fn trace_events(&self) -> &[TraceEvent] {
        &self.trace_events
    }

    /// How the compound `id` names splits, if this plan's pass saw it.
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
            ..Plan::default()
        };
        let mut trace_events = Vec::with_capacity(body.instructions.len());
        for instruction in &body.instructions {
            // Set again in front of each instruction and read back after it,
            // because `note_instruction` only ever clears the flag: the walk
            // would otherwise report every instruction after the first
            // retracing one as retracing too.
            plan.keeps_the_trace_setting = true;
            plan.note_instruction(&instruction.kind, symbols);
            trace_events.push(match (plan.keeps_the_trace_setting, &instruction.kind) {
                (true, _) => TraceEvent::Keeps,
                (false, InstructionKind::Trace(setting)) => {
                    crate::trace::literal_trace_event(setting)
                }
                (false, _) => TraceEvent::Unknown,
            });
        }
        plan.trace_events = trace_events.into_boxed_slice();
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
                self.keeps_the_trace_setting = false;
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
                self.keeps_the_trace_setting = false;
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
                    self.keeps_the_trace_setting = false;
                }
                self.note_args(args, symbols);
            }
            Call::Qualified { name, args, .. } => {
                if names_trace(symbols.name(*name).as_bytes()) {
                    self.keeps_the_trace_setting = false;
                }
                self.note_args(args, symbols);
            }
            Call::Dynamic { target, args } => {
                // The target is a run-time value, so this cannot be read here.
                self.keeps_the_trace_setting = false;
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
                    self.keeps_the_trace_setting = false;
                }
                self.note_args(args, symbols);
            }
            ExprKind::QualifiedCall { name, args, .. } => {
                if names_trace(symbols.name(*name).as_bytes()) {
                    self.keeps_the_trace_setting = false;
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
mod tests;
