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

//! The expression tree and the instruction nodes.
//!
//! One variant per expression class under `interpreter/expression/`, and one
//! `InstructionKind` variant per instruction keyword, with the collapses noted
//! at each site.
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

/// One instruction, and the clause `TRACE` prints for it.
///
/// # Why there is no `next` and no jump target
///
/// Task 3.1 Step 3b settled a flat chain in one arena per code body, with
/// nesting held as indices rather than as child nodes. In a `Vec` the chain
/// itself is index order, so a `next` field would restate it. The jump targets
/// -- where an `IF` goes when its condition is false, which block an `END`
/// closes -- are not computable from one clause: they need the control stack
/// that walks the whole body, and the task that owns that stack adds them.
/// A field nothing sets reads as a contract, so none is declared here.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instruction {
    pub kind: InstructionKind,
    /// Byte range in the retained source: the clause this instruction was
    /// built from, which is what `TRACE` echoes on its `*-*` line.
    ///
    /// Not the extent of the node's own tokens. A `THEN` covers just the
    /// `then` keyword (`ThenInstruction.cpp:76`), and an `IF` stops at the
    /// START of whatever token ended its condition, so the bytes between two
    /// instructions' spans can belong to neither. See `ClauseCursor::split_before`.
    pub clause_span: Range<usize>,
}

/// One instruction form: the 35 keyword instructions, plus the four clause
/// shapes that no keyword introduces.
///
/// Two keywords that share a C++ implementation class still get a variant
/// each, because they are distinct `InstructionKeyword` values there and the
/// spelling is observable: `LEAVE`/`ITERATE` share `RexxInstructionLeave`,
/// `PUSH`/`QUEUE` share `RexxInstructionQueue`, `PARSE`/`ARG`/`PULL` share
/// `RexxInstructionParse`, and `DO`/`LOOP` share every loop class.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InstructionKind {
    // ---- the four clause shapes with no keyword ----
    /// `name = expr`, and also `name (op)= expr`, whose right-hand side is
    /// already the expanded `name op expr` tree (`assignmentOpNew`).
    Assignment {
        target: SymbolId,
        value: Expr,
    },
    /// `name:`, and also `"name":`. Task 3.4 already ended the clause at the
    /// colon.
    ///
    /// Bytes rather than a `SymbolId` because a literal label was never seen
    /// as a symbol, so it is not in the read-only symbol table, and both
    /// spellings reach `addLabel` through the same `token->value()`.
    Label {
        name: Box<[u8]>,
    },
    /// A standalone message send, `q~append(1)`, and the message-assignment
    /// forms `q[1] = 2` and `q[1] += 2`.
    ///
    /// `term` is always an `ExprKind::Message`, whose `cascade` flag carries
    /// the `~~` distinction that the C++ spells as a second instruction type.
    Message {
        term: Expr,
        /// The right-hand side when this is an assignment form. For the
        /// `(op)=` spelling this is already the expanded tree.
        value: Option<Expr>,
    },
    /// Anything else: a clause that is an expression is a command, dispatched
    /// through the current `ADDRESS`.
    Command {
        expression: Option<Expr>,
    },

    // ---- control flow (11) ----
    Do(Box<Loop>),
    Loop(Box<Loop>),
    If {
        condition: Expr,
    },
    Then,
    Else,
    Select {
        label: Option<SymbolId>,
        /// `SELECT CASE expr`, a different instruction class in the C++.
        case: Option<Expr>,
    },
    When {
        condition: Expr,
    },
    Otherwise,
    Leave {
        name: Option<SymbolId>,
    },
    Iterate {
        name: Option<SymbolId>,
    },
    End {
        name: Option<SymbolId>,
    },

    // ---- data (8) ----
    Drop {
        variables: Vec<VariableRef>,
    },
    Expose {
        variables: Vec<VariableRef>,
    },
    Parse(Box<Parse>),
    Arg(Box<Parse>),
    Pull(Box<Parse>),
    Push {
        expression: Option<Expr>,
    },
    Queue {
        expression: Option<Expr>,
    },
    Say {
        expression: Option<Expr>,
    },

    // ---- procedure (11) ----
    Call(Box<Call>),
    Return {
        expression: Option<Expr>,
    },
    Procedure {
        variables: Vec<VariableRef>,
    },
    Signal(Box<Signal>),
    Exit {
        expression: Option<Expr>,
    },
    Interpret {
        expression: Expr,
    },
    Guard(Box<Guard>),
    Reply {
        expression: Option<Expr>,
    },
    Forward(Box<Forward>),
    Raise(Box<Raise>),
    Use(Box<Use>),

    // ---- settings (4) ----
    Numeric {
        setting: NumericSetting,
        expression: Option<Expr>,
    },
    Address(Box<Address>),
    Trace(Trace),
    Options {
        expression: Expr,
    },

    // ---- and NOP ----
    Nop,
}

impl InstructionKind {
    /// The instruction keyword that introduced this node, or `None` for the
    /// four clause shapes that no keyword introduces.
    ///
    /// The spelling is the one in `keywordInstructions[]`, so this is what a
    /// test asserts a keyword reached its node by.
    pub fn keyword(&self) -> Option<&'static str> {
        Some(match self {
            InstructionKind::Assignment { .. }
            | InstructionKind::Label { .. }
            | InstructionKind::Message { .. }
            | InstructionKind::Command { .. } => return None,
            InstructionKind::Do(_) => "DO",
            InstructionKind::Loop(_) => "LOOP",
            InstructionKind::If { .. } => "IF",
            InstructionKind::Then => "THEN",
            InstructionKind::Else => "ELSE",
            InstructionKind::Select { .. } => "SELECT",
            InstructionKind::When { .. } => "WHEN",
            InstructionKind::Otherwise => "OTHERWISE",
            InstructionKind::Leave { .. } => "LEAVE",
            InstructionKind::Iterate { .. } => "ITERATE",
            InstructionKind::End { .. } => "END",
            InstructionKind::Drop { .. } => "DROP",
            InstructionKind::Expose { .. } => "EXPOSE",
            InstructionKind::Parse(_) => "PARSE",
            InstructionKind::Arg(_) => "ARG",
            InstructionKind::Pull(_) => "PULL",
            InstructionKind::Push { .. } => "PUSH",
            InstructionKind::Queue { .. } => "QUEUE",
            InstructionKind::Say { .. } => "SAY",
            InstructionKind::Call(_) => "CALL",
            InstructionKind::Return { .. } => "RETURN",
            InstructionKind::Procedure { .. } => "PROCEDURE",
            InstructionKind::Signal(_) => "SIGNAL",
            InstructionKind::Exit { .. } => "EXIT",
            InstructionKind::Interpret { .. } => "INTERPRET",
            InstructionKind::Guard(_) => "GUARD",
            InstructionKind::Reply { .. } => "REPLY",
            InstructionKind::Forward(_) => "FORWARD",
            InstructionKind::Raise(_) => "RAISE",
            InstructionKind::Use(_) => "USE",
            InstructionKind::Numeric { .. } => "NUMERIC",
            InstructionKind::Address(_) => "ADDRESS",
            InstructionKind::Trace(_) => "TRACE",
            InstructionKind::Options { .. } => "OPTIONS",
            InstructionKind::Nop => "NOP",
        })
    }
}

/// One name in a `DROP`, `EXPOSE`, `PROCEDURE EXPOSE` or `USE LOCAL` list.
///
/// `processVariableList` (`InstructionParser.cpp:4469`) admits both spellings
/// and wraps the second in a `RexxVariableReference`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum VariableRef {
    /// A name written out: a simple variable, a stem, or a compound. Which
    /// of the three follows from the spelling, as it does for `ExprKind`.
    Direct(SymbolId),
    /// `(name)`, where the *value* of `name` names the variable to act on.
    Indirect(SymbolId),
}

/// A `DO` or `LOOP` header.
///
/// One struct for both keywords and for all 23 of the C++'s loop instruction
/// classes, because those classes differ only in which of these fields are
/// present: `createLoop` fills the same `ControlledLoop`, `OverLoop`,
/// `WithLoop`, `ForLoop` and `WhileUntilLoop` structs and then picks a class
/// from which ones came back non-null.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Loop {
    /// `DO LABEL name`. For a controlled or `OVER` loop with no `LABEL`, the
    /// control variable's name becomes the label, exactly as
    /// `newControlledLoop` does.
    pub label: Option<SymbolId>,
    /// `DO COUNTER name`.
    pub counter: Option<SymbolId>,
    pub kind: LoopKind,
    /// A trailing `WHILE` or `UNTIL`. Never both: `parseLoopConditional`
    /// requires the end of the clause after the one it parsed.
    pub conditional: Option<LoopConditional>,
}

/// Which loop a `DO` or `LOOP` header is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LoopKind {
    /// `DO` alone: a block, not a loop. `LOOP` alone is `Forever` instead.
    Simple,
    /// `DO FOREVER`, and `LOOP` with no control at all.
    Forever,
    /// `DO expr`, repeated that many times.
    Count(Option<Expr>),
    /// `DO i = 1 TO 9 BY 2 FOR 3`.
    Controlled(Box<Controlled>),
    /// `DO name OVER expr`.
    Over {
        control: SymbolId,
        target: Expr,
        for_count: Option<Expr>,
    },
    /// `DO WITH INDEX i ITEM v OVER expr`. At least one of the two variables
    /// is present (`Error_Invalid_do_with_no_control` otherwise).
    With {
        index: Option<SymbolId>,
        item: Option<SymbolId>,
        target: Expr,
        for_count: Option<Expr>,
    },
}

/// The control expressions of `DO i = initial TO t BY b FOR f`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Controlled {
    pub control: SymbolId,
    pub initial: Expr,
    pub to: Option<Expr>,
    pub by: Option<Expr>,
    pub for_count: Option<Expr>,
    /// The order the three keyword expressions were written in, which is the
    /// order they are evaluated in (`control.expressions[keyslot++]`).
    /// Evaluation order is observable, because an expression can have side
    /// effects, so it is recorded rather than fixed.
    pub order: Vec<ControlExpr>,
}

/// One entry of a controlled loop's evaluation order.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ControlExpr {
    To,
    By,
    For,
}

/// A `WHILE` or `UNTIL` on a loop.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LoopConditional {
    /// True for `UNTIL`, which is tested after the body rather than before.
    pub until: bool,
    pub condition: Expr,
}

/// A `PARSE`, `ARG` or `PULL` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Parse {
    pub source: ParseSource,
    /// `PARSE UPPER`, and implied by the `ARG` and `PULL` spellings.
    pub upper: bool,
    pub lower: bool,
    pub caseless: bool,
    /// The templates. `None` is the comma fence between one template and the
    /// next, which the C++ pushes as a null entry.
    pub template: Vec<Option<ParseTrigger>>,
}

/// Where a `PARSE` gets its string.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ParseSource {
    Arg,
    LineIn,
    Pull,
    Source,
    Version,
    Var(SymbolId),
    /// `PARSE VALUE expr WITH`. The expression is optional and defaults to
    /// the null string.
    Value(Option<Expr>),
}

/// One template trigger and the variables it assigns.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseTrigger {
    pub kind: TriggerKind,
    /// The pattern or column: a literal, a numeric symbol, or a
    /// parenthesised expression. Absent for `TriggerKind::End`.
    pub value: Option<Expr>,
    /// The targets assigned when this trigger fires. `None` is a `.`
    /// placeholder, which consumes a field and assigns nothing.
    pub targets: Vec<Option<Expr>>,
}

/// What a `PARSE` template trigger matches on.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TriggerKind {
    /// The implicit trigger that assigns whatever is left.
    End,
    /// `+n`, relative forward.
    Plus,
    /// `-n`, relative backward.
    Minus,
    /// `=n` and a bare numeric symbol, both absolute.
    Absolute,
    /// `<n`.
    MinusLength,
    /// `>n`.
    PlusLength,
    /// A literal or `(expr)` pattern.
    String,
    /// The same, under `PARSE CASELESS`.
    Mixed,
}

/// A `CALL` instruction, in all four of its forms.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Call {
    /// `CALL name arg, arg`. `literal` is true for `CALL "name"`, which
    /// bypasses the internal label search.
    Named {
        name: Box<[u8]>,
        literal: bool,
        args: Vec<Option<Expr>>,
    },
    /// `CALL (expr) arg`, whose target is only known at run time.
    Dynamic {
        target: Expr,
        args: Vec<Option<Expr>>,
    },
    /// `CALL ns:name arg`, restricted to public routines of that namespace.
    Qualified {
        namespace: SymbolId,
        name: SymbolId,
        args: Vec<Option<Expr>>,
    },
    /// `CALL ON cond NAME label` and `CALL OFF cond`.
    Trap(ConditionTrap),
}

/// A `SIGNAL` instruction, in all three of its forms.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Signal {
    /// `SIGNAL label` and `SIGNAL "label"`.
    Label(Box<[u8]>),
    /// `SIGNAL VALUE expr`, and the implicit form where the target is not a
    /// symbol or a literal.
    Value(Expr),
    /// `SIGNAL ON cond NAME label` and `SIGNAL OFF cond`.
    Trap(ConditionTrap),
}

/// The shared shape of `CALL ON`/`OFF` and `SIGNAL ON`/`OFF`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConditionTrap {
    pub on: bool,
    /// The condition name, `USER name` spelled out for a user condition
    /// exactly as `commonString(name->concatToCstring("USER "))` builds it.
    pub condition: Box<[u8]>,
    /// The label to trap to. `None` for the `OFF` form, which is how the
    /// C++ distinguishes them too.
    pub label: Option<Box<[u8]>>,
}

/// A `GUARD ON`/`OFF` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Guard {
    pub on: bool,
    /// `GUARD ON WHEN expr`.
    pub condition: Option<Expr>,
}

/// A `FORWARD` instruction's options, all of them optional.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Forward {
    pub to: Option<Expr>,
    pub message: Option<Expr>,
    pub class: Option<Expr>,
    /// `FORWARD ARGUMENTS expr`, mutually exclusive with `array`.
    pub arguments: Option<Expr>,
    /// `FORWARD ARRAY (a, b)`.
    pub array: Option<Vec<Option<Expr>>>,
    pub continue_: bool,
}

/// A `RAISE` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Raise {
    /// The condition name, with `USER ` prefixed for a user condition.
    pub condition: Box<[u8]>,
    pub propagate: bool,
    /// The argument `ERROR`, `FAILURE` and `SYNTAX` take.
    pub rc: Option<Expr>,
    pub description: Option<Expr>,
    /// `ADDITIONAL expr`, mutually exclusive with `array`.
    pub additional: Option<Expr>,
    pub array: Option<Vec<Option<Expr>>>,
    /// `RETURN expr` or `EXIT expr`, whose value is optional either way.
    pub result: Option<RaiseResult>,
}

/// `RAISE ... RETURN expr` versus `RAISE ... EXIT expr`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RaiseResult {
    pub exit: bool,
    pub value: Option<Expr>,
}

/// A `USE ARG`, `USE STRICT ARG` or `USE LOCAL` instruction.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Use {
    Arg {
        strict: bool,
        /// True when the list ended with `...`, which stops argument-count
        /// checking at that point.
        allow_optionals: bool,
        /// An omitted position, written as a bare comma, is `None`.
        targets: Vec<Option<UseTarget>>,
    },
    Local {
        variables: Vec<VariableRef>,
    },
}

/// One target of a `USE ARG` list.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UseTarget {
    /// A variable or a message term (`parseVariableOrMessageTerm`).
    pub target: Expr,
    /// `USE ARG a = 1`, a constant expression. Never present with `alias`.
    pub default: Option<Expr>,
    /// `USE ARG >a`, which aliases the caller's variable rather than copying.
    /// `<` is the same thing (`isOperator(OPERATOR_LESSTHAN)`).
    pub alias: bool,
}

/// An `ADDRESS` instruction.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Address {
    /// `ADDRESS env`, the constant target. A symbol contributes its upcased
    /// spelling and a literal its bytes, which is what `token->value()`
    /// yields for each.
    pub environment: Option<Box<[u8]>>,
    /// `ADDRESS VALUE expr`, and the implicit form where the target is
    /// neither a symbol nor a literal.
    pub dynamic: Option<Expr>,
    /// `ADDRESS env command`.
    pub command: Option<Expr>,
    /// The `WITH` redirections. Absent means the plain `RexxInstructionAddress`
    /// rather than `RexxInstructionAddressWith`.
    pub io: Option<Box<AddressIo>>,
}

/// `ADDRESS ... WITH` input and output redirection.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct AddressIo {
    pub input: Redirection,
    pub output: Redirection,
    pub error: Redirection,
    pub output_option: OutputOption,
    pub error_option: OutputOption,
}

/// Where one of the three command streams goes.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum Redirection {
    /// Not mentioned at all.
    #[default]
    Default,
    /// `NORMAL`, which resets this stream to the default.
    Normal,
    /// `STEM name.`.
    Stem(SymbolId),
    /// `STREAM expr`, a constant expression naming a file.
    Stream(Expr),
    /// `USING expr`, an object decided at run time.
    Using(Expr),
}

/// `APPEND` or `REPLACE` on an output redirection.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum OutputOption {
    #[default]
    Default,
    Replace,
    Append,
}

/// Which setting a `NUMERIC` instruction changes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NumericSetting {
    Digits,
    Fuzz,
    /// `NUMERIC FORM` alone, which resets to the package default.
    FormDefault,
    FormScientific,
    FormEngineering,
    /// `NUMERIC FORM VALUE expr`, and the implicit form where what follows
    /// `FORM` is not a symbol.
    FormValue,
}

/// A `TRACE` instruction, in its four forms.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Trace {
    /// `TRACE` alone: the default setting.
    Default,
    /// A validated option string, kept as written. Only its leading `?`s and
    /// first other character mean anything (`TraceSetting.cpp:135`), and the
    /// rest is retained because the setting is echoed back by `TRACE()`.
    Setting(Box<[u8]>),
    /// A whole number, which skips that many debug pauses. Negative for the
    /// `TRACE -n` spelling.
    Skip(i64),
    /// `TRACE VALUE expr`, and the implicit form where what follows is not a
    /// symbol, a literal, or a signed number.
    Value(Expr),
}

#[cfg(test)]
mod tests;
