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

//! The expression tree.
//!
//! One variant per expression class under `interpreter/expression/`, with the
//! collapses noted at each site.
//!
//! # Why a tree of owned children, where instructions are a flat chain
//!
//! D13 puts the AST in one arena per code body, and Task 3.1 Step 3b made that
//! arena a `Vec<Instruction>` with nesting expressed as indices, because
//! `SIGNAL`, `ITERATE`, `LEAVE` and `END` all jump *into* the instruction
//! sequence and an index is what a jump target is.
//!
//! Expressions have no such jumps. Nothing branches into the middle of an
//! expression, so an index would buy nothing an owned child does not already
//! give, and it would cost a second arena plus an untyped index that no
//! borrow check covers. So an `Expr` owns its children through `Box` and
//! `Vec`, and an `Instruction` owns its `Expr`s inline. The whole body still
//! lives in one arena object, which is what D13 asks for. Only the shape
//! inside an instruction differs, and it differs because the reason for the
//! chain does not apply here.
//!
//! # The span invariant
//!
//! Every node carries a byte range into the retained source, and a node's
//! range contains every one of its children's ranges. That holds by
//! construction rather than by discipline: `Expr::new` widens the extent it is
//! given to cover each child before storing it, so a caller that computes an
//! extent too narrowly gets a correct span anyway.
//!
//! Parentheses are the one thing that does not appear here. `parse_subterm`
//! returns the parenthesised expression itself, exactly as the C++ does, so
//! `(a)` and `a` give the same node and the node's span covers `a` alone. A
//! node's span is the extent of the tokens that *built* that node, and a
//! `Variable` node built from one symbol token must not claim a range holding
//! anything else, or the source spelling recovered from it would be wrong.

use std::ops::Range;

use crate::token::{Operator, SymbolId};
// Only `shape` needs the table, to turn a `SymbolId` back into a spelling, and
// `shape` renders trees for test assertions.
#[cfg(test)]
use crate::token::SymbolTable;

/// One expression node: what it is, and the source it came from.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    /// Byte range in the retained source. Contains every child's range.
    pub span: Range<usize>,
}

/// The prefix operators.
///
/// `+`, `-` and `\` are the only three (`LanguageParser.cpp:3644`). A prefix
/// `>` or `<` is not an operator at all: it builds a `VariableReference`.
///
/// These have no entry in `RexxToken::precedence` (`Token.cpp:111`), and that
/// absence is the whole reason `-2 ** 2` is 4 rather than -4: a prefix
/// operator takes a whole message subterm as its operand before any dyadic
/// operator is considered, so it can never lose a binding contest.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PrefixOp {
    Plus,
    Minus,
    Not,
}

/// What a function call names.
///
/// Kept apart rather than folded into one string because the two resolve
/// differently, and observably so. Measured with `build/bin/rexx`:
/// `'abs'(-3)` fails with `Error 43.1: Could not find routine "abs"` while
/// `'ABS'(-3)` gives 3. So a literal call name is used exactly as written,
/// case included, and never reaches the upcased builtin table, where a symbol
/// name is upcased by the scanner before it gets here.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CallTarget {
    /// `f(...)`. The id holds the upcased spelling, and this form can also
    /// resolve to an internal label (`parseFunction` calls `addReference`).
    Symbol(SymbolId),
    /// `"f"(...)`. Never an internal label, never a builtin unless the literal
    /// is already upper case.
    Literal(Box<[u8]>),
}

/// One expression form.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprKind {
    /// A quoted literal, carrying its decoded bytes: doubled quotes collapsed
    /// and any `x` or `b` suffix already packed.
    Literal(Box<[u8]>),
    /// A symbol whose value is its own spelling, so never a variable:
    /// `SYMBOL_CONSTANT` and `SYMBOL_DUMMY`, which `addText` treats alike
    /// (`LanguageParser.cpp:2352`).
    ///
    /// The id holds the *upcased* spelling and that is the value, which is
    /// observable: `say 1e5` prints `1E5`, not `100000` and not `1e5`.
    Constant(SymbolId),
    /// A simple variable, no periods.
    Variable(SymbolId),
    /// `stem.`, the id including the trailing period.
    Stem(SymbolId),
    /// `stem.i.j`, the id holding the whole dotted name.
    ///
    /// The tail decomposition is deliberately not stored. It is a pure
    /// function of the spelling with no parse-time decision in it, see
    /// `compound_parts`, and the pieces cannot be interned here because
    /// `ParseCtx::symbols` is read-only during parsing and the pieces are not
    /// tokens, so `scan` never saw them. Storing them un-interned would put
    /// bare strings where every other variable reference carries a `SymbolId`.
    Compound(SymbolId),
    /// `.name`, an environment symbol.
    ///
    /// `ExpressionDotVariable` and `SpecialDotVariable` are collapsed into
    /// this one variant. They are not two syntaxes: the C++ pre-loads
    /// `.nil`, `.true` and `.false` into its `dotVariables` table as
    /// `SpecialDotVariable` instances (`LanguageParser.cpp:782`-`784`) so
    /// that those three resolve without a lookup, which is a retrieval
    /// optimisation inside one syntactic form.
    DotVariable(SymbolId),
    /// A prefix operator applied to a message subterm.
    Prefix { op: PrefixOp, operand: Box<Expr> },
    /// A dyadic operator. `Operator::Backslash` never appears here: `\` is
    /// prefix-only, and one in a dyadic position is error 35.1.
    Binary {
        op: Operator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `f(...)` or `"f"(...)`.
    Call {
        target: CallTarget,
        /// An omitted argument is `None`: `f(,1)` passes two arguments of
        /// which the first is omitted. Trailing omitted arguments are already
        /// dropped, so `f(1,)` holds one.
        args: Vec<Option<Expr>>,
    },
    /// `ns:name(...)`, a namespace-qualified call.
    QualifiedCall {
        namespace: SymbolId,
        name: SymbolId,
        args: Vec<Option<Expr>>,
    },
    /// `ns:name` with no argument list, a namespace-qualified class lookup.
    ClassResolver { namespace: SymbolId, name: SymbolId },
    /// `target~name`, `target~~name`, and `target[...]`.
    ///
    /// `target[...]` is collapsed in here rather than given its own variant
    /// because the interpreter does not distinguish them: `parseCollectionMessage`
    /// builds a `RexxExpressionMessage` whose name is `[]`
    /// (`LanguageParser.cpp:3317`). Measured, `"abc"[2]` and `"abc"~"[]"(2)`
    /// both give `b`, so a user-defined `[]` method answers both spellings and
    /// two variants would be a distinction the language does not make.
    Message {
        target: Box<Expr>,
        /// Upcased, for every spelling. Measured: `"abc"~'length'`,
        /// `"abc"~'LENGTH'` and `"abc"~"lEnGtH"` all give 3, because
        /// `parseMessage` upcases the name whether it came from a symbol or a
        /// literal. `[]` for the bracket form.
        ///
        /// Bytes rather than a `SymbolId`, for two reasons. A method name is
        /// resolved against a behaviour keyed by string, not against a
        /// variable slot keyed by index, so the C++'s `commonString` is
        /// deduplication and nothing more. And a name from a literal cannot be
        /// interned here at all: `ParseCtx::symbols` is read-only during
        /// parsing, and `scan` never saw `LENGTH` as a symbol in `a~'length'`.
        name: Box<[u8]>,
        /// `target~name:super(...)`, the superclass override.
        super_class: Option<Box<Expr>>,
        args: Vec<Option<Expr>>,
        /// True for `~~`, which discards the result and yields the target.
        cascade: bool,
    },
    /// A comma-separated list in parentheses, which builds an array.
    ///
    /// Unlike a call's argument list this keeps trailing omitted elements:
    /// measured, `(1,)~size` is 2 and `(1,,)~size` is 3, where `f(1,)` passes
    /// one argument. `parseFullSubExpression` returns `total` where
    /// `parseArgList` returns `realcount` (`LanguageParser.cpp:3145`).
    List(Vec<Option<Expr>>),
    /// A comma-separated list in a conditional, which is a logical AND of its
    /// parts: `RexxExpressionLogical`, built by `parseLogical` for `IF`,
    /// `WHEN`, `GUARD`, `WHILE` and `UNTIL`. No element may be omitted.
    Logical(Vec<Expr>),
    /// A prefix `>` or `<` on a simple variable or a stem.
    ///
    /// One variant for both spellings because `parseMessageSubterm` maps
    /// `OPERATOR_LESSTHAN` and `OPERATOR_GREATERTHAN` to the same
    /// `parseVariableReferenceTerm` (`LanguageParser.cpp:3661`-`3666`), so the
    /// two are interchangeable and nothing downstream can tell them apart.
    /// The inner node is always a `Variable` or a `Stem`. Anything else is
    /// error 20.930.
    VariableReference(Box<Expr>),
}

impl ExprKind {
    /// Calls `f` on each child expression, in source order.
    ///
    /// An omitted argument has no node and is skipped, so this yields fewer
    /// items than an argument list has positions.
    pub(crate) fn for_each_child(&self, f: &mut impl FnMut(&Expr)) {
        match self {
            ExprKind::Literal(_)
            | ExprKind::Constant(_)
            | ExprKind::Variable(_)
            | ExprKind::Stem(_)
            | ExprKind::Compound(_)
            | ExprKind::DotVariable(_)
            | ExprKind::ClassResolver { .. } => {}
            ExprKind::Prefix { operand, .. } => f(operand),
            ExprKind::Binary { left, right, .. } => {
                f(left);
                f(right);
            }
            ExprKind::Call { args, .. } | ExprKind::QualifiedCall { args, .. } => {
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            ExprKind::Message {
                target,
                super_class,
                args,
                ..
            } => {
                f(target);
                if let Some(super_class) = super_class {
                    f(super_class);
                }
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            ExprKind::List(items) => {
                for item in items.iter().flatten() {
                    f(item);
                }
            }
            ExprKind::Logical(items) => {
                for item in items {
                    f(item);
                }
            }
            ExprKind::VariableReference(inner) => f(inner),
        }
    }
}

impl Expr {
    /// Builds a node whose span is `extent` widened to contain every child's
    /// span.
    ///
    /// The widening is what makes the containment invariant structural. A
    /// caller passes the extent of the tokens it consumed, which is already
    /// right in every case here, and the widening means a caller that gets it
    /// wrong produces a node with too wide a span rather than one that breaks
    /// the invariant.
    pub fn new(kind: ExprKind, extent: Range<usize>) -> Self {
        let mut span = extent;
        kind.for_each_child(&mut |child| {
            span.start = span.start.min(child.span.start);
            span.end = span.end.max(child.span.end);
        });
        Expr { kind, span }
    }

    /// A binary node spanning from its left operand to its right.
    ///
    /// The operator token itself needs no extent: it sits between the two
    /// operands, so their union already covers it. That is also true of the
    /// abuttal operator, whose token the parser synthesises with zero length
    /// at the start of the right operand.
    pub fn binary(op: Operator, left: Expr, right: Expr) -> Self {
        let extent = left.span.start..right.span.end;
        Expr::new(
            ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            extent,
        )
    }

    /// A canonical rendering, for asserting tree shape in tests.
    ///
    /// Follows the D10 spike's `render`, so that the shapes recorded in
    /// `d10-decision.md` still read the same way, with two departures. Both
    /// exist because a rendering that maps two different trees onto one string
    /// silently weakens every assertion made with it.
    ///
    /// An omitted argument renders `<omitted>` and not `_`, because `_` is a
    /// legal symbol character, so `f(_,1)` and `f(,1)` rendered alike. Neither
    /// `<` nor `>` can start a rendered leaf, so `<omitted>` cannot be
    /// produced any other way.
    ///
    /// A message name and a literal render through `{:?}`, quoted and escaped,
    /// because a name is arbitrary bytes: unquoted, `a~"b c"` and `a~b(c)`
    /// rendered alike, and `'a''b'` decodes to `a'b`, which an unescaped
    /// `'...'` cannot render unambiguously either.
    #[cfg(test)]
    pub(crate) fn shape(&self, symbols: &SymbolTable) -> String {
        match &self.kind {
            ExprKind::Literal(bytes) => quoted(bytes),
            ExprKind::Constant(id) | ExprKind::Variable(id) => symbols.name(*id).to_string(),
            ExprKind::Stem(id) => format!("stem:{}", symbols.name(*id)),
            ExprKind::Compound(id) => {
                let name = symbols.name(*id);
                let (stem, tails) = compound_parts(name);
                let rendered: Vec<String> = tails
                    .iter()
                    .map(|t| match t {
                        Tail::Constant(text) => format!("const:{text}"),
                        Tail::Variable(text) => format!("var:{text}"),
                    })
                    .collect();
                format!("compound:{stem}[{}]", rendered.join(","))
            }
            ExprKind::DotVariable(id) => format!("env:{}", symbols.name(*id)),
            ExprKind::Prefix { op, operand } => {
                let name = match op {
                    PrefixOp::Plus => "u+",
                    PrefixOp::Minus => "u-",
                    PrefixOp::Not => "u\\",
                };
                format!("({name} {})", operand.shape(symbols))
            }
            ExprKind::Binary { op, left, right } => {
                let name = match op {
                    Operator::Abuttal => "abut",
                    Operator::Blank => "blank",
                    other => other.spelling(),
                };
                format!("({name} {} {})", left.shape(symbols), right.shape(symbols))
            }
            ExprKind::Call { target, args } => {
                let name = match target {
                    CallTarget::Symbol(id) => symbols.name(*id).to_string(),
                    CallTarget::Literal(bytes) => quoted(bytes),
                };
                format!("(call {name}{})", render_args(symbols, args))
            }
            ExprKind::QualifiedCall {
                namespace,
                name,
                args,
            } => format!(
                "(qcall {}:{}{})",
                symbols.name(*namespace),
                symbols.name(*name),
                render_args(symbols, args)
            ),
            ExprKind::ClassResolver { namespace, name } => {
                format!(
                    "(class {}:{})",
                    symbols.name(*namespace),
                    symbols.name(*name)
                )
            }
            ExprKind::Message {
                target,
                name,
                super_class,
                args,
                cascade,
            } => {
                let twiddle = if *cascade { "~~" } else { "~" };
                let sup = match super_class {
                    Some(s) => format!(" :{}", s.shape(symbols)),
                    None => String::new(),
                };
                format!(
                    "(msg{twiddle} {} {}{sup}{})",
                    target.shape(symbols),
                    quoted(name),
                    render_args(symbols, args)
                )
            }
            ExprKind::List(items) => format!("(list{})", render_args(symbols, items)),
            ExprKind::Logical(items) => {
                let mut out = String::from("(logical");
                for item in items {
                    out.push(' ');
                    out.push_str(&item.shape(symbols));
                }
                out.push(')');
                out
            }
            ExprKind::VariableReference(inner) => format!("(vref {})", inner.shape(symbols)),
        }
    }
}

#[cfg(test)]
fn render_args(symbols: &SymbolTable, args: &[Option<Expr>]) -> String {
    let mut out = String::new();
    for arg in args {
        out.push(' ');
        match arg {
            Some(e) => out.push_str(&e.shape(symbols)),
            // An omitted argument, which is a position with no expression.
            None => out.push_str("<omitted>"),
        }
    }
    out
}

/// Bytes as a quoted, escaped string, so that no two byte strings render alike.
#[cfg(test)]
fn quoted(bytes: &[u8]) -> String {
    format!("{:?}", String::from_utf8_lossy(bytes))
}

/// One tail element of a compound variable, borrowed from the interned name.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Tail<'a> {
    /// A piece that is empty or starts with a digit, so it can never be a
    /// variable name and stands for itself.
    Constant(&'a str),
    /// Anything else: a simple variable whose value supplies this piece.
    Variable(&'a str),
}

/// Splits a compound variable's name into its stem and its tail pieces.
///
/// `name` is the upcased spelling of a symbol the scanner classified
/// `SymbolClass::Compound`, so it holds at least one period that is neither
/// the only one nor at the end. The stem keeps its trailing period, matching
/// `addCompound` (`LanguageParser.cpp:2153`), and a piece that is empty or
/// starts with a digit is a constant rather than a variable
/// (`LanguageParser.cpp:2184`). A trailing period therefore yields a final
/// empty constant piece.
///
/// Measured with `build/bin/rexx`: with `b = 2` and `c = 1`, `say a.b.c`
/// prints `A.2.1`, so `B` and `C` are looked up as variables while the stem
/// contributes its own name.
pub fn compound_parts(name: &str) -> (&str, Vec<Tail<'_>>) {
    let dot = name
        .find('.')
        .expect("a compound symbol holds at least one period");
    let (stem, rest) = name.split_at(dot + 1);
    let tails = rest
        .split('.')
        .map(|piece| {
            if piece.is_empty() || piece.starts_with(|c: char| c.is_ascii_digit()) {
                Tail::Constant(piece)
            } else {
                Tail::Variable(piece)
            }
        })
        .collect();
    (stem, tails)
}

#[cfg(test)]
mod tests;
