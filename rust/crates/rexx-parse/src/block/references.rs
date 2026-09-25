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

//! Every variable reference an instruction makes, by name.

use crate::ast::{
    Call, Expr, ExprKind, Instruction, InstructionKind, LoopKind, ParseSource, Redirection, Signal,
    Trace, Use, VariableRef,
};
use crate::token::{SymbolId, SymbolTable};

/// Calls `f` with the name of every variable REFERENCE in `instruction`.
pub(super) fn for_each_variable_name(
    instruction: &Instruction,
    symbols: &SymbolTable,
    f: &mut impl FnMut(&str),
) {
    match &instruction.kind {
        InstructionKind::Assignment { target, value } => {
            visit_expr(target, symbols, f);
            visit_expr(value, symbols, f);
        }
        InstructionKind::Message { term, value } => {
            visit_expr(term, symbols, f);
            visit_opt(value, symbols, f);
        }
        InstructionKind::Command { expression }
        | InstructionKind::Push { expression }
        | InstructionKind::Queue { expression }
        | InstructionKind::Say { expression }
        | InstructionKind::Return { expression }
        | InstructionKind::Exit { expression }
        | InstructionKind::Reply { expression } => visit_opt(expression, symbols, f),
        InstructionKind::Interpret { expression } | InstructionKind::Options { expression } => {
            visit_expr(expression, symbols, f);
        }
        InstructionKind::If { condition, .. } | InstructionKind::When { condition, .. } => {
            visit_expr(condition, symbols, f);
        }
        InstructionKind::WhenCase { values, .. } => {
            for value in values {
                visit_expr(value, symbols, f);
            }
        }
        InstructionKind::Do(body) | InstructionKind::Loop(body) => {
            // `label` is deliberately absent: a loop label names the block, not
            // a variable. `counter` and every control variable are variables.
            visit_slot(body.counter, symbols, f);
            match &body.kind {
                LoopKind::Simple | LoopKind::Forever => {}
                LoopKind::Count(count) => visit_opt(count, symbols, f),
                LoopKind::Controlled(controlled) => {
                    visit_slot(Some(controlled.control), symbols, f);
                    visit_expr(&controlled.initial, symbols, f);
                    visit_opt(&controlled.to, symbols, f);
                    visit_opt(&controlled.by, symbols, f);
                    visit_opt(&controlled.for_count, symbols, f);
                }
                LoopKind::Over {
                    control,
                    target,
                    for_count,
                } => {
                    visit_slot(Some(*control), symbols, f);
                    visit_expr(target, symbols, f);
                    visit_opt(for_count, symbols, f);
                }
                LoopKind::With {
                    index,
                    item,
                    target,
                    for_count,
                } => {
                    visit_slot(*index, symbols, f);
                    visit_slot(*item, symbols, f);
                    visit_expr(target, symbols, f);
                    visit_opt(for_count, symbols, f);
                }
            }
            if let Some(conditional) = &body.conditional {
                visit_expr(&conditional.condition, symbols, f);
            }
        }
        InstructionKind::Drop { variables }
        | InstructionKind::Expose { variables }
        | InstructionKind::Procedure { variables } => visit_refs(variables, symbols, f),
        InstructionKind::Parse(body) | InstructionKind::Arg(body) | InstructionKind::Pull(body) => {
            match &body.source {
                ParseSource::Var(id) => visit_slot(Some(*id), symbols, f),
                ParseSource::Value(value) => visit_opt(value, symbols, f),
                ParseSource::Arg
                | ParseSource::LineIn
                | ParseSource::Pull
                | ParseSource::Source
                | ParseSource::Version => {}
            }
            for trigger in body.template.iter().flatten() {
                visit_opt(&trigger.value, symbols, f);
                visit_list(&trigger.targets, symbols, f);
            }
        }
        InstructionKind::Call(call) => match call.as_ref() {
            // A routine name is not a variable, whichever spelling it took.
            Call::Named { args, .. } | Call::Qualified { args, .. } => visit_list(args, symbols, f),
            Call::Dynamic { target, args } => {
                visit_expr(target, symbols, f);
                visit_list(args, symbols, f);
            }
            Call::Trap(_) => {}
        },
        InstructionKind::Signal(signal) => match signal.as_ref() {
            // A label, and a trap's label, name instructions rather than
            // variables. `SIGNAL VALUE` evaluates an expression.
            Signal::Label(_) | Signal::Trap(_) => {}
            Signal::Value(value) => visit_expr(value, symbols, f),
        },
        InstructionKind::Guard(guard) => visit_opt(&guard.condition, symbols, f),
        InstructionKind::Forward(forward) => {
            visit_opt(&forward.to, symbols, f);
            visit_opt(&forward.message, symbols, f);
            visit_opt(&forward.class, symbols, f);
            visit_opt(&forward.arguments, symbols, f);
            if let Some(array) = &forward.array {
                visit_list(array, symbols, f);
            }
        }
        InstructionKind::Raise(raise) => {
            visit_opt(&raise.rc, symbols, f);
            visit_opt(&raise.description, symbols, f);
            visit_opt(&raise.additional, symbols, f);
            if let Some(array) = &raise.array {
                visit_list(array, symbols, f);
            }
            if let Some(result) = &raise.result {
                visit_opt(&result.value, symbols, f);
            }
        }
        InstructionKind::Use(use_) => match use_.as_ref() {
            Use::Arg { targets, .. } => {
                for target in targets.iter().flatten() {
                    visit_expr(&target.target, symbols, f);
                    visit_opt(&target.default, symbols, f);
                }
            }
            Use::Local { variables } => visit_refs(variables, symbols, f),
        },
        InstructionKind::Numeric { expression, .. } => visit_opt(expression, symbols, f),
        InstructionKind::Address(address) => {
            // `environment` is a name rather than a variable.
            visit_opt(&address.dynamic, symbols, f);
            visit_opt(&address.command, symbols, f);
            if let Some(io) = &address.io {
                for redirection in [&io.input, &io.output, &io.error] {
                    match redirection {
                        Redirection::Stem(id) => visit_slot(Some(*id), symbols, f),
                        Redirection::Stream(value) | Redirection::Using(value) => {
                            visit_expr(value, symbols, f)
                        }
                        Redirection::Default | Redirection::Normal => {}
                    }
                }
            }
        }
        InstructionKind::Trace(trace) => match trace {
            Trace::Value(value) => visit_expr(value, symbols, f),
            Trace::Default | Trace::Setting(_) | Trace::Skip(_) => {}
        },
        // No variable reference of any kind. A block name on an `END`, `LEAVE`,
        // `ITERATE` or `SELECT` names a block, and a label names itself.
        InstructionKind::Label { .. }
        | InstructionKind::Then
        | InstructionKind::Else { .. }
        | InstructionKind::Otherwise
        | InstructionKind::Leave { .. }
        | InstructionKind::Iterate { .. }
        | InstructionKind::End { .. }
        | InstructionKind::Nop => {}
        InstructionKind::Select { case, .. } => visit_opt(case, symbols, f),
    }
}

/// `for_each_variable_in_expr` for an optional expression.
fn visit_opt(expr: &Option<Expr>, symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    if let Some(expr) = expr {
        visit_expr(expr, symbols, f);
    }
}

/// `for_each_variable_in_expr` over an argument list, whose omitted positions
/// hold no node.
fn visit_list(args: &[Option<Expr>], symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    for arg in args.iter().flatten() {
        visit_expr(arg, symbols, f);
    }
}

/// A bare variable slot, which is a name and not an expression.
fn visit_slot(id: Option<SymbolId>, symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    if let Some(id) = id {
        f(symbols.name(id));
    }
}

/// A `DROP`, `EXPOSE`, `PROCEDURE EXPOSE` or `USE LOCAL` list. Both spellings
/// reach `addVariable`, the indirect one through the symbol inside the
/// parentheses.
fn visit_refs(variables: &[VariableRef], symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    for variable in variables {
        let (VariableRef::Direct(id) | VariableRef::Indirect(id)) = variable;
        f(symbols.name(*id));
    }
}

/// Calls `f` with the name of every variable reference in one expression.
fn visit_expr(expr: &Expr, symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    let mut stack: Vec<&Expr> = vec![expr];
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
                f(symbols.name(*id));
            }
            _ => {}
        }
        expr.kind.for_each_child(&mut |child| stack.push(child));
    }
}
