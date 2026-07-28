//! The tree's own invariants, apart from any parse.
//!
//! The grammar's tests in `expr/tests.rs` check spans on real input. These
//! check that the construction guarantees hold even for an extent a caller got
//! wrong, which is what makes the containment property structural.

use super::{CallTarget, Expr, ExprKind, PrefixOp, Tail, compound_parts};
use crate::token::{Operator, SymbolTable};

fn leaf(id: u32, span: std::ops::Range<usize>) -> Expr {
    // A `SymbolId` is only compared, never dereferenced, by anything under
    // test here, so a table is interned to get real ids rather than faked.
    let mut symbols = SymbolTable::default();
    let mut last = symbols.intern("x0");
    for i in 1..=id {
        last = symbols.intern(&format!("x{i}"));
    }
    Expr::new(ExprKind::Variable(last), span)
}

#[test]
fn a_nodes_span_is_widened_to_cover_every_child() {
    // The extent given here covers neither child, which is the case the
    // widening exists for: a node must never claim a range narrower than what
    // it contains, whatever the caller computed.
    let left = leaf(0, 10..12);
    let right = leaf(1, 30..34);
    let node = Expr::new(
        ExprKind::Binary {
            op: Operator::Plus,
            left: Box::new(left),
            right: Box::new(right),
        },
        20..21,
    );
    assert_eq!(node.span, 10..34);
}

#[test]
fn a_binary_nodes_span_runs_from_its_left_operand_to_its_right() {
    let left = leaf(0, 4..5);
    let right = leaf(1, 8..9);
    let node = Expr::binary(Operator::Multiply, left, right);
    assert_eq!(node.span, 4..9);
}

#[test]
fn an_omitted_argument_is_no_child_and_widens_nothing() {
    let node = Expr::new(
        ExprKind::Call {
            target: CallTarget::Literal(Box::from(&b"f"[..])),
            args: vec![None, Some(leaf(0, 7..8)), None],
        },
        0..9,
    );
    let mut seen = Vec::new();
    node.kind
        .for_each_child(&mut |child| seen.push(child.span.clone()));
    assert_eq!(seen, vec![7..8], "only the one real argument is a child");
    assert_eq!(node.span, 0..9);
}

#[test]
fn children_are_visited_in_source_order() {
    let node = Expr::new(
        ExprKind::Message {
            target: Box::new(leaf(0, 0..1)),
            name: Box::from(&b"M"[..]),
            super_class: Some(Box::new(leaf(1, 4..5))),
            args: vec![Some(leaf(2, 6..7)), Some(leaf(3, 8..9))],
            cascade: false,
        },
        0..10,
    );
    let mut seen = Vec::new();
    node.kind
        .for_each_child(&mut |child| seen.push(child.span.clone()));
    assert_eq!(seen, [0..1, 4..5, 6..7, 8..9]);
}

#[test]
fn a_prefix_extent_that_omits_its_operand_is_widened_to_include_it() {
    // The parser passes the operator token's own span, which never covers the
    // operand, so this is the ordinary case and not a corner one.
    let node = Expr::new(
        ExprKind::Prefix {
            op: PrefixOp::Minus,
            operand: Box::new(leaf(0, 1..4)),
        },
        0..1,
    );
    assert_eq!(node.span, 0..4);
}

#[test]
fn a_compound_name_splits_at_the_first_period_and_keeps_it_on_the_stem() {
    // Matches `addCompound` (`LanguageParser.cpp:2153`), which builds the stem
    // as `start` up to and including the period it stopped on.
    assert_eq!(
        compound_parts("A.B.C"),
        ("A.", vec![Tail::Variable("B"), Tail::Variable("C")])
    );
    assert_eq!(
        compound_parts("A.B.C.D"),
        (
            "A.",
            vec![
                Tail::Variable("B"),
                Tail::Variable("C"),
                Tail::Variable("D")
            ]
        )
    );
}

#[test]
fn a_tail_piece_that_cannot_be_a_variable_name_is_a_constant() {
    // `LanguageParser.cpp:2184`: empty, or starting with a digit. Measured with
    // `build/bin/rexx` and `b = 2`: `say a.1.b` prints `A.1.2`, so `1` stood
    // for itself where `B` was looked up, and `say a..b` prints `A..2`.
    assert_eq!(
        compound_parts("A.1.B"),
        ("A.", vec![Tail::Constant("1"), Tail::Variable("B")])
    );
    assert_eq!(
        compound_parts("A..B"),
        ("A.", vec![Tail::Constant(""), Tail::Variable("B")])
    );
    // A trailing period gives a final empty piece rather than dropping one.
    assert_eq!(
        compound_parts("A.B."),
        ("A.", vec![Tail::Variable("B"), Tail::Constant("")])
    );
    // A digit anywhere but the front does not make a constant: `1B` cannot be
    // a symbol, but `B1` can.
    assert_eq!(compound_parts("A.B1"), ("A.", vec![Tail::Variable("B1")]));
}

#[test]
fn the_first_period_ends_the_stem_however_many_follow() {
    // A whole-name split would give the wrong stem for `A.B.C`: the stem is
    // `A.` and not `A.B.`.
    let (stem, tails) = compound_parts("STEM.I.J.K");
    assert_eq!(stem, "STEM.");
    assert_eq!(tails.len(), 3);
}

#[test]
fn a_shape_renders_a_literal_and_a_constant_differently() {
    // A rendering that could not tell `'2'` from `2` would make every shape
    // assertion in `expr/tests.rs` weaker than it looks.
    let mut symbols = SymbolTable::default();
    let two = symbols.intern("2");
    let constant = Expr::new(ExprKind::Constant(two), 0..1);
    let literal = Expr::new(ExprKind::Literal(Box::from(&b"2"[..])), 0..3);
    assert_eq!(constant.shape(&symbols), "2");
    assert_eq!(literal.shape(&symbols), "'2'");
    assert_ne!(constant.shape(&symbols), literal.shape(&symbols));
}

#[test]
fn a_shape_distinguishes_the_four_variable_forms() {
    let mut symbols = SymbolTable::default();
    let simple = symbols.intern("a");
    let stem = symbols.intern("a.");
    let compound = symbols.intern("a.b");
    let dot = symbols.intern(".a");
    assert_eq!(
        Expr::new(ExprKind::Variable(simple), 0..1).shape(&symbols),
        "A"
    );
    assert_eq!(
        Expr::new(ExprKind::Stem(stem), 0..2).shape(&symbols),
        "stem:A."
    );
    assert_eq!(
        Expr::new(ExprKind::Compound(compound), 0..3).shape(&symbols),
        "compound:A.[var:B]"
    );
    assert_eq!(
        Expr::new(ExprKind::DotVariable(dot), 0..2).shape(&symbols),
        "env:.A"
    );
}
