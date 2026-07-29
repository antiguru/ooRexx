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

use std::collections::BTreeMap;
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

/// One code body: an instruction chain and the labels declared in it.
///
/// One of these per `translateBlock` call: the main program has one, and each
/// `::METHOD`, `::ATTRIBUTE` and `::ROUTINE` that carries a body has its own.
/// A label is local to the body that declares it, which is why the table lives
/// here rather than once per program.
///
/// Every index stored anywhere inside -- a jump target, the block an `END`
/// closes, a `labels` value -- indexes `instructions` of THIS body. An index
/// from one body is meaningless in another.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CodeBody {
    /// In source order, which is also the execution chain: control falls from
    /// each instruction to the next index unless a jump target says otherwise.
    ///
    /// There is no `next` field because there is nothing for one to say. Every
    /// instruction enters the chain through the port of `addClause`
    /// (`LanguageParser.cpp:2544`), which only ever appends, so index order is
    /// the chain exactly.
    pub instructions: Vec<Instruction>,
    /// Keyed by the label token's VALUE, not by `SymbolId`: upcased for a
    /// symbol label, verbatim for a literal one. The first occurrence of a
    /// duplicated label wins.
    pub labels: BTreeMap<Box<[u8]>, usize>,
}

/// Which closure action an `END` performs: `EndBlockType`
/// (`RexxInstruction.hpp:107`), as `getEndStyle` and
/// `RexxInstructionSelect::matchEnd` set it.
///
/// The C++ enum also holds `LABELED_SELECT_BLOCK`, which nothing sets: a
/// `SELECT`'s own `getEndStyle` (`SelectInstruction.hpp:76`) returns
/// `SELECT_BLOCK` whether it is labelled or not, and `matchEnd` overrides only
/// the two `OTHERWISE` cases. There is no variant for it here.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EndStyle {
    /// A `DO` with no control expression and no `LABEL`.
    Do,
    /// A `DO` with no control expression but with a `LABEL`.
    LabeledDo,
    /// Every other `DO`/`LOOP` form, labelled or not: `getEndStyle`
    /// (`DoInstruction.hpp:105`) answers `LOOP_BLOCK` for all of them, so the
    /// label makes no difference here where it does for the block form.
    Loop,
    /// A `SELECT` with no `OTHERWISE`, labelled or not. Reaching this `END` at
    /// run time is error 7.3, because every `WHEN` was false.
    Select,
    /// A `SELECT` with an `OTHERWISE` and no `LABEL`.
    Otherwise,
    /// A `SELECT` with an `OTHERWISE` and a `LABEL`.
    LabeledOtherwise,
}

/// What an `END` turned out to close, and how.
///
/// The two travel together because neither is decided until the `END` is
/// matched, and neither is meaningful without the other.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct EndTarget {
    /// The block instruction this `END` closes: a `Do`, a `Loop` or a
    /// `Select`. Never the `Otherwise`, even when `style` says the `SELECT` had
    /// one: `translateBlock` pops the `OTHERWISE` and matches the `END` against
    /// the `SELECT` behind it (`LanguageParser.cpp:1530`).
    pub block: usize,
    pub style: EndStyle,
}

/// One instruction, and the clause `TRACE` prints for it.
///
/// # Why there is no `next`
///
/// Task 3.1 Step 3b settled a flat chain in one arena per code body, with
/// nesting held as indices rather than as child nodes. In a `Vec` the chain
/// itself is index order, so a `next` field would restate it. The jump targets
/// -- where an `IF` goes when its condition is false, which block an `END`
/// closes -- are not computable from one clause: they need the control stack
/// that walks the whole body, and they live on the kinds that jump.
///
/// # Why there is no node for the synthetic end of a branch
///
/// The C++ chain holds `RexxInstructionEndIf` markers that `endIfNew` builds
/// out of nothing, one at the end of every `THEN` and `ELSE` branch. They are
/// not reproduced. Measured with `trace r`, `if 1 = 1 then say "y"` followed by
/// `say "after"` traces exactly three `*-*` lines for the `IF` clause and then
/// the next line's, so no marker is ever echoed and nothing observable is lost.
/// Reproducing them would put an `Instruction` with no source clause into
/// `instructions`, which two gate criteria are stated over: source-ordered
/// non-overlapping `clause_span`s, and one `*-*` line per clause. A jump index
/// on the instruction that jumps carries the same information with neither
/// exception.
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
    ///
    /// `target` is what `addVariable` builds, so its kind is always
    /// `Variable`, `Stem` or `Compound`: `needVariable` rejects every other
    /// class before this node exists. It is an `Expr` rather than a bare
    /// `SymbolId` so that the class is carried the way every other variable
    /// reference in this tree carries it, and so that the target keeps its own
    /// span.
    Assignment {
        target: Expr,
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

    // ---- control flow (12) ----
    Do(Box<Loop>),
    Loop(Box<Loop>),
    If {
        condition: Expr,
        /// Where control goes when `condition` is false: the `ELSE` when there
        /// is one, otherwise the instruction after the `THEN` branch. `None` is
        /// the end of this body.
        ///
        /// `RexxInstructionIf::else_location->nextInstruction`
        /// (`IfInstruction.cpp:147`). The C++ target is the synthetic marker
        /// that closes the `THEN` branch, and control resumes at the
        /// instruction after it, so this is that instruction directly.
        false_target: Option<usize>,
    },
    Then,
    Else {
        /// Where control goes when the `THEN` branch finished, which is the
        /// instruction after the `ELSE` branch. `None` is the end of this body.
        ///
        /// Stored on the `ELSE` because there is nothing to skip without one:
        /// a `THEN` branch with no `ELSE` falls straight through, which is the
        /// C++'s null `else_end` (`EndIf.cpp:154`). `RexxInstructionElse` does
        /// not read it either -- executing an `ELSE` only traces
        /// (`ElseInstruction.cpp:113`) -- it forwards the target to the marker
        /// that closes the `THEN` branch (`:145`), and this field is where that
        /// forwarding lands.
        then_exit: Option<usize>,
    },
    Select {
        label: Option<SymbolId>,
        /// `SELECT CASE expr`, a different instruction class in the C++.
        case: Option<Expr>,
        /// The `WHEN` instructions this `SELECT` collected, in source order.
        ///
        /// `RexxInstructionSelect::whenList`, filled by `addWhen`. Only a
        /// `WHEN` whose immediate enclosing block is this `SELECT` is here, and
        /// that is narrower than "every `WHEN` between here and the `END`":
        /// measured rc 0, `select` / `when 1 = 1 then` / `when 2 = 2 then nop` /
        /// `end` is accepted and the second `WHEN` is the first one's `THEN`
        /// instruction, so it is never added (`LanguageParser.cpp:1319`).
        whens: Vec<usize>,
        otherwise: Option<usize>,
        /// The `END` that closes this `SELECT`.
        ///
        /// `None` only while the body is still being assembled: an unclosed
        /// `SELECT` is error 14.2, so a body that parsed has this set.
        end: Option<usize>,
    },
    When {
        condition: Expr,
        /// Where control goes when `condition` is false: the next `WHEN`, the
        /// `OTHERWISE`, or the enclosing `SELECT`'s `END`.
        false_target: Option<usize>,
        /// Where control goes when this `WHEN`'s branch finished, which is the
        /// instruction after the enclosing `SELECT`'s `END`, because one true
        /// `WHEN` ends the whole `SELECT`. `None` is the end of this body.
        ///
        /// `fixWhen` (`SelectInstruction.cpp:222`) sets it, so it is the same
        /// value for every `WHEN` of one `SELECT`, and `None` for a `WHEN` that
        /// `whens` never collected.
        exit: Option<usize>,
    },
    /// A `WHEN` inside `SELECT CASE`: `RexxInstructionCaseWhen`, a different
    /// class from `RexxInstructionIf` because the clause means something else.
    ///
    /// `parseCaseWhenList` (`LanguageParser.cpp:3168`) builds a LIST of values
    /// to compare against the `SELECT`'s own expression where `parseLogical`
    /// builds an AND of conditions. Measured at run time, which is the only
    /// place the two differ for a single-element list: `select case 2` /
    /// `when 1, 2 then say "hit"` prints `hit`, while plain `select` /
    /// `when 1, 2` is error 34.6 `found "2"` because 2 is not a logical value.
    WhenCase {
        /// At least one, and none may be omitted: an empty element is 35.934.
        values: Vec<Expr>,
        false_target: Option<usize>,
        exit: Option<usize>,
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
        /// What this `END` closes, and how.
        ///
        /// `None` only while the body is still being assembled: an `END` with
        /// no open block is error 10.1, so a body that parsed has this set.
        closes: Option<EndTarget>,
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
            InstructionKind::Else { .. } => "ELSE",
            InstructionKind::Select { .. } => "SELECT",
            // Both spellings are the WHEN keyword. They are separate variants
            // because they are separate instruction classes in the C++ and the
            // clause means something else in each, not because the keyword
            // differs.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. } => "WHEN",
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
    /// The `END` that closes this block.
    ///
    /// `None` only while the body is still being assembled: an unclosed block
    /// is error 14.1 or 14.5, so a body that parsed has this set.
    pub end: Option<usize>,
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

/// One `::` directive, and the clause it was built from.
///
/// Shaped like `Instruction` on purpose, because both are one clause in and one
/// node out. A directive is not part of any instruction chain: `translate`
/// drains the instructions first and then loops over directives
/// (`LanguageParser.cpp:735`), so a directive has no index into the chain and
/// nothing jumps to one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Directive {
    pub kind: DirectiveKind,
    /// Byte range in the retained source: the directive clause.
    ///
    /// A directive is never traced, so unlike `Instruction::clause_span` nothing
    /// echoes these bytes. They are kept because `ClassDirective`,
    /// `RequiresDirective` and `ConstantDirective` all retain their clause for
    /// the location an install-time error is reported against.
    pub clause_span: Range<usize>,
}

/// One directive form, one variant per row of `RexxToken::directives[]`
/// (`KeywordConstants.cpp:52`-`63`).
///
/// There are nine rows and nine variants. `DIRECTIVE_LIBRARY` is a
/// `DirectiveKeyword` enum member with no row in that table and so no variant
/// here: a library is `::REQUIRES name LIBRARY`, which `requiresDirective`
/// turns into a `LibraryDirective` at the end
/// (`DirectiveParser.cpp:2857`-`2860`), and it is never a directive keyword of
/// its own.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DirectiveKind {
    Annotate(Box<Annotate>),
    Attribute(Box<AttributeDirective>),
    Class(Box<ClassDirective>),
    Constant(Box<ConstantDirective>),
    Method(Box<MethodDirective>),
    /// `::OPTIONS`, whose options may repeat and whose order is observable, so
    /// this is a list and not a struct of fields. Measured with
    /// `build/bin/rexx`: `::options digits 12` then `::options digits 5` makes
    /// `digits()` report 5, and the two directives swapped make it report 12,
    /// so a later option overrides an earlier one.
    Options(Vec<PackageOption>),
    Requires(Box<Requires>),
    Resource(Box<Resource>),
    Routine(Box<RoutineDirective>),
}

impl DirectiveKind {
    /// The directive keyword that introduced this node.
    ///
    /// The spelling is the one in `directives[]`, so this is what a test
    /// asserts a keyword reached its node by.
    pub fn keyword(&self) -> &'static str {
        match self {
            DirectiveKind::Annotate(_) => "ANNOTATE",
            DirectiveKind::Attribute(_) => "ATTRIBUTE",
            DirectiveKind::Class(_) => "CLASS",
            DirectiveKind::Constant(_) => "CONSTANT",
            DirectiveKind::Method(_) => "METHOD",
            DirectiveKind::Options(_) => "OPTIONS",
            DirectiveKind::Requires(_) => "REQUIRES",
            DirectiveKind::Resource(_) => "RESOURCE",
            DirectiveKind::Routine(_) => "ROUTINE",
        }
    }
}

/// A method's or routine's access scope.
///
/// `Default` is a value and not the absence of one: the C++ keeps
/// `DEFAULT_ACCESS_SCOPE` distinct from `PUBLIC_SCOPE` precisely so that a
/// SECOND access keyword is an error, which is why `::METHOD m PUBLIC PUBLIC`
/// and `::METHOD m PUBLIC PRIVATE` are both 25.902.
///
/// `::CLASS` and `::ROUTINE` admit only `Public` and `Private`. `Package` is
/// reachable on `::METHOD` and `::ATTRIBUTE` alone.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Access {
    #[default]
    Default,
    Private,
    Public,
    Package,
}

/// Whether a method runs with the object's method protection.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Protection {
    #[default]
    Default,
    Protected,
    Unprotected,
}

/// Whether a method takes the object's guard lock.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum GuardOption {
    #[default]
    Default,
    Guarded,
    Unguarded,
}

/// A decoded `EXTERNAL` specification.
///
/// `decodeExternalMethod` (`DirectiveParser.cpp:1403`) and the routine form
/// (`DirectiveParser.cpp:2649`-`2749`) split the string into blank-delimited
/// words and upcase the first, then require two or three words whose first is
/// `LIBRARY`. A routine also accepts `REGISTERED`. Anything else is 99.917,
/// which is a parse error, so the decode happens here and not at install time.
///
/// Resolving the library is NOT done here and cannot be: measured,
/// `::METHOD m EXTERNAL "LIBRARY nosuch"` is `Error 98.903 Unable to load
/// library "nosuch"` at rc 158, a run-time failure of a program that parsed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExternalSpec {
    /// True for the `REGISTERED` spelling, which only `::ROUTINE` accepts and
    /// which resolves an old-style external function instead of a library
    /// entry point.
    pub registered: bool,
    /// The library name, as written after the first word.
    pub library: Box<[u8]>,
    /// The entry point, when the specification named a third word.
    ///
    /// `None` is the absence of a third word and NOT a default, because the
    /// default is three different names depending on what asked for it, and only
    /// the asker knows which:
    ///
    /// * A `::ROUTINE` uses the routine's own name AS WRITTEN, case included
    ///   (`DirectiveParser.cpp:2664`).
    /// * A `::METHOD` uses the method's UPCASED lookup name
    ///   (`DirectiveParser.cpp:1406`).
    /// * A `::METHOD ATTRIBUTE` or `::ATTRIBUTE` uses that upcased name with
    ///   `GET` or `SET` appended, one method each
    ///   (`DirectiveParser.cpp:1678`-`1679`).
    ///
    /// Resolving those is the caller's, along with loading the library, so
    /// filling one in here would be picking one of the three arbitrarily.
    pub entry: Option<Box<[u8]>>,
}

/// A `::CLASS` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClassDirective {
    /// The name as written. The class is also exported under the upcased
    /// spelling, which is what a duplicate is detected on, but that is the
    /// accumulator's business and not this node's.
    pub name: Box<[u8]>,
    /// `PUBLIC` or `PRIVATE`. `Package` never appears: measured,
    /// `::CLASS c PACKAGE` is 25.901.
    pub access: Access,
    pub abstract_: bool,
    /// `SUBCLASS c` and `MIXINCLASS c`, which fill the same slot in the C++
    /// (`setMixinClass` sets the subclass too), which is why a directive
    /// carrying both is 25.901 whichever order they come in.
    pub subclass: Option<ClassRef>,
    /// True when the subclass came from `MIXINCLASS`.
    pub mixin: bool,
    pub metaclass: Option<ClassRef>,
    /// `INHERIT a b c`, which consumes every remaining token of the clause.
    pub inherit: Vec<ClassRef>,
}

/// A class reference on a `::CLASS` directive: the argument of `SUBCLASS`,
/// `MIXINCLASS`, `METACLASS`, or one entry of `INHERIT`.
///
/// `parseClassReference` (`DirectiveParser.cpp:287`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ClassRef {
    /// `ns:name`, restricted to the symbol spelling. A literal cannot carry a
    /// namespace: `parseClassReference` returns immediately for one.
    pub namespace: Option<SymbolId>,
    /// Upcased for both spellings. A symbol arrives upcased from the scanner
    /// and a literal is upcased here, because `parseClassReference` calls
    /// `token->upperValue()` for the literal form.
    pub name: Box<[u8]>,
}

/// A `::METHOD` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MethodDirective {
    /// The name as written, which is the method object's own name. The lookup
    /// name is its upcased spelling, and the two differ observably for
    /// `::METHOD "abc"`.
    pub name: Box<[u8]>,
    /// `CLASS`, making this a class method rather than an instance method.
    pub class_method: bool,
    /// `ATTRIBUTE`, which generates a getter and a setter pair.
    pub attribute: bool,
    pub abstract_: bool,
    pub access: Access,
    pub protection: Protection,
    pub guard: GuardOption,
    pub external: Option<ExternalSpec>,
    /// `DELEGATE property`, forwarding every message to that property's value.
    /// A symbol only: measured, `::METHOD m DELEGATE "p"` is 20.926.
    pub delegate: Option<SymbolId>,
    /// This method's code body: the clauses after this directive, assembled.
    ///
    /// `None` for every option that generates the method itself, and then a
    /// following non-directive clause is an error with its own number: 99.946
    /// for `DELEGATE`, 99.934 for `ATTRIBUTE`, 99.933 for `ABSTRACT` and
    /// 99.936 for `EXTERNAL`. Those are raised while parsing this directive,
    /// so a `None` here has already been checked against what follows.
    ///
    /// The directive parser decides only whether a body belongs here and leaves
    /// an empty `CodeBody`, which is also the right answer for a body with no
    /// clauses in it. The block assembler fills it.
    pub body: Option<CodeBody>,
}

/// A `::ATTRIBUTE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AttributeDirective {
    /// The name as written, which also names the instance variable the
    /// generated methods read and write.
    pub name: Box<[u8]>,
    pub style: AttributeStyle,
    pub class_method: bool,
    pub abstract_: bool,
    pub access: Access,
    pub protection: Protection,
    pub guard: GuardOption,
    pub external: Option<ExternalSpec>,
    pub delegate: Option<SymbolId>,
    /// This attribute method's code body: the clauses after this directive,
    /// assembled.
    ///
    /// Only `GET` or `SET` with no generating option can have one, and there
    /// the C++ asks `hasBody()` (`DirectiveParser.cpp:1773`) rather than
    /// deciding from the options: with a body the method is written in Rexx,
    /// without one it is generated. So whether this is `Some` is the one place
    /// a directive parse depends on the clause that FOLLOWS it.
    pub body: Option<CodeBody>,
}

/// Which of the attribute method pair a `::ATTRIBUTE` directive defines.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum AttributeStyle {
    /// Neither `GET` nor `SET`, so both methods are generated and no body may
    /// follow.
    #[default]
    Both,
    Get,
    Set,
}

/// A `::CONSTANT` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConstantDirective {
    pub name: Box<[u8]>,
    pub value: ConstantValue,
}

/// What a `::CONSTANT` directive's value is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConstantValue {
    /// No value at all, whose value is the constant's own name AS WRITTEN and
    /// not upcased (`value = name` at `DirectiveParser.cpp:1875`).
    Name,
    /// A literal, a symbol, or a signed constant symbol, taken as text. The
    /// signed form is concatenated exactly as the C++ concatenates it, so
    /// `::CONSTANT c - 5` yields `-5` with the blank dropped.
    Text(Box<[u8]>),
    /// `(expr)`, which is evaluated when the package is installed rather than
    /// now.
    Expression(Expr),
}

/// A `::ANNOTATE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Annotate {
    pub target: AnnotationTarget,
    /// The `symbol value` pairs, in order. An `::ANNOTATE` may carry none.
    pub annotations: Vec<Annotation>,
}

/// What a `::ANNOTATE` directive annotates.
///
/// Every name is upcased, because `annotateDirective` looks each one up with
/// `commonString(token->upperValue())`. Each target must already exist, which
/// needs the accumulated package and is not checked here.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AnnotationTarget {
    Package,
    Class(Box<[u8]>),
    Routine(Box<[u8]>),
    Method(Box<[u8]>),
    /// The getter name. The C++ annotates whichever of the getter/setter pair
    /// exists (`processAttributeAnnotations`), so the setter name is derived
    /// rather than stored.
    Attribute(Box<[u8]>),
    Constant(Box<[u8]>),
}

/// One `name value` pair of a `::ANNOTATE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Annotation {
    /// The name, which must be a symbol: measured, `::ANNOTATE PACKAGE "a" 1`
    /// is 20.919.
    pub name: SymbolId,
    /// The value as text, with the same three forms a `::CONSTANT` value has
    /// minus the parenthesised one, which is not accepted here.
    pub value: Box<[u8]>,
}

/// One option of a `::OPTIONS` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PackageOption {
    /// `DIGITS n`, always at least 1.
    Digits(usize),
    Fuzz(usize),
    Form(OptionsForm),
    /// A validated `TRACE` option string, kept as written for the same reason
    /// `Trace::Setting` keeps one.
    Trace(Box<[u8]>),
    /// One of the seven conditions that can be raised as a SYNTAX error
    /// instead of as a condition. `syntax` is true for the `SYNTAX` spelling
    /// and false for `CONDITION`.
    Condition {
        which: ConditionOption,
        syntax: bool,
    },
    /// `PROLOG` and `NOPROLOG`, true for the first.
    Prolog(bool),
    /// `NUMERIC INHERIT` and `NUMERIC NOINHERIT`, true for the first.
    NumericInherit(bool),
}

/// `::OPTIONS FORM`'s two settings.
///
/// Kept apart from `NumericSetting` because that enum's `FormValue` and
/// `FormDefault` have no `::OPTIONS` spelling: measured,
/// `::OPTIONS FORM VALUE` is 25.11.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OptionsForm {
    Scientific,
    Engineering,
}

/// Which condition a `::OPTIONS` condition option selects.
///
/// `All` is a spelling and not a set, because it is a row of
/// `subDirectives[]` in its own right and it sets the other six at once.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ConditionOption {
    All,
    Error,
    Failure,
    LostDigits,
    NoString,
    NotReady,
    NoValue,
}

/// A `::REQUIRES` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Requires {
    /// The name as written, so `::REQUIRES "Mixed.cls"` keeps its case while
    /// the symbol spelling arrives upcased.
    pub name: Box<[u8]>,
    /// `LIBRARY`, which makes this a native library rather than a package
    /// file. Never true together with a namespace: measured,
    /// `::REQUIRES x LIBRARY NAMESPACE ns` is 25.904 whichever order the two
    /// come in.
    pub library: bool,
    /// `NAMESPACE ns`. A symbol only, so always upcased, and never `REXX`:
    /// measured, `::REQUIRES "x" NAMESPACE REXX` is 99.944.
    pub namespace: Option<SymbolId>,
}

/// A `::RESOURCE` directive and its verbatim body.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Resource {
    /// The name as written. The package's resource table is keyed by the
    /// upcased spelling.
    pub name: Box<[u8]>,
    /// The line that ends the body, `::END` unless the directive named
    /// another. Compared against the source verbatim, so a lower-case `::end`
    /// does NOT end a body that expects `::END`.
    pub end_marker: Box<[u8]>,
    /// Byte range of each body line in the retained source, line terminators
    /// and the marker line excluded.
    ///
    /// Ranges rather than text, because the body is a slice of the retained
    /// source and copying it would duplicate the whole of a large resource.
    pub lines: Vec<Range<usize>>,
}

/// A `::ROUTINE` directive.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RoutineDirective {
    /// The name as written and NOT upcased, because a quoted routine name is
    /// looked up case-sensitively (`DirectiveParser.cpp:2575`).
    pub name: Box<[u8]>,
    /// `PUBLIC` or `PRIVATE`. `Package` never appears: measured,
    /// `::ROUTINE r PACKAGE` is 25.903.
    pub access: Access,
    pub external: Option<ExternalSpec>,
    /// This routine's code body: the clauses after this directive, assembled.
    /// `Some` for every routine that is not external.
    pub body: Option<CodeBody>,
}

#[cfg(test)]
mod tests;
