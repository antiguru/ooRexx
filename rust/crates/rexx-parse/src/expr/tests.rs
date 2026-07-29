//! The expression grammar, pinned against `build/bin/rexx` and
//! `build/bin/rexxc`.
//!
//! Every precedence and associativity expectation below carries the probe that
//! produced it, as a comment naming the program and the answer. Where the
//! interpreter's *value* cannot tell two trees apart the comment says so
//! rather than implying evidence that does not exist.
//!
//! In-crate rather than under `tests/`, because `ParseCtx`, `TokenCursor` and
//! `parse_expr` are all `pub(crate)` and an integration test is a separate
//! crate.

use crate::ast::{Expr, ExprKind};
use crate::clause::split_clauses;
use crate::token::{ParseCtx, ParseError, SymbolTable, TokenCursor};
use crate::{ProgramSource, SourceKind, scan};

use super::{Terminators, parse_expr, parse_expression, parse_logical};

/// How the grammar is entered. One variant per entry point, so a test names
/// which one it is exercising.
#[derive(Copy, Clone)]
pub(super) enum Entry {
    /// `parse_expr`: required, end of clause only. 918 stands in for the
    /// sub-number an instruction parser passes, which for an assignment is
    /// 35.918.
    Required,
    /// `parse_expression` with a terminator set.
    Optional(Terminators),
    /// `parse_logical` with a terminator set.
    Logical(Terminators),
}

/// Parses `text` as one clause and returns the tree with the table its
/// `SymbolId`s belong to.
///
/// `text` is the whole program, so a span in the result indexes `text`
/// directly. That rules out an input whose first two tokens are a symbol and a
/// colon, because `split_clauses` reads that as a label. Wrap such an
/// expression in parentheses, which build no node.
pub(super) fn parse(text: &str, entry: Entry) -> Result<(Expr, SymbolTable), ParseError> {
    let source = ProgramSource::new(text.as_bytes().to_vec(), SourceKind::Program);
    let scanned = scan(&source).expect("the test input scans");
    let clauses = split_clauses(&scanned.tokens).expect("the test input splits");
    assert_eq!(
        clauses.len(),
        1,
        "{text:?} is not one clause, so a span would not index the whole text"
    );
    assert!(
        clauses[0].label.is_none(),
        "{text:?} was read as a label clause"
    );
    let result = {
        let ctx = ParseCtx {
            source: &source,
            tokens: &scanned.tokens,
            symbols: &scanned.symbols,
            keywords: &scanned.keywords,
            resources: &scanned.resources,
        };
        let mut cursor = TokenCursor::new(clauses[0].tokens.clone());
        match entry {
            Entry::Required => parse_expr(&ctx, &mut cursor, Terminators::EOC, 918),
            Entry::Optional(term) => parse_expression(&ctx, &mut cursor, term)
                .map(|e| e.expect("the test input is not an empty expression")),
            Entry::Logical(term) => parse_logical(&ctx, &mut cursor, term, 929),
        }
    };
    result.map(|expr| (expr, scanned.symbols))
}

/// The canonical shape of `text`, which must parse.
fn shape(text: &str) -> String {
    let (expr, symbols) =
        parse(text, Entry::Required).unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
    check_spans(&expr, text);
    expr.shape(&symbols)
}

/// The shape of `text` parsed with a terminator set, plus the source of
/// whatever the parse stopped before.
fn shape_until(text: &str, term: Terminators) -> String {
    let (expr, symbols) = parse(text, Entry::Optional(term))
        .unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
    check_spans(&expr, text);
    expr.shape(&symbols)
}

fn shape_logical(text: &str, term: Terminators) -> String {
    let (expr, symbols) = parse(text, Entry::Logical(term))
        .unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
    check_spans(&expr, text);
    expr.shape(&symbols)
}

/// The error number and sub-number `text` raises, which it must.
fn error(text: &str) -> (u16, u16) {
    let err = parse(text, Entry::Required)
        .err()
        .unwrap_or_else(|| panic!("{text:?} parsed, but an error was expected"));
    (err.code, err.sub)
}

fn logical_error(text: &str, term: Terminators) -> (u16, u16) {
    let err = parse(text, Entry::Logical(term))
        .err()
        .unwrap_or_else(|| panic!("{text:?} parsed, but an error was expected"));
    (err.code, err.sub)
}

/// Asserts the gate's span property over a whole tree: every node's span
/// contains each of its children's, and the root stays inside `text`.
pub(super) fn check_spans(expr: &Expr, text: &str) {
    assert!(
        expr.span.end <= text.len() && expr.span.start <= expr.span.end,
        "span {:?} is not a range inside {text:?}",
        expr.span
    );
    expr.kind.for_each_child(&mut |child| {
        assert!(
            expr.span.start <= child.span.start && child.span.end <= expr.span.end,
            "in {text:?}: node span {:?} does not contain child span {:?}",
            expr.span,
            child.span
        );
        check_spans(child, text);
    });
}

// ---------------------------------------------------------------------------
// Precedence, one test per boundary in `RexxToken::precedence`.
// ---------------------------------------------------------------------------

#[test]
fn prefix_minus_binds_tighter_than_power() {
    // build/bin/rexx: `r = -2 ** 2` => 4, where C and Python give -4.
    assert_eq!(shape("-2 ** 2"), "(** (u- 2) 2)");
    // And on both sides: `r = -2 ** -2` => 0.25, which is (-2) ** (-2).
    assert_eq!(shape("-2 ** -2"), "(** (u- 2) (u- 2))");
}

#[test]
fn prefix_not_binds_tighter_than_power() {
    // build/bin/rexx: `r = \0 ** 0` => 1, so this is (\0) ** 0 = 1 ** 0.
    // Parsing it as \(0 ** 0) would be \1 = 0.
    //
    // The exponent has to be 0 to discriminate: for any x in {0,1},
    // (\x) ** 2 and \(x ** 2) are both \x, so `\0 ** 2` proves nothing.
    assert_eq!(shape("\\0 ** 0"), "(** (u\\ 0) 0)");
}

#[test]
fn power_binds_tighter_than_multiplication() {
    // build/bin/rexx: `r = 2 * 3 ** 2` => 18, which is 2 * 9. Left to right
    // would give 36.
    assert_eq!(shape("2 * 3 ** 2"), "(* 2 (** 3 2))");
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    // build/bin/rexx: `r = 2 + 3 * 4` => 14, not 20.
    assert_eq!(shape("2 + 3 * 4"), "(+ 2 (* 3 4))");
}

#[test]
fn addition_binds_tighter_than_concatenation() {
    // build/bin/rexx with `a = 2; b = 3`: `r = a b + 1` => `2 4`. Taking the
    // concatenation first would give "23" + 1 = 24.
    assert_eq!(shape("a b + 1"), "(blank A (+ B 1))");
}

#[test]
fn concatenation_binds_tighter_than_comparison() {
    // build/bin/rexx: `r = 1 = 1 2` => 0, which is 1 = "1 2". Comparing first
    // would give (1 = 1) then " 2", printing `1 2`.
    assert_eq!(shape("1 = 1 2"), "(= 1 (blank 1 2))");
}

#[test]
fn comparison_binds_tighter_than_and() {
    // build/bin/rexx with `a = 2; b = 2`: `r = a = b & 1` => 1, which is
    // (2 = 2) & 1. Taking the `&` first would be 2 & (2 & 1), and `2 &` is
    // error 34.901, so the two are told apart by success against failure.
    assert_eq!(shape("a = b & 1"), "(& (= A B) 1)");
}

#[test]
fn and_binds_tighter_than_or() {
    // build/bin/rexx: `r = 1 | 0 & 0` => 1, which is 1 | (0 & 0). Left to
    // right would be (1 | 0) & 0 = 0.
    assert_eq!(shape("1 | 0 & 0"), "(| 1 (& 0 0))");
}

#[test]
fn or_and_xor_share_one_level_and_associate_left() {
    // build/bin/rexx: `r = 1 | 1 && 1` => 0, which is (1 | 1) && 1 = 1 && 1.
    // Right association would be 1 | (1 && 1) = 1 | 0 = 1.
    assert_eq!(shape("1 | 1 && 1"), "(&& (| 1 1) 1)");
}

// ---------------------------------------------------------------------------
// Associativity, per level.
// ---------------------------------------------------------------------------

#[test]
fn power_is_left_associative_unlike_most_languages() {
    // build/bin/rexx: `r = 2 ** 3 ** 2` => 64, which is (2 ** 3) ** 2. Right
    // association, which is what nearly every other language does, gives 512.
    assert_eq!(shape("2 ** 3 ** 2"), "(** (** 2 3) 2)");
}

#[test]
fn subtraction_is_left_associative() {
    // build/bin/rexx: `r = 10 - 3 - 2` => 5, not 9.
    assert_eq!(shape("10 - 3 - 2"), "(- (- 10 3) 2)");
}

#[test]
fn division_is_left_associative() {
    // build/bin/rexx: `r = 100 / 10 / 2` => 5, not 20.
    assert_eq!(shape("100 / 10 / 2"), "(/ (/ 100 10) 2)");
}

#[test]
fn operators_sharing_the_multiplicative_level_associate_left() {
    // build/bin/rexx: `r = 7 // 4 * 2` => 6, which is (7 // 4) * 2 = 3 * 2.
    // Right association would be 7 // (4 * 2) = 7.
    assert_eq!(shape("7 // 4 * 2"), "(* (// 7 4) 2)");
}

#[test]
fn comparison_is_left_associative() {
    // build/bin/rexx with `a = 2; b = 2; c = 1`: `r = a = b = c` => 1, which
    // is (a = b) = c, so 1 = 1. Right association gives 2 = (2 = 1) = 2 = 0,
    // which is 0. Equal operands cannot tell the two apart, which is why b is
    // 2 and c is 1.
    assert_eq!(shape("a = b = c"), "(= (= A B) C)");
    // Same with `==` and `>`: 1 and 0 respectively, against 0 and 1.
    assert_eq!(shape("a == b == c"), "(== (== A B) C)");
    assert_eq!(shape("a > b > c"), "(> (> A B) C)");
}

#[test]
fn concatenation_is_left_associative_though_no_value_can_show_it() {
    // Concatenation is associative, so the interpreter cannot discriminate:
    // `r = 1 2 3` is `1 2 3` under either grouping, and so is `1 || 2 3`. The
    // shape comes from the C++'s stack machine, which pops while
    // `token->precedence() <= second->precedence()`, and is asserted here
    // because a later change to the loop bound would otherwise go unnoticed.
    assert_eq!(shape("a b c"), "(blank (blank A B) C)");
    assert_eq!(shape("'a' || 'b' || 'c'"), "(|| (|| \"a\" \"b\") \"c\")");
}

// ---------------------------------------------------------------------------
// Prefix operators.
// ---------------------------------------------------------------------------

#[test]
fn a_prefix_operator_binds_looser_than_a_message_cascade() {
    // build/bin/rexx: `r = - "5"~length` => -1. Binding the prefix to the
    // literal first would be (-"5")~length, which is "-5"~length = 2.
    assert_eq!(shape("- \"5\"~length"), "(u- (msg~ \"5\" \"LENGTH\"))");
}

#[test]
fn prefix_operators_chain() {
    // build/bin/rexx: `r = - -2` => 2 and `r = \\0` => 0. The blank in `- -2`
    // is needed because `--` starts a line comment.
    assert_eq!(shape("- -2"), "(u- (u- 2))");
    assert_eq!(shape("\\\\0"), "(u\\ (u\\ 0))");
    assert_eq!(shape("+ - 2"), "(u+ (u- 2))");
}

#[test]
fn prefix_not_applies_before_a_comparison() {
    // build/bin/rexx with `a = 2`: `r = \a = 2` fails with error 34.901,
    // `Logical value must be exactly "0" or "1"; found "2"`, so the `\` was
    // applied to a and not to the comparison's result. Parsing it as
    // \(a = 2) would have succeeded with 0.
    assert_eq!(shape("\\a = 2"), "(= (u\\ A) 2)");
}

// ---------------------------------------------------------------------------
// The blank operator, abuttal, and function calls.
// ---------------------------------------------------------------------------

#[test]
fn a_blank_before_a_parenthesis_makes_a_concatenation_not_a_call() {
    // build/bin/rexx: `r = abs ('2.5')` => `ABS 2.5`, with a blank, because
    // `abs` is an uninitialised variable whose value is its own name and the
    // blank is the operator. `r = abs('2.5')` => 2.5.
    assert_eq!(shape("abs ('2.5')"), "(blank ABS \"2.5\")");
    assert_eq!(shape("abs('2.5')"), "(call ABS \"2.5\")");
}

#[test]
fn adjacent_terms_abut_and_separated_ones_concatenate_with_a_blank() {
    // build/bin/rexx with `a = 2; b = 2`: `r = (a)(b)` => `22` and
    // `r = (a) (b)` => `2 2`.
    assert_eq!(shape("(a)(b)"), "(abut A B)");
    assert_eq!(shape("(a) (b)"), "(blank A B)");
}

#[test]
fn a_keyword_spelling_is_an_ordinary_variable_in_an_expression() {
    // build/bin/rexx with `a = 2; b = 2`: `r = a b if` => `2 2 IF`, so `if`
    // is an uninitialised variable here. Keywords are not reserved words.
    assert_eq!(shape("a b if"), "(blank (blank A B) IF)");
}

// ---------------------------------------------------------------------------
// Leaf forms.
// ---------------------------------------------------------------------------

#[test]
fn a_numeric_symbol_keeps_its_upcased_spelling_as_its_value() {
    // build/bin/rexx: `r = 1e5` and `r = 1E5` both print `1E5`, not `100000`
    // and not `1e5`. So the value is the upcased spelling, which is exactly
    // what the interned name holds.
    assert_eq!(shape("1e5"), "1E5");
    // And a written decimal keeps its trailing zeros: `r = 1.50` prints 1.50.
    assert_eq!(shape("1.50"), "1.50");
}

#[test]
fn a_stem_keeps_its_trailing_period_and_a_compound_splits_into_tails() {
    // build/bin/rexx with `b = 2; c = 1`: `r = a.` => `A.`, `r = a.b.c` =>
    // `A.2.1`, `r = a.1.b` => `A.1.2` and `r = a..b` => `A..2`. So B and C
    // are looked up as variables while `1` and the empty piece stand for
    // themselves.
    assert_eq!(shape("a."), "stem:A.");
    assert_eq!(shape("a.b.c"), "compound:A.[var:B,var:C]");
    assert_eq!(shape("a.1.b"), "compound:A.[const:1,var:B]");
    assert_eq!(shape("a..b"), "compound:A.[const:,var:B]");
}

#[test]
fn a_leading_period_makes_an_environment_symbol() {
    // build/bin/rexx: `r = .true` => 1.
    assert_eq!(shape(".true"), "env:.TRUE");
}

#[test]
fn a_call_name_from_a_literal_is_used_exactly_as_written() {
    // build/bin/rexx: `r = 'abs'(-3)` fails with `Error 43.1: Could not find
    // routine "abs"`, while `r = 'ABS'(-3)` => 3. So a literal call name is
    // not upcased and does not reach the builtin table unless it is already
    // upper case, where a symbol name was upcased by the scanner.
    assert_eq!(shape("'abs'(-3)"), "(call \"abs\" (u- 3))");
    assert_eq!(shape("'ABS'(-3)"), "(call \"ABS\" (u- 3))");
    assert_eq!(shape("abs(-3)"), "(call ABS (u- 3))");
}

// ---------------------------------------------------------------------------
// Message sends.
// ---------------------------------------------------------------------------

#[test]
fn a_bracket_reference_is_the_bracket_message() {
    // build/bin/rexx: `r = "abc"[2]` and `r = "abc"~"[]"(2)` both => `b`, so
    // the two spellings are one operation and get one node.
    assert_eq!(shape("\"abc\"[2]"), "(msg~ \"abc\" \"[]\" 2)");
    assert_eq!(shape("\"abc\"~\"[]\"(2)"), "(msg~ \"abc\" \"[]\" 2)");
}

#[test]
fn a_message_name_is_upcased_whether_it_came_from_a_symbol_or_a_literal() {
    // build/bin/rexx: `r = "abc"~'length'`, `r = "abc"~'LENGTH'` and
    // `r = "abc"~"lEnGtH"` all => 3.
    assert_eq!(shape("\"abc\"~'length'"), "(msg~ \"abc\" \"LENGTH\")");
    assert_eq!(shape("\"abc\"~\"lEnGtH\""), "(msg~ \"abc\" \"LENGTH\")");
    assert_eq!(shape("\"abc\"~length"), "(msg~ \"abc\" \"LENGTH\")");
    // A blank on either side of the twiddle changes nothing, because the
    // scanner emits no blank token next to a `~`: `r = "abc" ~ length` => 3.
    assert_eq!(shape("\"abc\" ~ length"), "(msg~ \"abc\" \"LENGTH\")");
}

#[test]
fn a_cascade_is_one_term_and_reads_left_to_right() {
    // build/bin/rexxc accepts `r = a~~b~c`. The shape is what puts `~~b`
    // inside `~c` rather than the other way round.
    assert_eq!(shape("a~~b~c"), "(msg~ (msg~~ A \"B\") \"C\")");
    // build/bin/rexx: `r = .array~of(1,2)~~append(9)~items` => 3.
    assert_eq!(
        shape(".array~of(1,2)~~append(9)~items"),
        "(msg~ (msg~~ (msg~ env:.ARRAY \"OF\" 1 2) \"APPEND\" 9) \"ITEMS\")"
    );
}

#[test]
fn a_colon_after_a_message_name_is_a_superclass_override() {
    // The gate is `isVariableOrDot` (`Token.hpp:576`), which is
    // `VARIABLE | STEM | COMPOUND | DOTSYMBOL`, and it is wider than a class
    // name has any use for. build/bin/rexxc translates all five of these:
    //
    //   r = a~b:.nil       rc=0
    //   r = a~b:c          rc=0
    //   r = a~b:c.         rc=0     (and then fails at run time, 88.914)
    //   r = a~b:c.d        rc=0
    //   r = a~b:c.d.e      rc=0
    assert_eq!(shape("a~b:.nil"), "(msg~ A \"B\" :env:.NIL)");
    assert_eq!(shape("a~b:c(1)"), "(msg~ A \"B\" :C 1)");
    assert_eq!(shape("a~b:c."), "(msg~ A \"B\" :stem:C.)");
    assert_eq!(shape("a~b:c.d"), "(msg~ A \"B\" :compound:C.[var:D])");
    assert_eq!(
        shape("a~b:c.d.e"),
        "(msg~ A \"B\" :compound:C.[var:D,var:E])"
    );

    // And it must not be wider than those four classes. build/bin/rexxc:
    //
    //   r = a~b:1          Error 20.917
    //   r = a~b:1e5        Error 20.917
    //   r = a~b:.          Error 20.917    (a lone period is SYMBOL_DUMMY)
    assert_eq!(error("a~b:1"), (20, 917));
    assert_eq!(error("a~b:1e5"), (20, 917));
    assert_eq!(error("a~b:."), (20, 917));
    // A literal cannot pass either, because `isVariableOrDot` reads only the
    // symbol subclass and a literal token has none of the four.
    assert_eq!(error("a~b:'c'"), (20, 917));
}

#[test]
fn a_cascade_can_follow_a_variable_reference() {
    // This is the one route to `binary_rest`'s message-operator arm:
    // `variable_reference_term` returns straight to its caller rather than
    // through `message_subterm`'s cascade loop, so the `~` arrives with the
    // reference already on the left. build/bin/rexxc translates `r = >a~b`,
    // `r = >a[1]`, `r = >a~b~c` and `r = >a.~b`, all rc=0.
    assert_eq!(shape(">a~b"), "(msg~ (vref A) \"B\")");
    assert_eq!(shape(">a[1]"), "(msg~ (vref A) \"[]\" 1)");
    assert_eq!(shape(">a~b~c"), "(msg~ (msg~ (vref A) \"B\") \"C\")");
    assert_eq!(shape(">a.~b"), "(msg~ (vref stem:A.) \"B\")");
}

#[test]
fn only_a_parenthesis_abutted_to_a_message_name_is_an_argument_list() {
    // The blank before `(` is a token, so `a~m (1)` concatenates. Compare
    // `a~m(1)`, which passes 1.
    assert_eq!(shape("a~m (1)"), "(blank (msg~ A \"M\") 1)");
    assert_eq!(shape("a~m(1)"), "(msg~ A \"M\" 1)");
}

// ---------------------------------------------------------------------------
// Argument lists and parenthesised lists.
// ---------------------------------------------------------------------------

#[test]
fn trailing_omitted_arguments_are_dropped_but_list_elements_are_kept() {
    // build/bin/rexx with `::routine t; return arg()`: `t()` and `t(,)` and
    // `t(,,)` all report 0 arguments, `t(1,)` and `t(1,,)` report 1, `t(,1)`
    // reports 2 and `t(1,2)` reports 2.
    assert_eq!(shape("t()"), "(call T)");
    assert_eq!(shape("t(,)"), "(call T)");
    assert_eq!(shape("t(1,)"), "(call T 1)");
    assert_eq!(shape("t(,1)"), "(call T <omitted> 1)");
    assert_eq!(shape("t(1,2)"), "(call T 1 2)");

    // A parenthesised list keeps them: `(1,)~size` => 2, `(1,,)~size` => 3,
    // `(,)~size` => 2 and `(,1)~size` => 2.
    assert_eq!(shape("(1,)"), "(list 1 <omitted>)");
    assert_eq!(shape("(1,,)"), "(list 1 <omitted> <omitted>)");
    assert_eq!(shape("(,)"), "(list <omitted> <omitted>)");
    assert_eq!(shape("(,1)"), "(list <omitted> 1)");
}

#[test]
fn a_single_parenthesised_expression_is_not_a_list() {
    // build/bin/rexx: `r = (1)~class` reports `The String class`, while
    // `r = (1,2)~class` reports `The Array class`. So one element makes no
    // list node at all.
    assert_eq!(shape("(1)"), "1");
    assert_eq!(shape("((a))"), "A");
    assert_eq!(shape("(1,2)"), "(list 1 2)");
}

#[test]
fn calls_nest() {
    // build/bin/rexx: `r = f(g(h(2)))` fails with `Could not find routine
    // "H"`, so the innermost call was the first attempted.
    assert_eq!(shape("f(g(h(2)))"), "(call F (call G (call H 2)))");
    assert_eq!(shape("a[1,2]"), "(msg~ A \"[]\" 1 2)");
    assert_eq!(shape("a[]"), "(msg~ A \"[]\")");
}

// ---------------------------------------------------------------------------
// Variable references and qualified symbols.
// ---------------------------------------------------------------------------

#[test]
fn a_prefix_angle_bracket_takes_a_variable_reference() {
    // build/bin/rexxc: `r = >a`, `r = <a` and `r = >a.` all parse, while
    // `r = >a.b`, `r = >1` and `r = >"x"` are all error 20.930. `>` and `<` build the
    // same node, so nothing downstream distinguishes them.
    assert_eq!(shape(">a"), "(vref A)");
    assert_eq!(shape("<a"), "(vref A)");
    assert_eq!(shape(">a."), "(vref stem:A.)");
    assert_eq!(error(">a.b"), (20, 930));
    assert_eq!(error(">1"), (20, 930));
    assert_eq!(error(">\"x\""), (20, 930));
}

#[test]
fn a_symbol_before_a_colon_is_a_namespace_qualifier() {
    // build/bin/rexxc accepts all four. Wrapped in parentheses so that
    // `split_clauses` does not read the leading `foo:` as a label. The
    // parentheses build no node.
    assert_eq!(shape("(foo:bar)"), "(class FOO:BAR)");
    assert_eq!(shape("(foo:bar(1))"), "(qcall FOO:BAR 1)");
    // The qualified name may be any symbol class: `r = foo:1` and `r = 1:foo`
    // both parse.
    assert_eq!(shape("(foo:1)"), "(class FOO:1)");
    assert_eq!(shape("(1:foo)"), "(class 1:FOO)");
}

// ---------------------------------------------------------------------------
// Terminators.
// ---------------------------------------------------------------------------

#[test]
fn a_control_keyword_ends_a_control_expression_but_not_a_plain_one() {
    // build/bin/rexx: `r = 1 to 3` => `1 TO 3`, because with no terminator
    // set `to` is an ordinary uninitialised variable and both blanks are
    // operators. Under a `DO` control expression the same tokens stop at
    // `TO`, which is what keeps `do i = 1 to 3` from concatenating.
    assert_eq!(shape("1 to 3"), "(blank (blank 1 TO) 3)");
    assert_eq!(shape_until("1 to 3", Terminators::CONTROL), "1");
    assert_eq!(shape_until("1 while x", Terminators::COND), "1");
    assert_eq!(shape_until("1 until x", Terminators::COND), "1");
    assert_eq!(shape_until("1 then x", Terminators::IF), "1");
    // `OVER` has no case in `isTerminator`, so it never terminates however
    // the flags are set, and the two blanks then associate left.
    assert_eq!(
        shape_until("1 over x", Terminators::OVER),
        "(blank (blank 1 OVER) X)"
    );
}

#[test]
fn a_keyword_terminator_only_counts_as_a_simple_variable() {
    // `isTerminator` gates the keyword check on `isSimpleVariable`, so a
    // stem or a compound spelled like a keyword does not terminate.
    assert_eq!(
        shape_until("1 to. 3", Terminators::CONTROL),
        "(blank (blank 1 stem:TO.) 3)"
    );
}

#[test]
fn the_keyword_gate_is_what_admits_a_keyword_terminator() {
    // No terminator set the interpreter builds omits `TERM_KEYWORD` while
    // setting a keyword flag, so this constructs one that it never would. The
    // gate is otherwise unobservable: removing it from `is_terminator` fails
    // no other test here, and this records that rather than leaving it as a
    // silent hole.
    let gated = Terminators::EOC.with(Terminators::TO);
    let with_gate = gated.with(Terminators::KEYWORD);
    assert_eq!(shape_until("1 to 3", gated), "(blank (blank 1 TO) 3)");
    assert_eq!(shape_until("1 to 3", with_gate), "1");
}

#[test]
fn a_terminator_set_is_dropped_inside_parentheses() {
    // `parseSubTerm` passes only `TERM_RIGHT` into a parenthesised
    // subexpression, so a `DO` keyword inside parentheses is an ordinary
    // variable again.
    assert_eq!(
        shape_until("(1 to 3)", Terminators::CONTROL),
        "(blank (blank 1 TO) 3)"
    );
}

// ---------------------------------------------------------------------------
// Logical expression lists.
// ---------------------------------------------------------------------------

#[test]
fn a_comma_in_a_conditional_builds_a_logical_and() {
    // build/bin/rexxc accepts `if 1 = 1, 2 = 2 then nop`.
    assert_eq!(
        shape_logical("1 = 1, 2 = 2 then nop", Terminators::IF),
        "(logical (= 1 1) (= 2 2))"
    );
    // One element makes no node of its own.
    assert_eq!(shape_logical("1 = 1 then nop", Terminators::IF), "(= 1 1)");
}

#[test]
fn every_element_of_a_logical_list_is_required_including_the_first() {
    // build/bin/rexxc: `if then nop`, `if , 1 = 1 then nop` and
    // `if 1 = 1, then nop` are all error 35.929. So `parseLogical` never
    // returns an absent expression and the instruction gets no say.
    assert_eq!(logical_error("then nop", Terminators::IF), (35, 929));
    assert_eq!(
        logical_error(", 1 = 1 then nop", Terminators::IF),
        (35, 929)
    );
    assert_eq!(logical_error("1 = 1, then nop", Terminators::IF), (35, 929));
}

// ---------------------------------------------------------------------------
// Errors, against the corpus `d10-decision.md` recorded from `build/bin/rexxc`.
// ---------------------------------------------------------------------------

#[test]
fn the_error_corpus_reports_the_interpreters_numbers() {
    // Every case was run through `build/bin/rexxc` as `r = <expr>`, and the
    // number and sub-number are what this phase gates. The message text and
    // its substitutions are deliberately not reproduced.
    let cases: &[(&str, (u16, u16))] = &[
        (")", (37, 2)),
        ("]", (37, 901)),
        ("a[1]]", (37, 901)),
        ("1 2 3)", (37, 2)),
        ("a b )", (37, 2)),
        ("(a))", (37, 2)),
        ("a +", (35, 1)),
        ("a ||", (35, 1)),
        ("a || || b", (35, 1)),
        ("a + * b", (35, 1)),
        ("**2", (35, 1)),
        ("~a", (35, 1)),
        ("a %% b", (35, 1)),
        ("a \\(1 = 2)", (35, 1)),
        ("()", (35, 1)),
        ("\\", (35, 901)),
        ("(a", (36, 901)),
        ("((a)", (36, 901)),
        ("f(a b", (36, 901)),
        ("a[", (36, 902)),
        ("a[1", (36, 902)),
        ("(a[1", (36, 902)),
        ("a~", (19, 909)),
        ("a~~", (19, 909)),
        ("a~b~", (19, 909)),
        // `~[` is not a message name, and neither is a bracket after `~~`.
        (".array~of(1,2)~[1]", (19, 909)),
        // A closer of the wrong kind is a stray, not an unmatched opener:
        // `r = a[1)` is 37.2 and `r = a(1]` is 37.901.
        ("a[1)", (37, 2)),
        ("a(1]", (37, 901)),
        // A blank before `[` makes it a stray bracket rather than an index.
        ("a [1]", (35, 1)),
        ("a[1] [2]", (35, 1)),
        // An assignment shortcut is not an operator.
        ("a += 1", (35, 1)),
    ];
    let mut wrong = Vec::new();
    for (text, expected) in cases {
        let got = error(text);
        if got != *expected {
            wrong.push(format!("{text:?}: expected {expected:?}, got {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn three_inputs_that_look_like_errors_and_are_not() {
    // `d10-decision.md` records these as the ones a parser will get wrong if
    // it guesses. `build/bin/rexxc` accepts all three.
    assert_eq!(shape("a."), "stem:A.");
    assert_eq!(shape("f(,)"), "(call F)");
    assert_eq!(shape("a b if"), "(blank (blank A B) IF)");
}

#[test]
fn an_empty_required_expression_raises_the_sub_number_the_caller_supplied() {
    // The grammar never invents a number here, because the interpreter's
    // depends on which instruction wanted the expression: measured, `r =` is
    // 35.918 and `interpret` alone is 35.912. So the caller passes it.
    //
    // No source text reaches this, which is the point: `scan` never produces
    // an empty clause, so an empty expression only arises once an instruction
    // parser has consumed the clause's keywords, as `say` alone does. The
    // cursor is therefore built empty rather than derived from a clause.
    let source = ProgramSource::new(b"nop".to_vec(), SourceKind::Program);
    let scanned = scan(&source).expect("scans");
    let ctx = ParseCtx {
        source: &source,
        tokens: &scanned.tokens,
        symbols: &scanned.symbols,
        keywords: &scanned.keywords,
        resources: &scanned.resources,
    };
    for sub in [918, 912] {
        let mut cursor = TokenCursor::new(1..1);
        let err = parse_expr(&ctx, &mut cursor, Terminators::EOC, sub)
            .expect_err("an empty expression is an error");
        assert_eq!((err.code, err.sub), (35, sub));
    }
}

// ---------------------------------------------------------------------------
// Spans.
// ---------------------------------------------------------------------------

#[test]
fn a_nodes_span_covers_its_own_tokens_and_not_the_parentheses_around_them() {
    let text = "(a) + b";
    let (expr, _) = parse(text, Entry::Required).expect("parses");
    // A node's span runs from its leftmost token to its rightmost, and the
    // `+` node's leftmost token is `a` rather than `(`, because
    // `subterm` consumes the parentheses and returns the inner node
    // unchanged. So the span starts at 1.
    assert_eq!(expr.span, 1..7);
    // Slicing that span therefore yields unbalanced text, which is expected
    // and not a defect: a span is the extent from one token's start to
    // another's end, not a self-contained substring, and the same is true of a
    // clause span that crosses a comma continuation. The property the gate
    // checks is containment, and it holds: 1..7 contains both 1..2 and 6..7.
    assert_eq!(&text[expr.span.clone()], "a) + b");
    let ExprKind::Binary { left, right, .. } = &expr.kind else {
        panic!("expected a binary node, got {:?}", expr.kind);
    };
    assert_eq!(left.span, 1..2);
    assert_eq!(right.span, 6..7);
}

#[test]
fn a_leaf_span_is_exactly_its_token() {
    let text = "abc.def";
    let (expr, _) = parse(text, Entry::Required).expect("parses");
    assert_eq!(expr.span, 0..7);
    assert_eq!(&text[expr.span.clone()], "abc.def");
}

#[test]
fn a_call_span_reaches_its_closing_parenthesis() {
    let text = "f(1, 2)";
    let (expr, _) = parse(text, Entry::Required).expect("parses");
    assert_eq!(expr.span, 0..7);
    // And a message span reaches past its own closing parenthesis too.
    let text = "a~m(1)";
    let (expr, _) = parse(text, Entry::Required).expect("parses");
    assert_eq!(expr.span, 0..6);
}

#[test]
fn every_node_in_a_dense_expression_contains_its_operands() {
    // `check_spans` runs on every `shape` call, so this adds the shapes that
    // no other test needs plus the cases that stress the widening.
    for text in [
        "a + b * c ** -d || e f g",
        ".array~of(1, 2)~~append(3)[1]",
        "(1, 2, 3)",
        "f(g(h(x)), , y)",
        "\\a = b & c | d",
        ">stem.",
        "a.b.c d.e",
    ] {
        let (expr, _) = parse(text, Entry::Required).unwrap_or_else(|e| panic!("{text:?}: {e:?}"));
        check_spans(&expr, text);
    }
}

// ---------------------------------------------------------------------------
// The tables this module indexes by position.
// ---------------------------------------------------------------------------

#[test]
fn sub_keyword_indices_still_name_the_right_spellings() {
    // `is_terminator` compares `KeywordSet::index_of`'s result against these
    // constants, so a reordering of the C++ table would silently make `TO`
    // mean something else. The spellings are what matters, and they are
    // checked here rather than trusted.
    let mut symbols = SymbolTable::default();
    let keywords = crate::token::Keywords::new(&mut symbols);
    for (index, spelling) in [
        (super::SUBKEY_BY, "BY"),
        (super::SUBKEY_FOR, "FOR"),
        (super::SUBKEY_THEN, "THEN"),
        (super::SUBKEY_TO, "TO"),
        (super::SUBKEY_UNTIL, "UNTIL"),
        (super::SUBKEY_WHILE, "WHILE"),
        (super::SUBKEY_WITH, "WITH"),
    ] {
        assert_eq!(
            keywords.sub_keywords.index_of(symbols.intern(spelling)),
            Some(index),
            "sub-keyword {spelling} is not at index {index}"
        );
    }
}

#[test]
fn the_precedence_table_matches_the_cpp_level_for_level() {
    use crate::token::Operator::*;
    // `RexxToken::precedence` (`Token.cpp:111`), read out level by level. The
    // shape tests above check that these numbers are wired up correctly. This
    // checks the numbers themselves against the source they came from.
    for (op, level) in [
        (Backslash, 8),
        (Power, 7),
        (Multiply, 6),
        (Divide, 6),
        (IntDiv, 6),
        (Remainder, 6),
        (Plus, 5),
        (Subtract, 5),
        (Abuttal, 4),
        (Concatenate, 4),
        (Blank, 4),
        (Equal, 3),
        (BackslashEqual, 3),
        (GreaterThan, 3),
        (BackslashGreaterThan, 3),
        (LessThan, 3),
        (BackslashLessThan, 3),
        (GreaterThanEqual, 3),
        (LessThanEqual, 3),
        (StrictEqual, 3),
        (StrictBackslashEqual, 3),
        (StrictGreaterThan, 3),
        (StrictBackslashGreaterThan, 3),
        (StrictLessThan, 3),
        (StrictBackslashLessThan, 3),
        (StrictGreaterThanEqual, 3),
        (StrictLessThanEqual, 3),
        (LessThanGreaterThan, 3),
        (GreaterThanLessThan, 3),
        (And, 2),
        (Or, 1),
        (Xor, 1),
    ] {
        assert_eq!(
            super::precedence(op),
            level,
            "{} is not at level {level}",
            op.spelling()
        );
    }
}

#[test]
fn every_comparison_operator_parses_at_the_comparison_level() {
    // All 18 of them, each checked to bind looser than `||` and tighter than
    // `&`, which is what level 3 means. Spelling them out catches a missing
    // arm in `precedence`, which a handful of samples would not.
    for op in [
        "=", "\\=", ">", "\\>", "<", "\\<", ">=", "<=", "==", "\\==", ">>", "\\>>", "<<", "\\<<",
        ">>=", "<<=", "<>", "><",
    ] {
        let text = format!("a || b {op} c & d");
        assert_eq!(
            shape(&text),
            format!("(& ({op} (|| A B) C) D)"),
            "{op} did not parse at the comparison level"
        );
    }
}
