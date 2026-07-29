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

//! Phase 3 gate: every `Instruction` and `Expr` variant is constructed at
//! least once by parsing `rust/corpus/lang/` and `samples/` together, asserted
//! by enumerating the variants rather than by inspection.
//!
//! The enumeration cannot go stale. Each `tags!` invocation expands to a
//! `match` with no wildcard arm, so adding a variant to any of these enums
//! makes this file fail to compile, and the tag list the assertion checks
//! against is generated from the same invocation rather than written twice.
//!
//! `DirectiveKind` and the kind-bearing sub-enums (`LoopKind`, `Call`,
//! `Signal`, `Use`, `Trace`, `ParseSource`) are gated on the same machinery,
//! beyond the criterion's literal wording, because Phase 4 dispatches on those
//! too: the C++'s 52 instruction classes collapse into them, so "every
//! `InstructionKind` variant" alone would leave 23 loop classes covered by one
//! tag.

mod gate_walk;

use std::collections::HashSet;

use gate_walk::{
    corpus_dir, each_directive, each_expr, each_instruction, rex_files_under, samples_dir,
};
use rexx_parse::{
    Call, DirectiveKind, ExprKind, InstructionKind, LoopKind, ParseSource, Signal, Trace, Use,
    parse_program,
};

/// Expands to a tag function whose `match` has no wildcard arm, plus the list
/// of every tag it can produce. The two come from one invocation so they
/// cannot drift apart, and the missing wildcard is what makes a new variant a
/// compile error here rather than a silently shrinking check.
macro_rules! tags {
    ($fn_name:ident, $list:ident, $ty:ty, { $($pat:pat => $name:literal),+ $(,)? }) => {
        fn $fn_name(k: &$ty) -> &'static str {
            match k {
                $($pat => $name),+
            }
        }
        const $list: &[&str] = &[$($name),+];
    };
}

tags!(instruction_tag, INSTRUCTION_TAGS, InstructionKind, {
    InstructionKind::Assignment { .. } => "Assignment",
    InstructionKind::Label { .. } => "Label",
    InstructionKind::Message { .. } => "Message",
    InstructionKind::Command { .. } => "Command",
    InstructionKind::Do(_) => "Do",
    InstructionKind::Loop(_) => "Loop",
    InstructionKind::If { .. } => "If",
    InstructionKind::Then => "Then",
    InstructionKind::Else { .. } => "Else",
    InstructionKind::Select { .. } => "Select",
    InstructionKind::When { .. } => "When",
    InstructionKind::WhenCase { .. } => "WhenCase",
    InstructionKind::Otherwise => "Otherwise",
    InstructionKind::Leave { .. } => "Leave",
    InstructionKind::Iterate { .. } => "Iterate",
    InstructionKind::End { .. } => "End",
    InstructionKind::Drop { .. } => "Drop",
    InstructionKind::Expose { .. } => "Expose",
    InstructionKind::Parse(_) => "Parse",
    InstructionKind::Arg(_) => "Arg",
    InstructionKind::Pull(_) => "Pull",
    InstructionKind::Push { .. } => "Push",
    InstructionKind::Queue { .. } => "Queue",
    InstructionKind::Say { .. } => "Say",
    InstructionKind::Call(_) => "Call",
    InstructionKind::Return { .. } => "Return",
    InstructionKind::Procedure { .. } => "Procedure",
    InstructionKind::Signal(_) => "Signal",
    InstructionKind::Exit { .. } => "Exit",
    InstructionKind::Interpret { .. } => "Interpret",
    InstructionKind::Guard(_) => "Guard",
    InstructionKind::Reply { .. } => "Reply",
    InstructionKind::Forward(_) => "Forward",
    InstructionKind::Raise(_) => "Raise",
    InstructionKind::Use(_) => "Use",
    InstructionKind::Numeric { .. } => "Numeric",
    InstructionKind::Address(_) => "Address",
    InstructionKind::Trace(_) => "Trace",
    InstructionKind::Options { .. } => "Options",
    InstructionKind::Nop => "Nop",
});

tags!(expr_tag, EXPR_TAGS, ExprKind, {
    ExprKind::Literal(_) => "Literal",
    ExprKind::Constant(_) => "Constant",
    ExprKind::Variable(_) => "Variable",
    ExprKind::Stem(_) => "Stem",
    ExprKind::Compound(_) => "Compound",
    ExprKind::DotVariable(_) => "DotVariable",
    ExprKind::Prefix { .. } => "Prefix",
    ExprKind::Binary { .. } => "Binary",
    ExprKind::Call { .. } => "Call",
    ExprKind::QualifiedCall { .. } => "QualifiedCall",
    ExprKind::ClassResolver { .. } => "ClassResolver",
    ExprKind::Message { .. } => "Message",
    ExprKind::List(_) => "List",
    ExprKind::Logical(_) => "Logical",
    ExprKind::VariableReference(_) => "VariableReference",
});

tags!(directive_tag, DIRECTIVE_TAGS, DirectiveKind, {
    DirectiveKind::Annotate(_) => "Annotate",
    DirectiveKind::Attribute(_) => "Attribute",
    DirectiveKind::Class(_) => "Class",
    DirectiveKind::Constant(_) => "Constant",
    DirectiveKind::Method(_) => "Method",
    DirectiveKind::Options(_) => "Options",
    DirectiveKind::Requires(_) => "Requires",
    DirectiveKind::Resource(_) => "Resource",
    DirectiveKind::Routine(_) => "Routine",
});

tags!(loop_tag, LOOP_TAGS, LoopKind, {
    LoopKind::Simple => "Simple",
    LoopKind::Forever => "Forever",
    LoopKind::Count(_) => "Count",
    LoopKind::Controlled(_) => "Controlled",
    LoopKind::Over { .. } => "Over",
    LoopKind::With { .. } => "With",
});

tags!(call_tag, CALL_TAGS, Call, {
    Call::Named { .. } => "Named",
    Call::Dynamic { .. } => "Dynamic",
    Call::Qualified { .. } => "Qualified",
    Call::Trap(_) => "Trap",
});

tags!(signal_tag, SIGNAL_TAGS, Signal, {
    Signal::Label(_) => "Label",
    Signal::Value(_) => "Value",
    Signal::Trap(_) => "Trap",
});

tags!(use_tag, USE_TAGS, Use, {
    Use::Arg { .. } => "Arg",
    Use::Local { .. } => "Local",
});

tags!(trace_tag, TRACE_TAGS, Trace, {
    Trace::Default => "Default",
    Trace::Setting(_) => "Setting",
    Trace::Skip(_) => "Skip",
    Trace::Value(_) => "Value",
});

tags!(parse_source_tag, PARSE_SOURCE_TAGS, ParseSource, {
    ParseSource::Arg => "Arg",
    ParseSource::LineIn => "LineIn",
    ParseSource::Pull => "Pull",
    ParseSource::Source => "Source",
    ParseSource::Version => "Version",
    ParseSource::Var(_) => "Var",
    ParseSource::Value(_) => "Value",
});

/// One category's seen-set against its full list.
struct Coverage {
    category: &'static str,
    all: &'static [&'static str],
    seen: HashSet<&'static str>,
}

impl Coverage {
    fn new(category: &'static str, all: &'static [&'static str]) -> Self {
        Coverage {
            category,
            all,
            seen: HashSet::new(),
        }
    }

    fn missing(&self) -> Vec<&'static str> {
        self.all
            .iter()
            .copied()
            .filter(|t| !self.seen.contains(t))
            .collect()
    }
}

#[test]
fn every_variant_is_constructed_by_the_corpus_and_samples() {
    let mut instructions = Coverage::new("InstructionKind", INSTRUCTION_TAGS);
    let mut exprs = Coverage::new("ExprKind", EXPR_TAGS);
    let mut directives = Coverage::new("DirectiveKind", DIRECTIVE_TAGS);
    let mut loops = Coverage::new("LoopKind", LOOP_TAGS);
    let mut calls = Coverage::new("Call", CALL_TAGS);
    let mut signals = Coverage::new("Signal", SIGNAL_TAGS);
    let mut uses = Coverage::new("Use", USE_TAGS);
    let mut traces = Coverage::new("Trace", TRACE_TAGS);
    let mut parse_sources = Coverage::new("ParseSource", PARSE_SOURCE_TAGS);

    let mut files = rex_files_under(&corpus_dir());
    let corpus_count = files.len();
    files.extend(rex_files_under(&samples_dir()));
    // A walk that silently found nothing would pass vacuously.
    assert!(corpus_count >= 14, "corpus went missing: {corpus_count}");
    assert!(files.len() >= 250, "samples went missing: {}", files.len());

    for path in &files {
        let text = std::fs::read(path).expect("readable program");
        let p = parse_program(text)
            .unwrap_or_else(|e| panic!("{} failed to parse: {e:?}", path.display()));
        each_instruction(&p, &mut |i| {
            instructions.seen.insert(instruction_tag(&i.kind));
            match &i.kind {
                InstructionKind::Do(l) | InstructionKind::Loop(l) => {
                    loops.seen.insert(loop_tag(&l.kind));
                }
                InstructionKind::Call(c) => {
                    calls.seen.insert(call_tag(c));
                }
                InstructionKind::Signal(s) => {
                    signals.seen.insert(signal_tag(s));
                }
                InstructionKind::Use(u) => {
                    uses.seen.insert(use_tag(u));
                }
                InstructionKind::Trace(t) => {
                    traces.seen.insert(trace_tag(t));
                }
                InstructionKind::Parse(p) | InstructionKind::Arg(p) | InstructionKind::Pull(p) => {
                    parse_sources.seen.insert(parse_source_tag(&p.source));
                }
                _ => {}
            }
        });
        each_directive(&p, &mut |d| {
            directives.seen.insert(directive_tag(&d.kind));
        });
        each_expr(&p, &mut |e| {
            exprs.seen.insert(expr_tag(&e.kind));
        });
    }

    let mut report = String::new();
    for cov in [
        &instructions,
        &exprs,
        &directives,
        &loops,
        &calls,
        &signals,
        &uses,
        &traces,
        &parse_sources,
    ] {
        let missing = cov.missing();
        if !missing.is_empty() {
            report.push_str(&format!(
                "{}: {} of {} variants never constructed: {}\n",
                cov.category,
                missing.len(),
                cov.all.len(),
                missing.join(", ")
            ));
        }
    }
    assert!(
        report.is_empty(),
        "variants not reached by corpus/lang + samples:\n{report}"
    );
}
