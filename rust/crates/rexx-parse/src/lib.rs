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

mod scanner;
mod source;
mod token;

pub use scanner::{ResourceBody, Scanned, scan};
pub use source::{ProgramSource, SourceKind};
pub use token::{
    KeywordSet, Keywords, Operator, ParseCtx, ParseError, SymbolClass, SymbolId, SymbolTable, Tag,
    Token, TokenCursor, TokenKind,
};
