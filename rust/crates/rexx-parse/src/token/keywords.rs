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

//! The pre-interned keyword tables.

use super::{SymbolId, SymbolTable};

/// One table: the interned spellings, in the order the C++ table lists them,
/// so a hit yields that table's own index and the caller maps the index to its
/// own enum.
#[derive(Debug)]
pub struct KeywordSet {
    ids: Vec<SymbolId>,
}

impl KeywordSet {
    /// Interns every spelling in `names`, keeping their order.
    fn new(symbols: &mut SymbolTable, names: &[&str]) -> Self {
        KeywordSet {
            ids: names.iter().map(|n| symbols.intern(n)).collect(),
        }
    }

    /// The table index of `id`, or `None` if `id` is not in this set.
    pub fn index_of(&self, id: SymbolId) -> Option<usize> {
        self.ids.iter().position(|&k| k == id)
    }

    /// How many spellings this table holds.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether this table is empty, which no real table is.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// The pre-interned spelling tables. Built by `scan` before it reads any
/// source, so a keyword test never hashes a string.
#[derive(Debug)]
pub struct Keywords {
    pub instructions: KeywordSet,
    pub sub_keywords: KeywordSet,
    pub conditions: KeywordSet,
    pub parse_options: KeywordSet,
    pub directives: KeywordSet,
    pub sub_directives: KeywordSet,
}

impl Keywords {
    /// Interns all six tables into `symbols`.
    pub fn new(symbols: &mut SymbolTable) -> Self {
        Keywords {
            instructions: KeywordSet::new(symbols, &INSTRUCTIONS),
            sub_keywords: KeywordSet::new(symbols, &SUB_KEYWORDS),
            conditions: KeywordSet::new(symbols, &CONDITIONS),
            parse_options: KeywordSet::new(symbols, &PARSE_OPTIONS),
            directives: KeywordSet::new(symbols, &DIRECTIVES),
            sub_directives: KeywordSet::new(symbols, &SUB_DIRECTIVES),
        }
    }
}

// The six tables, copied from `KeywordConstants.cpp` without reordering. The
// C++ keeps them in ASCII order so it can binary-search them; here the order
// is load-bearing for a different reason, since `index_of` returns a position
// that the caller maps to its own enum.

const INSTRUCTIONS: [&str; 35] = [
    "ADDRESS",
    "ARG",
    "CALL",
    "DO",
    "DROP",
    "ELSE",
    "END",
    "EXIT",
    "EXPOSE",
    "FORWARD",
    "GUARD",
    "IF",
    "INTERPRET",
    "ITERATE",
    "LEAVE",
    "LOOP",
    "NOP",
    "NUMERIC",
    "OPTIONS",
    "OTHERWISE",
    "PARSE",
    "PROCEDURE",
    "PULL",
    "PUSH",
    "QUEUE",
    "RAISE",
    "REPLY",
    "RETURN",
    "SAY",
    "SELECT",
    "SIGNAL",
    "THEN",
    "TRACE",
    "USE",
    "WHEN",
];

const SUB_KEYWORDS: [&str; 50] = [
    "ADDITIONAL",
    "APPEND",
    "ARG",
    "ARGUMENTS",
    "ARRAY",
    "BY",
    "CASE",
    "CLASS",
    "CONTINUE",
    "COUNTER",
    "DESCRIPTION",
    "DIGITS",
    "ENGINEERING",
    "ERROR",
    "EXIT",
    "EXPOSE",
    "FALSE",
    "FOR",
    "FOREVER",
    "FORM",
    "FUZZ",
    "INDEX",
    "INHERIT",
    "INPUT",
    "ITEM",
    "LABEL",
    "LOCAL",
    "MESSAGE",
    "NAME",
    "NOINHERIT",
    "NORMAL",
    "OFF",
    "ON",
    "OUTPUT",
    "OVER",
    "REPLACE",
    "RETURN",
    "SCIENTIFIC",
    "STEM",
    "STREAM",
    "STRICT",
    "THEN",
    "TO",
    "TRUE",
    "UNTIL",
    "USING",
    "VALUE",
    "WHEN",
    "WHILE",
    "WITH",
];

const CONDITIONS: [&str; 12] = [
    "ANY",
    "ERROR",
    "FAILURE",
    "HALT",
    "LOSTDIGITS",
    "NOMETHOD",
    "NOSTRING",
    "NOTREADY",
    "NOVALUE",
    "PROPAGATE",
    "SYNTAX",
    "USER",
];

const PARSE_OPTIONS: [&str; 10] = [
    "ARG", "CASELESS", "LINEIN", "LOWER", "PULL", "SOURCE", "UPPER", "VALUE", "VAR", "VERSION",
];

const DIRECTIVES: [&str; 9] = [
    "ANNOTATE",
    "ATTRIBUTE",
    "CLASS",
    "CONSTANT",
    "METHOD",
    "OPTIONS",
    "REQUIRES",
    "RESOURCE",
    "ROUTINE",
];

const SUB_DIRECTIVES: [&str; 40] = [
    "ABSTRACT",
    "ALL",
    "ATTRIBUTE",
    "CLASS",
    "CONDITION",
    "CONSTANT",
    "DELEGATE",
    "DIGITS",
    "END",
    "ERROR",
    "EXTERNAL",
    "FAILURE",
    "FORM",
    "FUZZ",
    "GET",
    "GUARDED",
    "INHERIT",
    "LIBRARY",
    "LOSTDIGITS",
    "METACLASS",
    "METHOD",
    "MIXINCLASS",
    "NAMESPACE",
    "NOPROLOG",
    "NOSTRING",
    "NOTREADY",
    "NOVALUE",
    "NUMERIC",
    "PACKAGE",
    "PRIVATE",
    "PROLOG",
    "PROTECTED",
    "PUBLIC",
    "ROUTINE",
    "SET",
    "SUBCLASS",
    "SYNTAX",
    "TRACE",
    "UNGUARDED",
    "UNPROTECTED",
];
