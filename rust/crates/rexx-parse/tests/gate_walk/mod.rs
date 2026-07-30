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

//! The AST walk shared by the Phase 3 gate tests (`tiling.rs`, `variants.rs`).
//!
//! Every `match` in this module is exhaustive on purpose: a new `ExprKind`,
//! `InstructionKind` or `DirectiveKind` variant makes this module fail to
//! compile, which is what forces the gate tests to learn about it rather than
//! silently skipping it.

// Each test binary compiles its own copy of this module, and neither binary
// uses every item: `tiling.rs` never lists the samples, `variants.rs` never
// walks children directly. The allowance is per-binary surplus in a shared
// test module, not a not-yet-called library item, so the gate criterion that
// every `allow(dead_code)` under `src/` names the task that deletes it does
// not apply here and the grep that enforces it does not look here.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use rexx_parse::{
    Call, CodeBody, ConstantValue, Directive, DirectiveKind, Expr, ExprKind, Forward, Guard,
    Instruction, InstructionKind, Loop, LoopKind, Parse, ParseSource, Program, Raise, Redirection,
    Signal, Trace, Use,
};

/// Calls `f` on each direct child expression of `expr`, in source order.
///
/// Reimplements the crate-private `ExprKind::for_each_child` from the public
/// field surface, because an integration test cannot reach the private one.
/// An omitted argument has no node and is skipped.
pub fn children_of<'a>(expr: &'a Expr, f: &mut impl FnMut(&'a Expr)) {
    match &expr.kind {
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

/// Calls `f` on each top-level expression an instruction holds.
///
/// Top-level means the expressions the instruction owns directly; recursing
/// into their children is `children_of`'s job. The match is exhaustive so a
/// new instruction variant cannot be skipped silently.
pub fn exprs_of_instruction<'a>(kind: &'a InstructionKind, f: &mut impl FnMut(&'a Expr)) {
    let opt = |e: &'a Option<Expr>, f: &mut dyn FnMut(&'a Expr)| {
        if let Some(e) = e {
            f(e);
        }
    };
    match kind {
        InstructionKind::Assignment { target, value } => {
            f(target);
            f(value);
        }
        InstructionKind::Message { term, value } => {
            f(term);
            opt(value, f);
        }
        InstructionKind::Command { expression }
        | InstructionKind::Push { expression }
        | InstructionKind::Queue { expression }
        | InstructionKind::Say { expression }
        | InstructionKind::Return { expression }
        | InstructionKind::Exit { expression }
        | InstructionKind::Reply { expression }
        | InstructionKind::Numeric { expression, .. } => opt(expression, f),
        InstructionKind::Do(l) | InstructionKind::Loop(l) => exprs_of_loop(l, f),
        InstructionKind::If { condition, .. } | InstructionKind::When { condition, .. } => {
            f(condition)
        }
        InstructionKind::WhenCase { values, .. } => {
            for v in values {
                f(v);
            }
        }
        InstructionKind::Select { case, .. } => opt(case, f),
        InstructionKind::Label { .. }
        | InstructionKind::Then
        | InstructionKind::Else { .. }
        | InstructionKind::Otherwise
        | InstructionKind::Leave { .. }
        | InstructionKind::Iterate { .. }
        | InstructionKind::End { .. }
        | InstructionKind::Drop { .. }
        | InstructionKind::Expose { .. }
        | InstructionKind::Procedure { .. }
        | InstructionKind::Nop => {}
        InstructionKind::Parse(p) | InstructionKind::Arg(p) | InstructionKind::Pull(p) => {
            exprs_of_parse(p, f)
        }
        InstructionKind::Call(c) => match &**c {
            Call::Named { args, .. } => {
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            Call::Dynamic { target, args } => {
                f(target);
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            Call::Qualified { args, .. } => {
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            Call::Trap(_) => {}
        },
        InstructionKind::Signal(s) => match &**s {
            Signal::Value(e) => f(e),
            Signal::Label(_) | Signal::Trap(_) => {}
        },
        InstructionKind::Interpret { expression } | InstructionKind::Options { expression } => {
            f(expression)
        }
        InstructionKind::Guard(g) => {
            let Guard { condition, .. } = &**g;
            opt(condition, f);
        }
        InstructionKind::Forward(fw) => {
            let Forward {
                to,
                message,
                class,
                arguments,
                array,
                continue_: _,
            } = &**fw;
            opt(to, f);
            opt(message, f);
            opt(class, f);
            opt(arguments, f);
            if let Some(items) = array {
                for item in items.iter().flatten() {
                    f(item);
                }
            }
        }
        InstructionKind::Raise(r) => {
            let Raise {
                condition: _,
                propagate: _,
                rc,
                description,
                additional,
                array,
                result,
            } = &**r;
            opt(rc, f);
            opt(description, f);
            opt(additional, f);
            if let Some(items) = array {
                for item in items.iter().flatten() {
                    f(item);
                }
            }
            if let Some(result) = result {
                opt(&result.value, f);
            }
        }
        InstructionKind::Use(u) => match &**u {
            Use::Arg { targets, .. } => {
                for t in targets.iter().flatten() {
                    f(&t.target);
                    opt(&t.default, f);
                }
            }
            Use::Local { .. } => {}
        },
        InstructionKind::Address(a) => {
            opt(&a.dynamic, f);
            opt(&a.command, f);
            if let Some(io) = &a.io {
                for r in [&io.input, &io.output, &io.error] {
                    match r {
                        Redirection::Stream(e) | Redirection::Using(e) => f(e),
                        Redirection::Default | Redirection::Normal | Redirection::Stem(_) => {}
                    }
                }
            }
        }
        InstructionKind::Trace(t) => match t {
            Trace::Value(e) => f(e),
            Trace::Default | Trace::Setting(_) | Trace::Skip(_) => {}
        },
    }
}

/// The expressions of a `DO`/`LOOP` header, in source order.
fn exprs_of_loop<'a>(l: &'a Loop, f: &mut impl FnMut(&'a Expr)) {
    match &l.kind {
        LoopKind::Simple | LoopKind::Forever => {}
        LoopKind::Count(e) => {
            if let Some(e) = e {
                f(e);
            }
        }
        LoopKind::Controlled(c) => {
            f(&c.initial);
            // The keyword expressions in written order, which `order` records.
            for e in [&c.to, &c.by, &c.for_count].into_iter().flatten() {
                f(e);
            }
        }
        LoopKind::Over {
            target, for_count, ..
        }
        | LoopKind::With {
            target, for_count, ..
        } => {
            f(target);
            if let Some(e) = for_count {
                f(e);
            }
        }
    }
    if let Some(cond) = &l.conditional {
        f(&cond.condition);
    }
}

/// The expressions of a `PARSE`/`ARG`/`PULL` instruction.
fn exprs_of_parse<'a>(p: &'a Parse, f: &mut impl FnMut(&'a Expr)) {
    if let ParseSource::Value(Some(e)) = &p.source {
        f(e);
    }
    for trigger in p.template.iter().flatten() {
        if let Some(e) = &trigger.value {
            f(e);
        }
        for target in trigger.targets.iter().flatten() {
            f(target);
        }
    }
}

/// The code body a directive carries, if it carries one.
pub fn body_of_directive(kind: &DirectiveKind) -> Option<&CodeBody> {
    match kind {
        DirectiveKind::Method(m) => m.body.as_ref(),
        DirectiveKind::Attribute(a) => a.body.as_ref(),
        DirectiveKind::Routine(r) => r.body.as_ref(),
        DirectiveKind::Annotate(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_) => None,
    }
}

/// The expression a directive holds outside any body: only `::CONSTANT (expr)`
/// has one.
pub fn expr_of_directive(kind: &DirectiveKind) -> Option<&Expr> {
    match kind {
        DirectiveKind::Constant(c) => match &c.value {
            ConstantValue::Expression(e) => Some(e),
            ConstantValue::Name | ConstantValue::Text(_) => None,
        },
        DirectiveKind::Annotate(_)
        | DirectiveKind::Attribute(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Method(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_)
        | DirectiveKind::Routine(_) => None,
    }
}

/// Calls `visit` on every instruction of the program: the main body's, then
/// each directive body's, in source order.
pub fn each_instruction<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Instruction)) {
    for i in &p.main.instructions {
        visit(i);
    }
    for d in &p.directives {
        if let Some(body) = body_of_directive(&d.kind) {
            for i in &body.instructions {
                visit(i);
            }
        }
    }
}

/// Calls `visit` on every directive of the program, in source order.
pub fn each_directive<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Directive)) {
    for d in &p.directives {
        visit(d);
    }
}

/// Calls `visit` on every expression in the program, parents before children.
pub fn each_expr<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Expr)) {
    // An explicit stack rather than recursion through `children_of`, because
    // a left-leaning expression tree the width of one clause overflows the
    // stack here otherwise: `deep_nested_expr.rex`'s 3000-term chain aborted
    // this walk on a default 2 MiB thread before Task 3b (see its report).
    // Pushing a node's children in reverse and popping is what keeps this a
    // parents-before-children, left-to-right walk exactly like the recursive
    // version: the leftmost child is pushed last, so it is popped, and so
    // fully visited depth-first, before its next sibling.
    fn walk<'a>(root: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            visit(e);
            let mut children: Vec<&'a Expr> = Vec::new();
            children_of(e, &mut |child| children.push(child));
            stack.extend(children.into_iter().rev());
        }
    }
    each_instruction(p, &mut |i| {
        exprs_of_instruction(&i.kind, &mut |e| walk(e, visit));
    });
    for d in &p.directives {
        if let Some(e) = expr_of_directive(&d.kind) {
            walk(e, visit);
        }
    }
}

/// Every `*.rex` file under `dir`, recursively, in a stable order.
pub fn rex_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("cannot read directory {}: {e}", d.display()))
        {
            let path = entry.expect("readable directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rex") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// The corpus directory, `rust/corpus/lang/`.
pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang")
}

/// The samples directory at the repository root.
pub fn samples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples")
}
