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

//! The Rexx parser: source retention, scanning, and the grammar.

// Phase 3 built this crate bottom up, one layer per task, so a layer's entry
// point had no non-test caller until the layer above it landed. `cargo clippy
// --all-targets` compiles the library once with `cfg(test)` off, and there each
// such item was dead.

mod ast;
mod block;
mod clause;
mod convert;
mod directive;
mod error;
mod expr;
mod instruction;
mod scanner;
mod selector;
mod source;
mod token;

pub use ast::{
    Access, Address, AddressIo, Annotate, Annotation, AnnotationTarget, AttributeDirective,
    AttributeStyle, Call, CallTarget, ClassDirective, ClassRef, CodeBody, ConditionOption,
    ConditionTrap, ConstantDirective, ConstantValue, ControlExpr, Controlled, Directive,
    DirectiveKind, EndStyle, EndTarget, Expr, ExprKind, ExternalSpec, Forward, Guard, GuardOption,
    Instruction, InstructionKind, Loop, LoopConditional, LoopKind, MethodDirective, NumericSetting,
    OptionsForm, OutputOption, PackageOption, Parse, ParseSource, ParseTrigger, PrefixOp,
    Protection, Raise, RaiseResult, Redirection, Requires, Resource, RoutineDirective, Signal,
    Tail, Trace, TriggerKind, Use, UseTarget, VariableRef, compound_parts,
};
/// The parser's own nesting limit, exported because a caller has to be able
/// to reason about it: it decides which inputs come back as `11.1` rather than
/// as an AST, and it is the number a test or an embedder checks its own depths
/// against rather than hardcoding 50,000 in two places.
pub use expr::MAX_EXPR_DEPTH;
pub use scanner::{ResourceBody, Scanned, scan};
pub use selector::Selector;
pub use source::{ProgramSource, SourceKind};
pub use token::{
    KeywordSet, Keywords, Operator, ParseError, SymbolClass, SymbolId, SymbolTable, Tag, Token,
    TokenKind,
};

use crate::block::translate_block;
use crate::clause::{ClauseCursor, split_clauses};
use crate::directive::parse_directive;
use crate::selector::SelectorTable;
use crate::token::ParseCtx;

/// A whole program: everything `translate` produces from one source buffer
/// (`LanguageParser.cpp:735`-`765`).
#[derive(Debug)]
pub struct Program {
    pub source: ProgramSource,
    /// The main code body. A directive's own body is a `CodeBody` too, held
    /// inside `directives` rather than here.
    pub main: CodeBody,
    /// The `::` directives, in source order, each carrying its own assembled
    /// body.
    pub directives: Vec<Directive>,
    /// Retained because a `SymbolId` is meaningless without it: Phase 4
    /// resolves names back to text to report them.
    pub symbols: SymbolTable,
}

/// What `INTERPRET` produces: one code body, parsed at *run time* rather than
/// at build time, from the string an `INTERPRET` instruction is about to run.
#[derive(Debug)]
pub struct Fragment {
    pub source: ProgramSource,
    pub body: CodeBody,
    pub symbols: SymbolTable,
}

/// Parses a whole program from `text`.
pub fn parse_program(text: Vec<u8>) -> Result<Program, ParseError> {
    let source = ProgramSource::new(text, SourceKind::Program);
    let parsed = parse(&source)?;
    Ok(Program {
        source,
        main: parsed.main,
        directives: parsed.directives,
        symbols: parsed.symbols,
    })
}

/// Parses a program whose physical lines are given one per element.
pub fn parse_lines(lines: &[&[u8]]) -> Result<Program, ParseError> {
    let source = ProgramSource::from_lines(lines);
    let parsed = parse(&source)?;
    Ok(Program {
        source,
        main: parsed.main,
        directives: parsed.directives,
        symbols: parsed.symbols,
    })
}

/// Parses the string an `INTERPRET` instruction is about to run.
pub fn parse_interpret(text: Vec<u8>) -> Result<Fragment, ParseError> {
    let source = ProgramSource::new(text, SourceKind::Interpret);
    let parsed = parse(&source)?;
    debug_assert!(
        parsed.directives.is_empty(),
        "INTERPRET text cannot carry a directive: `parse` raises 99.914 first"
    );
    // Measured in Task 1: a label inside `INTERPRET` text is 47.1, already
    // enforced where `parse_instruction` builds a `Label` node, so this map is
    // always empty rather than load-bearing.
    debug_assert!(
        parsed.main.labels.is_empty(),
        "INTERPRET text cannot carry a label: `parse_instruction` raises 47.1 first"
    );
    Ok(Fragment {
        source,
        body: parsed.main,
        symbols: parsed.symbols,
    })
}

/// What one parse produces, before it is split into `Program` or `Fragment`.
struct Parsed {
    /// The main code body. A directive's body is inside the directive.
    main: CodeBody,
    directives: Vec<Directive>,
    symbols: SymbolTable,
}

/// The composition shared by both entry points.
fn parse(source: &ProgramSource) -> Result<Parsed, ParseError> {
    let scanned = scan(source)?;
    // Declared ahead of `ctx`, which borrows it, and dropped with the parse.
    // A `Selector` in the tree holds the name's bytes itself, so what the
    // pool's own entry buys is finding a spelling again while the parse is
    // still reading names -- which is exactly as long as it lives.
    let selectors = std::cell::RefCell::new(SelectorTable::new());
    let ctx = ParseCtx {
        source,
        tokens: &scanned.tokens,
        symbols: &scanned.symbols,
        selectors: &selectors,
        keywords: &scanned.keywords,
        resources: &scanned.resources,
    };

    let mut cursor = ClauseCursor::new(split_clauses(ctx.tokens)?);
    let main = translate_block(&ctx, &mut cursor)?;

    // `translate` raises 99.914 exactly here, once, before `nextDirective` is
    // ever called (`LanguageParser.cpp:1113`-`1120`): `INTERPRET` text may not
    // carry a directive at all, so this is not a per-directive check. Measured
    // via a `signal on syntax` trap around `interpret "::routine r"`:
    // `condition('o')~code` is `99.914` with message "INTERPRET data must not
    // contain directive instructions."
    if let Some(clause) = cursor.peek()
        && source.kind() == SourceKind::Interpret
    {
        return Err(ParseError::new(99, 914, clause.span.start));
    }

    let mut directives = Vec::new();
    while cursor.peek().is_some() {
        let mut directive = parse_directive(&ctx, &mut cursor)?;
        // Only `::METHOD`/`::ATTRIBUTE`/`::ROUTINE` can carry a body, and each
        // already rejects one with its own specific error when its OWN shape
        // does not allow it (`::CONSTANT`'s body is 99.938, for one). Every
        // other directive kind never sets a body at all, and a clause trailing
        // one of those needs no special case here: the NEXT iteration's
        // `parse_directive` call sees a clause that does not start with `::` and
        // raises 99.916 on its own -- measured for all five kinds that can never
        // have a body (`::CLASS`, `::OPTIONS`, `::REQUIRES`, `::ANNOTATE`,
        // `::RESOURCE`).
        if let Some(slot) = directive_body(&mut directive.kind) {
            *slot = translate_block(&ctx, &mut cursor)?;
        }
        directives.push(directive);
    }

    Ok(Parsed {
        main,
        directives,
        symbols: scanned.symbols,
    })
}

/// The body slot of a directive that carries one, for the assembler to fill.
pub(crate) fn directive_body(kind: &mut DirectiveKind) -> Option<&mut CodeBody> {
    match kind {
        DirectiveKind::Method(method) => method.body.as_mut(),
        DirectiveKind::Attribute(attribute) => attribute.body.as_mut(),
        DirectiveKind::Routine(routine) => routine.body.as_mut(),
        DirectiveKind::Annotate(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_) => None,
    }
}

// The label table used to be built here, in a pass over the finished
// instruction list. It is built by `Block::add_clause` now, which is where
// `addLabel` sits in the C++, because a body's labels have to be its own and
// only the assembler knows where one body ends and the next begins.
