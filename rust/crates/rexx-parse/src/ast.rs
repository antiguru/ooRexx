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

use std::collections::BTreeMap;
use std::ops::Range;

use crate::selector::Selector;
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PrefixOp {
    Plus,
    Minus,
    Not,
}

impl PrefixOp {
    /// The canonical source spelling, which is what a `>P>` trace line carries
    /// as its tag.
    pub fn spelling(self) -> &'static str {
        match self {
            PrefixOp::Plus => "+",
            PrefixOp::Minus => "-",
            PrefixOp::Not => "\\",
        }
    }
}

/// What a function call names.
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
    Constant(SymbolId),
    /// A simple variable, no periods.
    Variable(SymbolId),
    /// `stem.`, the id including the trailing period.
    Stem(SymbolId),
    /// `stem.i.j`, the id holding the whole dotted name.
    Compound(SymbolId),
    /// `.name`, an environment symbol.
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
    Message {
        target: Box<Expr>,
        /// Upcased, for every spelling. Measured: `"abc"~'length'`,
        /// `"abc"~'LENGTH'` and `"abc"~"lEnGtH"` all give 3, because
        /// `parseMessage` upcases the name whether it came from a symbol or a
        /// literal. `[]` for the bracket form.
        name: Selector,
        /// `target~name:super(...)`, the superclass override.
        super_class: Option<Box<Expr>>,
        args: Vec<Option<Expr>>,
        /// True for `~~`, which discards the result and yields the target.
        cascade: bool,
    },
    /// A comma-separated list in parentheses, which builds an array.
    List(Vec<Option<Expr>>),
    /// A comma-separated list in a conditional, which is a logical AND of its
    /// parts: `RexxExpressionLogical`, built by `parseLogical` for `IF`,
    /// `WHEN`, `GUARD`, `WHILE` and `UNTIL`. No element may be omitted.
    Logical(Vec<Expr>),
    /// A prefix `>` or `<` on a simple variable or a stem.
    VariableReference(Box<Expr>),
}

impl ExprKind {
    /// Calls `f` on each child expression, in source order.
    pub(crate) fn for_each_child<'a>(&'a self, f: &mut impl FnMut(&'a Expr)) {
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

    /// The mutable twin of `for_each_child`, for `Expr`'s iterative `Drop`.
    fn for_each_child_mut(&mut self, f: &mut impl FnMut(&mut Expr)) {
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
                for arg in args.iter_mut().flatten() {
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
                for arg in args.iter_mut().flatten() {
                    f(arg);
                }
            }
            ExprKind::List(items) => {
                for item in items.iter_mut().flatten() {
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
    pub fn new(kind: ExprKind, extent: Range<usize>) -> Self {
        let mut span = extent;
        kind.for_each_child(&mut |child| {
            span.start = span.start.min(child.span.start);
            span.end = span.end.max(child.span.end);
        });
        Expr { kind, span }
    }

    /// A binary node spanning from its left operand to its right.
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
                    quoted(name.bytes()),
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

/// Drops the whole subtree by iteration, not by the recursion a derived
/// `Drop` would use.
impl Drop for Expr {
    fn drop(&mut self) {
        let mut worklist: Vec<Expr> = Vec::new();
        self.kind.for_each_child_mut(&mut |child| {
            worklist.push(std::mem::replace(child, cheap_leaf()));
        });
        while let Some(mut expr) = worklist.pop() {
            expr.kind.for_each_child_mut(&mut |child| {
                worklist.push(std::mem::replace(child, cheap_leaf()));
            });
        }
    }
}

/// A leaf with no children, swapped into a slot once its real value has been
/// moved onto `Drop`'s worklist, so the slot's own drop has nothing deep left
/// to walk. The span is meaningless: this node is never observed, only
/// dropped.
fn cheap_leaf() -> Expr {
    Expr {
        kind: ExprKind::Literal(Box::from(&b""[..])),
        span: 0..0,
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
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CodeBody {
    /// In source order, which is also the execution chain: control falls from
    /// each instruction to the next index unless a jump target says
    /// otherwise. Ends where the first `::` directive clause begins or the
    /// source ends -- `translate_block` stops there and leaves the cursor
    /// sitting on it for the caller, so an instruction after that point
    /// belongs to the next code body, never this one.
    pub instructions: Vec<Instruction>,
    /// Keyed by the label token's VALUE, not by `SymbolId`: upcased for a
    /// symbol label, verbatim for a literal one. `Box<[u8]>` rather than
    /// `Box<str>`, because a literal label is not required to be valid UTF-8
    /// any more than any other literal is -- measured, a label spelled with a
    /// raw non-UTF-8 byte is a legal `SIGNAL VALUE` target under
    /// `build/bin/rexx`. Interning the key would be wrong in both directions;
    /// see Task 3.3's six measurements. The first occurrence of a duplicated
    /// label wins -- measured, two labels spelled `a:` in one program is
    /// accepted and `signal a` reaches the first -- so this is built with
    /// "insert if absent", never an unconditional overwrite.
    pub labels: BTreeMap<Box<[u8]>, usize>,
}

/// Which closure action an `END` performs: `EndBlockType`
/// (`RexxInstruction.hpp:107`), as `getEndStyle` and
/// `RexxInstructionSelect::matchEnd` set it.
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
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Instruction {
    pub kind: InstructionKind,
    /// Byte range in the retained source: the clause this instruction was
    /// built from, which is what `TRACE` echoes on its `*-*` line.
    pub clause_span: Range<usize>,
}

/// One instruction form: the 35 keyword instructions, plus the four clause
/// shapes that no keyword introduces.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InstructionKind {
    // ---- the four clause shapes with no keyword ----
    /// `name = expr`, and also `name (op)= expr`, whose right-hand side is
    /// already the expanded `name op expr` tree (`assignmentOpNew`).
    Assignment {
        target: Expr,
        value: Expr,
    },
    /// `name:`, and also `"name":`. Task 3.4 already ended the clause at the
    /// colon.
    Label {
        name: Box<[u8]>,
    },
    /// A standalone message send, `q~append(1)`, and the message-assignment
    /// forms `q[1] = 2` and `q[1] += 2`.
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
        false_target: Option<usize>,
    },
    Then,
    Else {
        /// Where control goes when the `THEN` branch finished, which is the
        /// instruction after the `ELSE` branch. `None` is the end of this body.
        then_exit: Option<usize>,
    },
    Select {
        label: Option<SymbolId>,
        /// `SELECT CASE expr`, a different instruction class in the C++.
        case: Option<Expr>,
        /// The `WHEN` instructions this `SELECT` collected, in source order.
        whens: Vec<usize>,
        otherwise: Option<usize>,
        /// The `END` that closes this `SELECT`.
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
        exit: Option<usize>,
    },
    /// A `WHEN` inside `SELECT CASE`: `RexxInstructionCaseWhen`, a different
    /// class from `RexxInstructionIf` because the clause means something else.
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum VariableRef {
    /// A name written out: a simple variable, a stem, or a compound. Which
    /// of the three follows from the spelling, as it does for `ExprKind`.
    Direct(SymbolId),
    /// `(name)`, where the *value* of `name` names the variable to act on.
    Indirect(SymbolId),
}

/// A `DO` or `LOOP` header.
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
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Directive {
    pub kind: DirectiveKind,
    /// Byte range in the retained source: the directive clause.
    pub clause_span: Range<usize>,
}

/// One directive form, one variant per row of `RexxToken::directives[]`
/// (`KeywordConstants.cpp:52`-`63`).
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
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExternalSpec {
    /// True for the `REGISTERED` spelling, which only `::ROUTINE` accepts and
    /// which resolves an old-style external function instead of a library
    /// entry point.
    pub registered: bool,
    /// The library name, as written after the first word.
    pub library: Box<[u8]>,
    /// The entry point, when the specification named a third word.
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OptionsForm {
    Scientific,
    Engineering,
}

/// Which condition a `::OPTIONS` condition option selects.
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
