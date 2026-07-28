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
//!
//! The behaviour reproduced here was measured against `build/bin/rexx`
//! rather than taken from the ANSI standard; where they differ, the
//! interpreter wins.

// Phase 3 builds this crate bottom up, one layer per task, so a layer's entry
// point has no non-test caller until the layer above it lands. `cargo clippy
// --all-targets` compiles the library once with `cfg(test)` off, and there each
// such item is dead. The `#[allow(dead_code)]` attributes below and in
// `clause.rs`, `expr.rs` and `token.rs` mark exactly those items, each naming
// the task that will call it and so delete the attribute. There is no
// crate-wide allow, deliberately: a blanket one would also hide code that is
// dead by mistake.

mod ast;
mod clause;
mod expr;
mod scanner;
mod source;
mod token;

pub use ast::{CallTarget, Expr, ExprKind, PrefixOp, Tail, compound_parts};
pub use scanner::{ResourceBody, Scanned, scan};
pub use source::{ProgramSource, SourceKind};
pub use token::{
    KeywordSet, Keywords, Operator, ParseError, SymbolClass, SymbolId, SymbolTable, Tag, Token,
    TokenKind,
};
