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

//! Phase 3 gate: every `rust/corpus/lang/` program tiles.
//!
//! Three properties, each with its own checker so the can-fail probes at the
//! bottom can exercise them one at a time:
//!
//! 1. **Expressions nest**: every `Expr`'s span contains its children's spans.
//!    This holds **by construction**: `Expr::new` widens the extent it is
//!    given to cover each child before storing it, so no parseable input can
//!    violate it. The check is kept as a pin against a future change to
//!    `Expr::new`, not as evidence about the parser. See the gate assessment.
//! 2. **Binary operands are tight and ordered**, which IS falsifiable: the
//!    left operand ends at or before the right operand starts, and the bytes
//!    between them are the operator (plus whitespace-class bytes) and nothing
//!    else. A mis-nesting that attaches an operand to the wrong operator
//!    keeps containment, because widening preserves it, and breaks this.
//!    The AST does not retain the operator token's own position, so the check
//!    is over the gap's byte class rather than an exact spelling: two
//!    spellings can scan to one operator (`\` and the two not-sign bytes),
//!    and either operand may sit in parentheses that belong to no node.
//! 3. **Instructions are ordered**: consecutive `clause_span`s are in source
//!    order, do not overlap, and the interstices between them hold only
//!    whitespace-class bytes: blanks, line terminators, comments, and `,`/`-`
//!    line continuations.
//!
//! A `;` is deliberately NOT whitespace-class here, although a null clause
//! (`;;`, or `;` alone on a line) would put one into an interstice: the
//! corpus has no null clause today, and permitting `;` would weaken the
//! dropped-clause net this criterion exists for. If a corpus program ever
//! gains one, or if this checker is ever extended to `samples/`, revisit that
//! choice first; the failure message says the same thing.

mod gate_walk;

use std::ops::Range;

use gate_walk::{body_of_directive, children_of, corpus_dir, each_expr, rex_files_under};
use rexx_parse::{Expr, ExprKind, Operator, Program, parse_program};

/// The non-whitespace-class bytes of `text[range]`, each with its offset.
///
/// Whitespace-class means: blank, tab, CR, LF; a `--` comment to end of line;
/// a `/* ... */` comment, which nests, exactly as the scanner's do; and a `,`
/// or `-` whose rest-of-line is only blanks and comments, which is a line
/// continuation. A continuation may equally sit inside a clause span, so
/// permitting it here keeps the check independent of where the splitter put
/// it. Everything else is returned for the caller to judge.
fn non_whitespace_class(text: &[u8], range: Range<usize>) -> Vec<(usize, u8)> {
    let mut out = Vec::new();
    let mut i = range.start;
    let end = range.end;
    while i < end {
        match text[i] {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'-' if i + 1 < end && text[i + 1] == b'-' => {
                // A `--` line comment runs to the line terminator.
                while i < end && text[i] != b'\r' && text[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < end && text[i + 1] == b'*' => {
                // Block comments nest (measured against the oracle by the
                // scanner's own tests), so track a depth.
                let mut depth = 1;
                let open = i;
                i += 2;
                while i < end && depth > 0 {
                    if text[i] == b'/' && i + 1 < end && text[i + 1] == b'*' {
                        depth += 1;
                        i += 2;
                    } else if text[i] == b'*' && i + 1 < end && text[i + 1] == b'/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if depth > 0 {
                    // Unclosed within the range: report the opener rather
                    // than silently swallowing the rest.
                    out.push((open, b'/'));
                }
            }
            b',' | b'-' => {
                // A continuation candidate: it counts as whitespace-class
                // only if nothing but blanks and comments sits between it and
                // the line terminator (or the end of the range).
                let mark = i;
                let rest = non_whitespace_class_line_tail(text, i + 1..end);
                match rest {
                    Some(next) => {
                        i = next;
                    }
                    None => {
                        out.push((mark, text[mark]));
                        i += 1;
                    }
                }
            }
            other => {
                out.push((i, other));
                i += 1;
            }
        }
    }
    out
}

/// If `text[from..]` up to its first line terminator holds only blanks and
/// comments, the offset just past that terminator; otherwise `None`.
fn non_whitespace_class_line_tail(text: &[u8], range: Range<usize>) -> Option<usize> {
    let mut i = range.start;
    let end = range.end;
    while i < end {
        match text[i] {
            b'\r' | b'\n' => return Some(i + 1),
            b' ' | b'\t' => i += 1,
            b'-' if i + 1 < end && text[i + 1] == b'-' => {
                while i < end && text[i] != b'\r' && text[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < end && text[i + 1] == b'*' => {
                let mut depth = 1;
                i += 2;
                while i < end && depth > 0 {
                    if text[i] == b'/' && i + 1 < end && text[i + 1] == b'*' {
                        depth += 1;
                        i += 2;
                    } else if text[i] == b'*' && i + 1 < end && text[i + 1] == b'/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if depth > 0 {
                    return None;
                }
            }
            _ => return None,
        }
    }
    // The range ended before any line terminator, so the candidate is not a
    // continuation at all. This branch is load-bearing: a binary gap like
    // `n - 1` ends at the right operand's span start, and answering `Some`
    // here would swallow the subtract operator as a phantom continuation,
    // which is exactly what the first version of this scanner did.
    None
}

/// Property 1: every node's span contains its children's spans.
///
/// The numbering here is this file's own and does not match
/// `docs/superpowers/plans/phase-3-gate.md`, which numbers the exit criterion's
/// two clauses so that its Property 2 is clause ordering, this file's Property
/// 3. The gate doc's Property 2 and this file's Property 2 are different
/// properties, so cite them by name rather than by number.
fn containment_errors(e: &Expr, errors: &mut Vec<String>) {
    children_of(e, &mut |child| {
        if child.span.start < e.span.start || child.span.end > e.span.end {
            errors.push(format!(
                "child span {:?} escapes parent span {:?}",
                child.span, e.span
            ));
        }
    });
}

/// A byte that can spell part of a dyadic operator: the ASCII operator
/// characters plus the two single-byte not-sign spellings the scanner maps to
/// `\` (`0xAA` and `0xAC`, see `Operator::spelling`).
fn is_operator_byte(b: u8) -> bool {
    matches!(
        b,
        b'+' | b'-' | b'*' | b'/' | b'%' | b'|' | b'&' | b'=' | b'<' | b'>' | b'\\' | 0xAA | 0xAC
    )
}

/// Property 2: a binary node's operands are ordered and the gap between them
/// holds the operator and nothing else.
///
/// The gap may also hold `(` and `)`, because parentheses belong to no node:
/// `(a) + (b)` leaves `) + (` between the operand spans. For the two
/// synthesised operators, abuttal and blank, the gap holds no operator byte
/// at all; for every real operator it holds at least one.
fn binary_tightness_errors(text: &[u8], e: &Expr, errors: &mut Vec<String>) {
    let ExprKind::Binary { op, left, right } = &e.kind else {
        return;
    };
    if left.span.end > right.span.start {
        errors.push(format!(
            "binary {:?}: left operand span {:?} overlaps right operand span {:?}",
            op, left.span, right.span
        ));
        return;
    }
    let gap = non_whitespace_class(text, left.span.end..right.span.start);
    let synthetic = matches!(op, Operator::Abuttal | Operator::Blank);
    let mut saw_operator_byte = false;
    for &(offset, byte) in &gap {
        if byte == b'(' || byte == b')' {
            continue;
        }
        if is_operator_byte(byte) {
            saw_operator_byte = true;
            continue;
        }
        errors.push(format!(
            "binary {:?}: byte {:?} at offset {} sits between the operand spans \
             {:?} and {:?} but cannot be part of an operator",
            op, byte as char, offset, left.span, right.span
        ));
    }
    if synthetic && saw_operator_byte {
        errors.push(format!(
            "binary {:?}: an operator byte sits between the operand spans {:?} and \
             {:?}, but this operator is synthesised and has no source spelling",
            op, left.span, right.span
        ));
    }
    if !synthetic && !saw_operator_byte {
        errors.push(format!(
            "binary {:?}: no operator byte between the operand spans {:?} and {:?}",
            op, left.span, right.span
        ));
    }
}

/// A prefix node's span starts before its operand's, because the operator
/// token sits in front. Falsifiable the same way property 2 is.
fn prefix_order_errors(e: &Expr, errors: &mut Vec<String>) {
    if let ExprKind::Prefix { operand, .. } = &e.kind
        && e.span.start >= operand.span.start
    {
        errors.push(format!(
            "prefix node span {:?} does not start before its operand span {:?}",
            e.span, operand.span
        ));
    }
}

/// Property 3: the clause spans are ordered, non-overlapping, and every
/// interstice holds only whitespace-class bytes.
fn tiling_errors(text: &[u8], spans: &[Range<usize>], errors: &mut Vec<String>) {
    let mut previous_end = 0;
    for span in spans {
        if span.start > span.end {
            errors.push(format!("clause span {span:?} runs backwards"));
            continue;
        }
        if span.start < previous_end {
            errors.push(format!(
                "clause span {span:?} starts before the previous clause ended at {previous_end}"
            ));
            continue;
        }
        for (offset, byte) in non_whitespace_class(text, previous_end..span.start) {
            let hint = if byte == b';' {
                " (a null clause produces no instruction, so its `;` lands in an \
                 interstice; decide whether to permit it or keep the stricter \
                 dropped-clause net before changing this checker)"
            } else {
                ""
            };
            errors.push(format!(
                "byte {:?} at offset {} sits between clause spans and belongs to \
                 no instruction{}",
                byte as char, offset, hint
            ));
        }
        previous_end = span.end;
    }
    for (offset, byte) in non_whitespace_class(text, previous_end..text.len()) {
        errors.push(format!(
            "byte {:?} at offset {} sits after the last clause span and belongs \
             to no node",
            byte as char, offset
        ));
    }
}

/// The program's clause spans in source order: the main body's instructions,
/// then each directive's own clause followed by its body's instructions.
fn clause_spans(p: &Program) -> Vec<Range<usize>> {
    let mut spans: Vec<Range<usize>> = p
        .main
        .instructions
        .iter()
        .map(|i| i.clause_span.clone())
        .collect();
    for d in &p.directives {
        spans.push(d.clause_span.clone());
        if let Some(body) = body_of_directive(&d.kind) {
            spans.extend(body.instructions.iter().map(|i| i.clause_span.clone()));
        }
    }
    spans
}

#[test]
fn every_corpus_program_tiles() {
    let files = rex_files_under(&corpus_dir());
    assert!(files.len() >= 14, "corpus went missing: {}", files.len());

    let mut failures = Vec::new();
    for path in &files {
        let text = std::fs::read(path).expect("readable corpus file");
        let p = parse_program(text.clone())
            .unwrap_or_else(|e| panic!("{} failed to parse: {e:?}", path.display()));

        let mut errors = Vec::new();
        each_expr(&p, &mut |e| {
            containment_errors(e, &mut errors);
            binary_tightness_errors(&text, e, &mut errors);
            prefix_order_errors(e, &mut errors);
        });
        tiling_errors(&text, &clause_spans(&p), &mut errors);

        for error in errors {
            failures.push(format!("{}: {error}", path.display()));
        }
    }
    assert!(
        failures.is_empty(),
        "{} tiling violations:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ---- The checkers can fail. ----
//
// Property 1 cannot be violated by any input, because `Expr::new` widens a
// node's span over its children, and properties 2 and 3 have no corpus
// counterexample either. So each checker is demonstrated against hand-built
// nodes, which the public field surface permits: these prove the CHECKERS
// reject what they exist to reject, and say nothing about the parser.

/// A literal leaf claiming `span`, for building violating trees by hand.
fn leaf(span: Range<usize>) -> Expr {
    Expr {
        kind: ExprKind::Literal(b"x".to_vec().into_boxed_slice()),
        span,
    }
}

fn binary(op: Operator, left: Expr, right: Expr, span: Range<usize>) -> Expr {
    Expr {
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        span,
    }
}

#[test]
fn the_containment_checker_rejects_a_child_escaping_its_parent() {
    let e = binary(Operator::Plus, leaf(0..1), leaf(4..9), 0..5);
    let mut errors = Vec::new();
    containment_errors(&e, &mut errors);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("escapes"), "{errors:?}");
}

#[test]
fn the_tightness_checker_rejects_a_mis_nested_operand() {
    // "a + b c": a tree that attached `b` to `+` but claims `c`'s span as the
    // right operand keeps containment under widening and must fail here,
    // because the bytes between the operand spans include the symbol `b`.
    let text = b"a + b c";
    let e = binary(Operator::Plus, leaf(0..1), leaf(6..7), 0..7);
    let mut errors = Vec::new();
    binary_tightness_errors(text, &e, &mut errors);
    assert!(
        errors
            .iter()
            .any(|m| m.contains("cannot be part of an operator")),
        "{errors:?}"
    );
}

#[test]
fn the_tightness_checker_rejects_overlapping_operands() {
    let text = b"a + b";
    let e = binary(Operator::Plus, leaf(0..3), leaf(2..5), 0..5);
    let mut errors = Vec::new();
    binary_tightness_errors(text, &e, &mut errors);
    assert!(errors.iter().any(|m| m.contains("overlaps")), "{errors:?}");
}

#[test]
fn the_tightness_checker_rejects_a_missing_operator() {
    // Both operand spans are right, but the operator between them is absent
    // from the source: a blank sits where `+` claims to be.
    let text = b"a   b";
    let e = binary(Operator::Plus, leaf(0..1), leaf(4..5), 0..5);
    let mut errors = Vec::new();
    binary_tightness_errors(text, &e, &mut errors);
    assert!(
        errors.iter().any(|m| m.contains("no operator byte")),
        "{errors:?}"
    );
}

#[test]
fn the_prefix_checker_rejects_an_operator_with_nothing_in_front() {
    let e = Expr {
        kind: ExprKind::Prefix {
            op: rexx_parse::PrefixOp::Minus,
            operand: Box::new(leaf(0..1)),
        },
        span: 0..1,
    };
    let mut errors = Vec::new();
    prefix_order_errors(&e, &mut errors);
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn the_tiling_checker_rejects_a_dropped_clause() {
    // Three clauses in the source, two in the chain: the dropped middle one
    // leaves its bytes in the interstice.
    let text = b"nop\nsay hi\nnop\n";
    let spans = [0..3, 11..14];
    let mut errors = Vec::new();
    tiling_errors(text, &spans, &mut errors);
    assert!(
        errors
            .iter()
            .any(|m| m.contains("belongs to no instruction")),
        "{errors:?}"
    );
}

#[test]
fn the_tiling_checker_rejects_overlapping_clause_spans() {
    let text = b"nop nop\n";
    let spans = [0..5, 4..7];
    let mut errors = Vec::new();
    tiling_errors(text, &spans, &mut errors);
    assert!(
        errors.iter().any(|m| m.contains("starts before")),
        "{errors:?}"
    );
}

#[test]
fn the_tiling_checker_rejects_an_interstitial_semicolon() {
    // A null clause's `;` is real Rexx and still rejected, by choice: see the
    // module comment. This test pins the choice so relaxing it is a visible
    // decision rather than a drive-by edit.
    let text = b"nop\n;\nnop\n";
    let spans = [0..3, 6..9];
    let mut errors = Vec::new();
    tiling_errors(text, &spans, &mut errors);
    assert!(
        errors.iter().any(|m| m.contains("null clause")),
        "{errors:?}"
    );
}

#[test]
fn the_tiling_checker_permits_comments_and_continuations_between_clauses() {
    // Interstices holding a block comment, a `--` comment, a `,` continuation
    // and a `-` continuation are all whitespace-class. The continuation bytes
    // sit in an interstice here; inside a continued clause they would sit
    // inside its span, and both placements are deliberately permitted.
    let text = b"nop\n/* a /* nested */ comment */\n-- line comment\n,\n-\nnop\n";
    let spans = [0..3, (text.len() - 4)..(text.len() - 1)];
    let mut errors = Vec::new();
    tiling_errors(text, &spans, &mut errors);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn the_tiling_checker_rejects_bytes_after_the_last_clause() {
    let text = b"nop\nsay hi\n";
    // Hoisted so the array literal is unambiguous to clippy's
    // single_range_in_vec_init, which cannot tell `[0..3]` from a
    // `[value; len]` repetition.
    let only_clause = 0..3;
    let spans = [only_clause];
    let mut errors = Vec::new();
    tiling_errors(text, &spans, &mut errors);
    assert!(
        errors.iter().any(|m| m.contains("after the last clause")),
        "{errors:?}"
    );
}
