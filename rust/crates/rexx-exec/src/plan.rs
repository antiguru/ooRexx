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
use crate::run::{NameShape, shape_of};
use crate::trace::ChunkTrace;
use rexx_parse::{
    Call, CallTarget, CodeBody, Expr, ExprKind, Fragment, Instruction, InstructionKind, Loop,
    LoopKind, Parse, ParseSource, ProgramSource, Redirection, Signal, SymbolId, SymbolTable, Tail,
    Trace, Use, VariableRef, compound_parts,
};
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

/// Which package something belongs to.
///
/// **A variant for the interpreter's own package rather than an absent
/// [`ProgramId`].** `Interp::package_objects` keys on this, and the class a
/// program installed and the class the crate's own bootstrap registered are
/// different packages with different `~name` answers -- measured,
/// `.Array~package~name` is `REXX` and a `::CLASS`'s is the program's own
/// path. Spelling the first as an absence puts it in the same type as "there
/// is no package at all", which is a different state that `Caller::package`
/// (`dispatch.rs`) carries and that the oracle's `checkPackage` refuses
/// (`classes/ObjectClass.cpp:665`-`:669`).
///
/// **`Interp::class_packages` does not key on this and wants no variant.** It
/// maps a class handle to the [`ProgramId`] whose directives installed that
/// class, so a class the bootstrap registered is simply absent from it, and
/// `Interp::package_object_for` is the one reader that turns the absence into
/// `Package::Rexx`. Putting a `Package` in there would give one state two
/// spellings -- absent, and present as `Rexx` -- which is the shape this enum
/// exists to remove.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Package {
    /// The package the interpreter's own classes belong to. `PackageClass::
    /// getProgramName` answers `REXX` for it (`classes/PackageClass.hpp:147`).
    Rexx,
    /// The package a loaded program's own directives install into.
    Program(ProgramId),
}

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
    /// `None` is the program's main body, `Some(index)` is
    /// `directives[index]`'s.
    ///
    /// **The same selector `Activation::body` carries**, decided with it by
    /// Task 3 rather than separately: a plan is cached under this key and
    /// looked up again by whatever runs that body, so if the two spellings
    /// denoted different things a body would run under another body's plan.
    /// `activation.rs`'s own `body_of` is the single place either is turned
    /// into a `&CodeBody`.
    ///
    /// `Interp::run` builds the main body's plan under `None`;
    /// `Interp::invoke_call`'s `::ROUTINE` step and
    /// `Interp::enter_method_body`'s `::METHOD`/`::ATTRIBUTE` step build
    /// theirs under `Some(index)`. Each gets a plan of its own rather than sharing
    /// the caller's, and that is what makes its pool safe to isolate: a
    /// different `CodeBody` means a different name-to-slot map, so the
    /// slot-index identity `PROCEDURE EXPOSE`'s alias bitset rests on does
    /// not hold across the two.
    pub(crate) directive: Option<usize>,
}

/// One tail piece of a compound's name, owned.
///
/// `rexx_parse::Tail` is the same two cases borrowed from the interned
/// spelling, and it is what `CompoundName::split` classifies with. An owned
/// copy is what a `Plan` can hold: the plan outlives no borrow of the
/// `SymbolTable` -- it is an `Rc` cached on `Interp` while the table lives
/// behind an `Rc<Program>` -- and owning the bytes is also what lets
/// `Interp::tail_key` join a key without touching the symbol table at all.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TailPiece {
    /// A piece that is empty or starts with a digit, so it can never be a
    /// variable name and stands for itself.
    Constant(Box<[u8]>),
    /// A simple variable whose value supplies this piece.
    ///
    /// `at` is the slot the plan holding this entry bound `name` to, so that
    /// reading the piece costs an index rather than a hash of its bytes.
    /// **`None` is a piece the plan assigned no slot**, and it is an ordinary
    /// outcome rather than a gap: `Plan::bind` records a split and assigns
    /// nothing, for the reason its own doc comment gives, so an entry existing
    /// does not mean its pieces carry slots. `Interp::read_by_name_at`
    /// resolves the name the ordinary way when there is none, exactly as every
    /// piece did before this field existed.
    ///
    /// A `Some` slot is never a *different* answer from resolving the name.
    /// `Interp::slot_of` reads the plan's own name map first and the
    /// activation's `extra` only after it misses, and a slot lands here only
    /// because `slot_for` put `name` in that same map -- so for a piece that
    /// carries one, `extra` was already unreachable. `Interp::join_tails`
    /// carries the debug tripwire for the premise that can break, which is the
    /// entry belonging to some plan other than the running activation's.
    Variable { name: Box<[u8]>, at: Option<usize> },
}

/// A compound's name, split into the pieces `Interp::tail_key` joins.
///
/// One of these per compound-shaped symbol in a body, keyed by that symbol's
/// own id on `Plan::compounds`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CompoundName {
    /// The stem half, its trailing period included (`compound_parts`' own
    /// convention, matching `addCompound`, `LanguageParser.cpp:2153`).
    pub(crate) stem: Box<[u8]>,
    /// The slot the plan holding this entry bound `stem` to, so that finding
    /// the stem object at a reference costs an index rather than a hash of the
    /// stem's bytes.
    ///
    /// **`None` is a stem the plan assigned no slot**, and it is an ordinary
    /// outcome rather than a gap, the same one `TailPiece::Variable`'s own
    /// `at` records: `Plan::bind` binds a *compound*-shaped name whole and
    /// gives its stem half nothing, and `CompoundName::split` is entered by an
    /// `INTERPRET` fragment with no plan at all. `Interp::stem_slot` resolves
    /// the name the ordinary way when there is none, exactly as every stem
    /// accessor did before this field existed.
    ///
    /// **A stem-shaped name is its own stem half, and `Plan::bind` fills this
    /// in for one.** `zs. = 'one'` and `do zs. = 1 to 3` bind `ZS.` whole, so
    /// the slot bound to the symbol *is* the stem's slot and there is no
    /// second name to assign. That is what lets `Interp::stem_assign_at` reach
    /// a bare stem's slot on both engines, where a compiled `Op::Store` would
    /// reach it on only one.
    ///
    /// A `Some` slot is never a *different* answer from resolving the name.
    /// `Interp::slot_of` reads the plan's own name map first and the
    /// activation's `extra` only after it misses, and a slot lands here only
    /// because `slot_for` put `stem` in that same map -- so for a stem that
    /// carries one, `extra` was already unreachable. A stem that carries
    /// **none** does reach `extra`, measured: `do za.zi = 1 to 3` binds the
    /// whole `ZA.ZI` and nothing named `ZA.`, so the loop's own read grows
    /// `ZA.` into `extra` and hits it on every pass.
    pub(crate) stem_at: Option<usize>,
    /// The tail pieces in source order.
    pub(crate) tails: Box<[TailPiece]>,
}

impl CompoundName {
    /// Splits one compound-shaped interned spelling.
    ///
    /// **The single definition of the split**, entered both by the upfront
    /// pass that fills `Plan::compounds` and by `Interp::tail_key`'s fallback
    /// for a body with no plan entry, so the two cannot come to disagree
    /// about how a name decomposes.
    ///
    /// `name` must hold a period: `compound_parts` panics without one. Every
    /// caller already guarantees it -- a `Compound` expression's own spelling
    /// always has one (`ast.rs`), and `Plan::note_variable_ref` checks before
    /// it calls.
    ///
    /// **The stem and every variable piece come back with no slot**, because
    /// how a name splits is a property of the text alone while a slot is a
    /// property of the plan the entry is going into, and this function is
    /// entered by a fragment that has no plan at all. `note_compound_name`
    /// fills the slots in afterwards, for the entries that get any.
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
    pub(crate) names: rexx_core::NameMap<Box<[u8]>, usize>,
    /// The slots `build` reserved for the names the interpreter assigns on
    /// its own -- `RESULT` after a call that returns a value, `SIGL` on a
    /// transfer. Cached because the alternative is `Interp::slot_of`, which
    /// hashes the name; measured, setting `RESULT` cost 236 user
    /// instructions per call against the oracle's 18.
    ///
    /// `None` on a `Plan::default()`, which `Interp` hands to an activation
    /// that has no body of its own (`lib.rs`) and whose name map is empty --
    /// there is no slot to name, so those callers keep the lookup.
    pub(crate) result_slot: Option<usize>,
    pub(crate) sigl_slot: Option<usize>,
    pub(crate) by_symbol: Vec<Option<usize>>,
    /// Whether nothing this body runs can change the `TRACE` setting in force
    /// while it runs, so that a compiled stream may decide at compile time
    /// which value echoes it carries rather than gating each one.
    ///
    /// **`false` is the safe answer and the default one**, which is what a
    /// [`Plan::default()`] gives a body-less activation and a fragment: it
    /// means "may retrace", so every echo is emitted and gated at run time
    /// exactly as it always was. [`Plan::build`] starts this `true` and lets
    /// the walk below falsify it.
    ///
    /// **What falsifies it** is a `TRACE` instruction, an `INTERPRET` or
    /// `OPTIONS` whose text is not known here, a call this body makes to
    /// `TRACE()` -- the builtin sets the running activation's own setting
    /// (`builtin::state::trace`) -- and a `CALL (expr)` whose target is not
    /// known until it runs. A call to anything else cannot reach it: a routine
    /// or method runs in an activation of its own, and `push_activation` and
    /// `pop_activation` restore this one's setting around it.
    never_retraces: bool,
    /// The static clause indent of every instruction in this body, by index.
    ///
    /// `static_indent` walks the flat instruction list from position zero to
    /// answer for one index, and `step_in_temps_frame` asks for the answer on
    /// every clause it steps -- so the cost of computing it on demand is the
    /// length of the body, per clause. Measured on `samples/rexxcps.rex`,
    /// where the body is long enough for that to show: `indent_in_range` was
    /// the single largest self-time function in the profile at 8.0%.
    ///
    /// The answer depends on the instruction list alone (`static_indent`'s
    /// own doc comment: never on which iteration is running), so it belongs
    /// here, with the rest of what one upfront pass over the body knows.
    ///
    /// Filled by `all_indents`, which walks the body **once** for every
    /// position rather than once per position -- that distinction is what
    /// makes filling upfront affordable. Asking `static_indent` for each
    /// index instead costs the body's length per index, and made a
    /// 20,000-clause body placed after an `EXIT` go from 22 ms to 211 ms.
    pub(crate) indents: Box<[usize]>,
    /// The 1-based source line every instruction in this body sits on, by
    /// index -- empty for a body built without a source, which is the
    /// fragment case `build`'s own parameter documents.
    ///
    /// `indents` above, for the other constant of the source text the same
    /// upfront pass can know. `ProgramSource::line_of` is a `partition_point`
    /// over the line starts and `enter_stepped_clause` asks for the answer on
    /// **every** stepped clause, unconditionally, because `SIGL` has to stay
    /// correct whether or not `TRACE` is on -- so the search ran once per
    /// executed clause on both engines. Counted on this crate's own axes at
    /// `9b2d416d3`, identically under `REXX_ENGINE=ir` and under the
    /// tree-walker: 50,000,005 searches on `bench-programs/emptyloop.rex`,
    /// 57,000,006 on `varlookup.rex`, and 10,161,437 on
    /// `samples/rexxcps.rex`.
    ///
    /// **What the search cost tracks the body's line count**, measured on the
    /// same axes by dividing each axis's instruction saving by its own count
    /// above: 40.0 instructions per removed search on a 7-line program, 42.0
    /// on an 8-line one, about 53 on each of the 12-to-14-line ones, 75.3 on
    /// a 51-line one and 94.7 on `rexxcps`' 198 lines. The planning spike
    /// concluded the opposite from probe arms that returned deliberately
    /// wrong line numbers, whose long-program arm the plan itself disowns as
    /// contaminated; the depth is real and it is most of the spread.
    ///
    /// It changes nothing about the fix. A table is not chosen over a faster
    /// search because the search is shallow -- it is chosen because a search
    /// that is not made costs neither its call nor its depth, which is the
    /// upper bound any faster search would be measured against.
    ///
    /// **Valid only for the source it was built from**, which is what the
    /// `debug_assert_eq!` in [`Plan::line_at`] exists to hold: a plan is
    /// cached by `BodyKey`, and a table of line numbers reached with another
    /// program's source does not miss, it answers wrongly and silently --
    /// `SIGL`, every condition's reported line and every `*-*` trace line at
    /// once, in programs that raise nothing and trace nothing.
    pub(crate) lines: Box<[usize]>,
    /// How a compound-shaped symbol this body names splits, by
    /// `SymbolId::index`, `None` where this pass recorded nothing for it.
    ///
    /// The same move `indents` above is, for a different constant of the
    /// source text. `Interp::tail_key` used to call `compound_parts` on the
    /// interned name on **every** reference, re-splitting a string that
    /// cannot change: measured by `perf record` over `samples/rexxcps.rex`
    /// (`REXX_ENGINE=ir`, `count=100`/`averaging=100`, 999 Hz), that split
    /// was 3.38% of self time with the `CharSearcher` its `split('.')`
    /// drives at a further 3.80%, on a program whose innermost loop
    /// references `acompound.key1.loop`.
    ///
    /// The upfront pass already had the answer and discarded it:
    /// `note_compound_name` splits every compound name to assign its stem
    /// and its variable pieces slots. It now keeps the split.
    ///
    /// **An entry is an optimisation and never a requirement**, which is what
    /// makes filling this safe to reason about one call site at a time: a
    /// symbol with no entry splits its own spelling at the reference, exactly
    /// as every reference did before this field existed. `Code::compound` is
    /// where the two meet.
    ///
    /// **A `Vec` indexed by id rather than a `HashMap` keyed by one**, which
    /// is correct because `SymbolId::index` is dense and zero-based within
    /// the table that interned it, and affordable because the id space is one
    /// program's distinct symbols. Measured over the corpus, this crate's
    /// bench programs and the oracle's `samples/` tree -- 381 programs that
    /// parse -- the largest program-wide total of `symbols.len()` summed over
    /// a program's code bodies is 2,244 entries, and the sum over all 381 is
    /// 85,428.
    ///
    /// **Sized by the whole symbol table and not by the compounds in this
    /// body**, so a body that names few compounds still carries an entry per
    /// symbol. That is what buys the indexing: an id is only an index into
    /// the table that interned it, and any denser addressing would need a
    /// second map from id to position, which is the hash this replaces.
    pub(crate) compounds: Box<[Option<CompoundName>]>,
}

impl Plan {
    /// The static clause indent of `target`.
    ///
    /// `instructions` is a parameter rather than a field because a `Plan` is
    /// cached by `BodyKey`, and the body it describes is reached through the
    /// `Rc<Program>` every caller already holds. It is only consulted for a
    /// position this plan has no entry for, which a body of the length the
    /// table was built from cannot produce.
    pub(crate) fn indent_of(&self, instructions: &[Instruction], target: usize) -> usize {
        match self.indents.get(target) {
            Some(indent) => *indent,
            None => crate::run::static_indent(instructions, target),
        }
    }

    /// The 1-based source line `target`'s own clause starts on.
    ///
    /// `instruction` is `target`'s own instruction, which every caller
    /// already holds; it supplies the fallback's span for a position this
    /// plan has no entry for -- a body of the length the table was built from
    /// cannot produce one, and a plan built with no source has no entries at
    /// all.
    ///
    /// **The `debug_assert_eq!` is the whole safety argument for this field**
    /// and is not decoration. A cached line that is merely absent falls back
    /// and is right; a cached line that is *wrong* misreports `SIGL`, every
    /// condition's line and every `*-*` trace line, in programs that observe
    /// none of the three and so cannot fail a differential. Asserting the
    /// cache against a recomputation at the accessor is what turns that into
    /// a debug-build failure at the first clause.
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
    ///
    /// **`id` must belong to the `SymbolTable` this plan was built against**,
    /// and nothing here can check that: an id from another table is either out
    /// of range, which answers `None`, or in range, which answers another
    /// symbol's entry with no complaint (`SymbolId::index`'s own doc comment
    /// on the two ways that goes wrong). `Code` is what pairs a plan with the
    /// table whose ids index it, and `Code::compound` is the only caller.
    /// Whether nothing this body runs can change the `TRACE` setting while it
    /// runs. See the field for what falsifies it and why `false` is safe.
    pub(crate) fn never_retraces(&self) -> bool {
        self.never_retraces
    }

    pub(crate) fn compound(&self, id: SymbolId) -> Option<&CompoundName> {
        self.compounds.get(id.index())?.as_ref()
    }

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
    ///
    /// **`source` is `None` for a body with no source of its own to index**,
    /// and the only such caller is `fragment_plan`: an `INTERPRET` fragment's
    /// clauses all report the enclosing `INTERPRET` clause's line through
    /// `Interp::clause_line_override`, `Code::plan` is `None` for a fragment
    /// so nothing would read the table, and a fragment's plan is discarded
    /// with the fragment. Filling it there would be a table built once per
    /// execution of the `INTERPRET` and read never.
    pub(crate) fn build(
        body: &CodeBody,
        symbols: &SymbolTable,
        source: Option<&ProgramSource>,
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
        //
        // The interpreter assigns all three itself, so leaving them out of the
        // plan does not mean they never get a slot -- it means they get one
        // from `Interp::slot_of`'s third source, which grows the frame and
        // records the name in `Activation::extra`. That map is cloned into
        // every callee, so a single `CALL` whose routine returns a value made
        // every later call allocate a map and copy a key, for a name the
        // program never wrote. Measured with the marginal method: `call sub`
        // against a routine ending `return 1` cost 4420.7 user instructions
        // per call with `RESULT` reaching `extra`, and 4053.0 with the body
        // mentioning `RESULT` so the plan held it already.
        plan.result_slot = Some(plan.slot_for(b"RESULT"));
        plan.slot_for(b"RC");
        plan.sigl_slot = Some(plan.slot_for(b"SIGL"));
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
    ///
    /// `label` is deliberately not bound: a loop label names the block for
    /// `LEAVE`/`ITERATE`/`END`, not a data variable, exactly like `Select`'s
    /// own `label` in `note_instruction`.
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
    ///
    /// Registers by name alone, with no slot bound to `id`: neither the stem
    /// prefix nor a tail piece has a `SymbolId` of its own --
    /// `compound_parts` only ever hands back a borrowed slice of the one
    /// interned spelling a `Compound` id carries, and a piece was never a
    /// token the scanner saw (`ast.rs`'s own `Compound` doc comment says so).
    /// That is what makes `names`, not `by_symbol`, the correct table for
    /// them to land on -- and the slot each one lands on is kept on the entry
    /// rather than dropped, so that a reference indexes the frame instead of
    /// hashing the name again.
    ///
    /// `id` is the whole compound's own id, and it addresses `compounds`
    /// rather than being bound to a slot. Every caller has one: `note`'s
    /// `ExprKind::Compound` arm, and `note_variable_ref` for either shape of
    /// `VariableRef`. In the `Indirect` case that id names the wrapper
    /// variable, which reaches here only when the wrapper is *itself*
    /// compound-shaped (`DROP (a.b)`) -- and then `run.rs`'s own
    /// `drop_variable` reads it through `tail_key` under this same id, so
    /// the entry is addressed by the id the reader will present.
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
    ///
    /// Both views are updated together because they are one assignment seen
    /// two ways: a second symbol spelling the same name must land on the slot
    /// the first one got, which is why the slot number comes from `names` and
    /// never from `by_symbol`'s length.
    ///
    /// **A compound-shaped spelling also gets its split recorded**, and here
    /// rather than at the call sites that can produce one, so that recording
    /// it is a property of binding a name at all. A `DO` control variable is
    /// the shape that makes this worth doing: `do a.i = 1 to 5` binds one
    /// whole dotted symbol, and `run.rs`'s controlled-loop step then resolves
    /// its tail through `tail_key` on **every pass**, which is exactly the
    /// per-reference split `compounds` exists to remove.
    ///
    /// **No new name is assigned a slot here, which is what separates this
    /// from `note_compound_name`**: `name` is bound whole above, and adding
    /// slots for its parts would move every later slot number in the body -- a
    /// change to frame layout, not to how a name splits. So the entry this
    /// writes has `at: None` on every variable piece, and `Interp::join_tails`
    /// resolves those by name.
    ///
    /// **`stem_at` is the exception, and it assigns nothing new.** A
    /// stem-shaped `name` has no part that is not itself: its stem half is the
    /// whole spelling, so the slot already bound to `id` above is the stem's
    /// slot and recording it costs no name and moves no layout. A
    /// compound-shaped `name` does have parts, so its stem stays `None` and
    /// `Interp::stem_slot` resolves it -- `do a.i = 1 to 5` binds `A.I` and
    /// nothing called `A.`.
    ///
    /// **An entry already recorded is left alone**, which matters when one id
    /// reaches `note_compound_name` as well -- `say v.i` and then `do v.i = 1
    /// to 2` name one symbol. Either entry holds the identical split, since
    /// they split the identical spelling, and they differ only in whether the
    /// pieces carry slots; overwriting would therefore change no answer and
    /// would throw away `note_compound_name`'s slots for every reference in
    /// the body, in whichever order the pass happened to reach them.
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
    ///
    /// The compile-time half of `Code::slot_for`, which carries the tripwire:
    /// this side holds no symbol table, so it has no name to check the answer
    /// against. Both read the one table, so the compiled answer and the
    /// run-time one stay one resolution made at two times.
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
    ///
    /// **`source` must be the source of the program `key` names**, and
    /// nothing here can check it -- the plan is cached under `key` and handed
    /// back to whatever asks for that key next, so a table of line numbers
    /// built from another program's text would be answered with no complaint.
    /// The two production callers both take the body, the symbols and the
    /// source out of one `Rc<Program>` reached through the same id `key`
    /// carries (`InstalledRoutine`'s own doc comment has that argument for
    /// the routine caller), and `Plan::line_at`'s `debug_assert_eq!` is what
    /// holds it at the read rather than at the build.
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
        let plan = Rc::new(Plan::build(body, symbols, Some(source)));
        self.plans.insert(key, Rc::clone(&plan));
        plan
    }

    /// The chunk for one body **under one trace setting**, from the cache or
    /// compiled and cached (D16's discipline, with the key D23 widens it by).
    ///
    /// **`BodyKey` alone does not identify a chunk and keying on it alone is a
    /// wrong-output defect, not a slow one.** The trace setting is an input to
    /// compilation (D23): it decides which clause echoes are emitted as ops,
    /// so one body compiles to two different streams under two settings, and a
    /// lookup that ignored the setting would hand back whichever was compiled
    /// first -- a body entered untraced and then under `trace i` would run the
    /// untraced stream the second time. `ChunkTrace` is exactly what
    /// `crate::ir::compile` reads, and `compile` takes nothing else, so the
    /// key cannot come to be narrower than the thing it names.
    ///
    /// Widened rather than evicted, because eviction throws away the chunk a
    /// program that toggles `TRACE` is about to want again: two settings mean
    /// two entries here and two compiles for the whole run, where eviction
    /// means one compile per change.
    ///
    /// `None` means the body does not fit the index widths and this
    /// activation runs on the tree-walker. `chunks_refused` counts that, once
    /// per refusal: only a compiled chunk is cached, so a refused body comes
    /// back here and is refused again the next time it is entered. That
    /// counter is what stops the fallback being silent -- the dual-engine
    /// harness asserts it is zero across the corpus.
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
    pub(crate) fn fragment_plan(&mut self, fragment: &Fragment) -> Vec<Option<usize>> {
        // The same upfront pass, run against the fragment's own body, which
        // numbers its names 0..n in walk order. Those numbers are local to the
        // fragment and mean nothing to the enclosing frame; the loop below is
        // what translates them.
        let local = Plan::build(&fragment.body, &fragment.symbols, None);

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
            .map(|entry| entry.map(|local_slot| enclosing[local_slot]))
            .collect()
    }
}

/// Whether a call names the `TRACE()` builtin, which sets the running
/// activation's own `TRACE` setting where every other call cannot.
///
/// Case-insensitive because the two spellings reach here differently: a bare
/// symbol arrives upcased, and `"trace"(...)` arrives exactly as written --
/// which never resolves to the builtin, so answering `true` for it costs the
/// optimisation on that body and nothing else.
fn names_trace(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"TRACE")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Novalue, planned_code};
    use rexx_parse::{Program, parse_interpret, parse_program};

    /// `indent_of` answers what `static_indent` answers, for every index.
    ///
    /// The table is filled by a separate traversal (`all_indents`), and
    /// `all_indents_fills_what_static_indent_computes_for_every_corpus_program`
    /// is what holds that traversal to this one. What this adds is the
    /// wiring in between: an off-by-one in which entry an index reads gives
    /// one instruction another's indent, and every arm of the program below
    /// has a distinct value.
    #[test]
    fn indent_of_answers_what_static_indent_answers_at_every_index() {
        let source = b"if 1 = 1 then\n  do i = 1 to 2\n    say i\n  end\nelse\n  nop\nselect\n  when 1 = 0 then nop\n  otherwise\n    say 'o'\nend\n";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
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
    ///
    /// `line_at`'s own `debug_assert_eq!` compares the two on every read, so
    /// most of what this test could assert is asserted by the accessor
    /// already -- what it adds is the table itself. An accessor that quietly
    /// recomputed, or a `build` that filled nothing, would satisfy every
    /// caller and leave the field empty, and only reading `plan.lines`
    /// directly says which of the two happened.
    ///
    /// Every clause below is on a line of its own, so a table shifted by one
    /// entry answers a different number at every index rather than at some.
    #[test]
    fn line_at_answers_what_line_of_answers_at_every_index() {
        let source = b"nop\nsay 1\nif 1 = 1 then\n  nop\nelse\n  nop\ndo i = 1 to 2\n  say i\nend\nsay 'done'\n";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
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
        let plan = Plan::build(&program.main, &program.symbols, None);
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
    ///
    /// This is the wiring the two accessors meet through, and the only place
    /// the override's precedence over the table is stated as an assertion.
    /// A table consulted *before* the override would give a fragment's
    /// clauses the fragment's own line numbers instead of the enclosing
    /// `INTERPRET` clause's -- which is a wrong `SIGL` and a wrong reported
    /// line, in a construct the corpus exercises and no arithmetic notices.
    #[test]
    fn clause_line_at_answers_what_clause_line_answers_and_the_override_still_wins() {
        let source = b"nop\nsay 1\nif 1 = 1 then\n  nop\nelse\n  nop\ndo i = 1 to 2\n  say i\nend\nsay 'done'\n";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
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
    ///
    /// The counterpart of
    /// `all_indents_fills_what_static_indent_computes_for_every_corpus_program`
    /// for the other constant of the source text, and it exists for the same
    /// reason: `build`'s fill and the accessor's fallback are two spellings
    /// of one rule, and a transcription can be wrong where the original is
    /// right. The corpus rather than examples written here, because it grows
    /// when a construct lands.
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
                let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
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
            let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
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
            let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));

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

    /// `build` records a compound's split under **the compound's own id**.
    ///
    /// The rows are a compound written as an expression, as a `DROP` target
    /// and as a `DO` control variable, which is one row per filler they
    /// reach: `note_compound_name` for the first two (through `note` and
    /// through `note_variable_ref`) and `bind` for the third (through
    /// `note_loop`). `note_variable_ref` also carries an `EXPOSE`,
    /// `PROCEDURE EXPOSE` and `USE LOCAL` target, and `bind` also carries a
    /// `PARSE VAR` source (`note_parse`) -- which is why the control
    /// variable's fix went into `bind` rather than beside `note_loop`'s own
    /// call, and measured, `i = 3; a.i = 'p q'; parse var a.i x y; say x y`
    /// prints `p q` on the oracle and on both engines.
    ///
    /// **What this catches that no output comparison can: an entry that is
    /// never written.** `Code::compound` falls back to splitting the
    /// spelling and hands back the identical pieces, so a filler that stops
    /// running leaves every corpus program and every `tail_key` assertion
    /// green. Only reading the table back says so. The pieces are spelled
    /// out here rather than compared against `CompoundName::split`, which
    /// would be the same function on both sides of the assertion.
    ///
    /// **An entry written under the wrong id is a different failure, and not
    /// what this test is needed for.** Nothing falls back for one: the table
    /// answers, with another symbol's split, so it is a wrong answer rather
    /// than a slow path. Measured, shifting every entry one id along reddens
    /// output-level tests in `run/tests.rs` and the corpus sweep as well as this
    /// one.
    ///
    /// The control-variable row is the one that was measured wrong: before
    /// `bind` recorded a split, `do aa.ii = 1 to 2` reached `tail_key` with
    /// no entry and re-split its name on every pass.
    ///
    /// **The slots are part of what is asserted, and `note_compound_name` and
    /// `bind` disagree about them.** `note_compound_name` puts the stem and
    /// each variable piece on the slot it assigned that name; `bind` assigns
    /// no name a slot beyond the one it binds whole, so `AA.II`'s stem and
    /// piece both carry `None` while `DD.JJ`'s and `V.I.7`'s carry a number.
    /// (A stem-*shaped* name bound by `bind` does carry one, because there the
    /// name bound whole and the stem are the same name;
    /// `bind_keeps_a_stem_shaped_names_own_slot_as_its_stems` is that pair.)
    /// The numbers are spelled out rather than looked back up out of
    /// `plan.names`, which would be the same map on both sides of the
    /// assertion.
    #[test]
    fn build_records_a_compounds_split_under_the_compounds_own_id() {
        let source = b"drop dd.jj; do aa.ii = 1 to 2; say v.i.7; end";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));

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
    ///
    /// `say v.i` takes `V.I` through `note_compound_name`, which assigns `V.`
    /// and `I` slots; `do v.i = 1 to 2` then takes the **same** id through
    /// `bind`, which assigns none. An overwrite there would leave the body's
    /// every reference to `V.I` resolving its piece by name again, and change
    /// no answer while doing it -- so nothing that compares output can see
    /// this, and only the entry says so.
    #[test]
    fn a_control_variable_does_not_take_the_slots_off_a_compound_already_seen() {
        let source = b"say v.i; do v.i = 1 to 2; end";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));

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
    ///
    /// `do v.i = 1 to 2` reaches `bind` first and records a slotless entry;
    /// the `say v.i` after the loop then reaches `note_compound_name`, whose
    /// write is **unconditional** and is what replaces that entry with the
    /// slotted one. Its neighbour above covers the reverse order, where
    /// `bind`'s `get_or_insert_with` is what preserves the slots -- between
    /// them the two rules are pinned in the direction each one decides.
    /// Neither order can be seen from output: both end with a key resolved
    /// the same way, and a piece with no slot answers identically through
    /// `read_by_name`.
    #[test]
    fn a_compound_seen_after_the_control_variable_still_gets_its_slots() {
        let source = b"do v.i = 1 to 2; end; say v.i";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));

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
    ///
    /// `do za.zi = 1 to 3` binds the whole dotted `ZA.ZI` to one slot and
    /// binds neither `ZA.` nor `ZI`, so the plan has no name `ZI` at all.
    /// Resolving the piece at run time therefore misses the plan, misses
    /// `extra`, and grows the frame -- recording `ZI` in `extra`, which is
    /// where every later pass of the loop finds it. Measured on an
    /// interpreter instrumented to print each growth: the loop above grows
    /// `ZI` once and hits `extra` for it on every pass.
    ///
    /// That is why a piece's slot is an `Option` and not a `usize`. It is
    /// also why a precomputed slot cannot shadow an `extra` binding: a slot
    /// is put on a piece by `slot_for`, which is what puts the name in
    /// `plan.names`, and `Interp::slot_of` reads `plan.names` before `extra`
    /// -- so a piece either carries a slot and never consults `extra`, or
    /// carries none and resolves exactly as it did before slots existed.
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

        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
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
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
        let code = planned_code(&program, &plan);
        // Unset, so the piece derives its own spelling -- the ordinary
        // uninitialised read, reached here through `extra` and growth.
        let key = interp.tail_key(&code, id);
        assert_eq!(key, b"ZI");
        assert!(
            interp.activation().extra.contains_key(b"ZI".as_slice()),
            "the piece must be recorded in extra, which is the source a \
             precomputed slot would have skipped"
        );
    }

    /// **A compound's stem can be bound in `extra` rather than in the plan,
    /// and this is the shape that reaches it.**
    ///
    /// The stem half of the question its neighbour above answers for a tail
    /// piece, and the answer is the same: `do za.zi = 1 to 3` binds the whole
    /// dotted `ZA.ZI` to one slot and binds neither `ZA.` nor `ZI`, so the
    /// plan has no name `ZA.` at all. The loop's own read therefore misses
    /// the plan, misses `extra`, and grows the frame -- recording `ZA.` in
    /// `extra`, which is where every later pass finds it. Measured on an
    /// interpreter instrumented to print each growth: that loop grows `ZA.`
    /// once and hits `extra` for it on every pass, alongside `ZI`.
    ///
    /// That is why the stem's slot is an `Option` and not a `usize`. It is
    /// also why a precomputed one cannot shadow an `extra` binding: a slot is
    /// put on the stem by `slot_for`, which is what puts the name in
    /// `plan.names`, and `Interp::slot_of` reads `plan.names` before `extra`
    /// -- so a stem either carries a slot and never consults `extra`, or
    /// carries none and resolves exactly as it did before slots existed.
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

        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
        assert_eq!(
            plan.slot_of(b"ZA."),
            None,
            "the plan binds the whole ZA.ZI and nothing named ZA."
        );
        assert_eq!(expect_entry(&plan, id).stem_at, None);

        let program = activate(&mut interp, program);
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
        let code = planned_code(&program, &plan);
        let (stem_name, stem_at) = code.stem(id);
        assert_eq!(stem_name, b"ZA.");
        assert_eq!(stem_at, None);

        // Nothing has been written, so the tail derives its own name from the
        // read site's spelling -- the ordinary uninitialised compound read,
        // reached here through `extra` and growth.
        let key = interp.tail_key(&code, id);
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
    ///
    /// Both go through `bind` -- the assignment target and both `DO` control
    /// variables -- and the split is recorded for all three. `ZT.` and `ZS.`
    /// have no stem half distinct from themselves, so the slot bound to the
    /// symbol *is* the stem's and the entry keeps it. `AA.II` does: `bind`
    /// binds the whole dotted name to one slot and nothing called `AA.`, so
    /// its stem carries `None` and `Interp::stem_slot` resolves it at every
    /// reference.
    ///
    /// The numbers are spelled out rather than looked back up out of
    /// `plan.names`, which is the map `slot_for` wrote them from and would be
    /// the same map on both sides of the assertion. They are the pass's own,
    /// in the order it assigned them: `ZT.` for the assignment, `ZS.` for the
    /// first control variable, `AA.II` whole for the second.
    #[test]
    fn bind_keeps_a_stem_shaped_names_own_slot_as_its_stems() {
        let source = b"zt. = 'v'\ndo zs. = 1 to 2\nnop\nend\ndo aa.ii = 1 to 2\nnop\nend";
        let program = parse_program(source.to_vec()).expect("test program parses");
        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));

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

        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
        let code = planned_code(&program, &plan);
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

        let plan = Plan::build(&program.main, &program.symbols, Some(&program.source));
        let code = planned_code(&program, &plan);
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
        // The table is sized by the fragment's own symbol table and is mostly
        // `None`; what the fragment *names* is the count of bound entries.
        assert_eq!(
            slots.iter().flatten().count(),
            1,
            "the fragment names exactly one variable"
        );

        let enclosing_slot = interp.slot_of(b"NEWVAR");
        let fragment_slot = slots.iter().flatten().next().copied().expect("one entry");
        assert_eq!(
            fragment_slot, enclosing_slot,
            "the fragment's own id must resolve to the SAME slot the \
             enclosing body would use for the same name"
        );
    }
    /// **What may change the `TRACE` setting under a running chunk**, which is
    /// the whole of what lets `ir::compile` decide a body's value echoes once
    /// instead of gating each one.
    ///
    /// Each blocking route is paired with the nearest body that does *not*
    /// block, because a guard that answered "may retrace" for everything would
    /// satisfy the first half alone and cost only the optimisation -- silently.
    #[test]
    fn a_body_that_can_reach_the_trace_setting_is_the_one_that_says_so() {
        let retraces = |source: &[u8]| {
            let program = parse_program(source.to_vec()).expect("test program parses");
            !Plan::build(&program.main, &program.symbols, None).never_retraces()
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
