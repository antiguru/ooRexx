//! The differential check: 4,240 generated expressions, evaluated under
//! `build/bin/rexx` and again over the parsed tree.
//!
//! A parse tree cannot be compared to the interpreter directly, so this
//! compares *results*. Any divergence on a well-formed expression is a
//! precedence or associativity error, because nothing else in the pipeline
//! differs: the arithmetic itself is `rexx-num`, which Phase 2 already pinned
//! against the same interpreter.
//!
//! The corpus and its answers live in `rust/corpus/expr/precedence.tsv`, whose
//! header records how it was generated. Answers are baked in rather than
//! recomputed, which is how every other test in this crate treats the oracle:
//! `cargo test` must not need a built C++ interpreter.
//!
//! # What the evaluator is not
//!
//! Deliberately bounded to what settles precedence: numeric and string
//! literals, simple variables from a fixed table, the arithmetic and
//! comparison operators through `rexx-num`, concatenation explicit and
//! abuttal, the logical operators, prefix `+ - \`, and parentheses. A call, a
//! message send or a compound variable is `Unsupported` here and is asserted
//! on by shape in `tests.rs` instead. Growing this evaluator any further would
//! be starting Phase 4 inside a test.

use rexx_num::{CompareOp, DivOp, Number};

use crate::ast::{Expr, ExprKind, PrefixOp};
use crate::token::{Operator, SymbolTable};

use super::tests::{Entry, parse};

/// `NUMERIC DIGITS` at its default, which is what the corpus was taken under.
const DIGITS: u64 = 9;

/// The variable table the corpus was evaluated with. Any other name is an
/// uninitialised variable, whose value in Rexx is its own upcased name.
const VARS: &[(&str, &str)] = &[
    ("A", "2"),
    ("B", "3"),
    ("C", "1"),
    ("D", "4"),
    ("S", "xy"),
    ("T", "9"),
];

/// Why an evaluation produced no value.
#[derive(PartialEq, Eq, Debug)]
enum Failure {
    /// The interpreter would raise a condition here too: bad arithmetic, or a
    /// logical operand that is not exactly `0` or `1`.
    Raised,
    /// A form this evaluator does not implement, which the corpus must not
    /// contain.
    Unsupported,
}

fn value_of(text: &str) -> String {
    let name = text;
    VARS.iter()
        .find(|(var, _)| *var == name)
        .map_or_else(|| name.to_string(), |(_, value)| (*value).to_string())
}

fn number(text: &str) -> Result<Number, Failure> {
    Number::parse(text).ok_or(Failure::Raised)
}

/// A logical operand, which must be exactly `0` or `1`.
///
/// Measured: `' 1 ' & 1`, `'1.0' & 1`, `1.0 & 1` and `'01' & 1` are all error
/// 34.901, `Logical value must be exactly "0" or "1"`. So this is a byte
/// comparison and not a numeric one.
fn logical(text: &str) -> Result<bool, Failure> {
    match text {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(Failure::Raised),
    }
}

fn boolean(value: bool) -> String {
    if value { "1" } else { "0" }.to_string()
}

/// The `rexx-num` comparison for a Rexx comparison operator.
///
/// The negated spellings map onto the positive ones, which the interpreter
/// agrees with: measured, `2 \> 2` and `2 <= 2` are both 1, `3 \> 2` and
/// `3 <= 2` are both 0, and `'2 ' \>> '2'` and `'2 ' <<= '2'` are both 0.
fn compare_op(op: Operator) -> Option<CompareOp> {
    Some(match op {
        Operator::Equal => CompareOp::Equal,
        Operator::BackslashEqual
        | Operator::LessThanGreaterThan
        | Operator::GreaterThanLessThan => CompareOp::NotEqual,
        Operator::GreaterThan => CompareOp::Greater,
        Operator::LessThan => CompareOp::Less,
        Operator::GreaterThanEqual | Operator::BackslashLessThan => CompareOp::GreaterEqual,
        Operator::LessThanEqual | Operator::BackslashGreaterThan => CompareOp::LessEqual,
        Operator::StrictEqual => CompareOp::StrictEqual,
        Operator::StrictBackslashEqual => CompareOp::StrictNotEqual,
        Operator::StrictGreaterThan => CompareOp::StrictGreater,
        Operator::StrictLessThan => CompareOp::StrictLess,
        Operator::StrictGreaterThanEqual | Operator::StrictBackslashLessThan => {
            CompareOp::StrictGreaterEqual
        }
        Operator::StrictLessThanEqual | Operator::StrictBackslashGreaterThan => {
            CompareOp::StrictLessEqual
        }
        _ => return None,
    })
}

fn eval(expr: &Expr, symbols: &SymbolTable) -> Result<String, Failure> {
    match &expr.kind {
        ExprKind::Literal(bytes) => {
            String::from_utf8(bytes.to_vec()).map_err(|_| Failure::Unsupported)
        }
        // A constant's value is its own upcased spelling, which is why
        // `say 1e5` prints `1E5`.
        ExprKind::Constant(id) => Ok(symbols.name(*id).to_string()),
        ExprKind::Variable(id) => Ok(value_of(symbols.name(*id))),
        ExprKind::Prefix { op, operand } => {
            let value = eval(operand, symbols)?;
            match op {
                // Prefix `+` and `-` are arithmetic, so they normalise their
                // operand: measured, `+' 2.50 '` is 2.50 and `-'007'` is -7,
                // matching `x + 0` and `0 - x`.
                PrefixOp::Plus => number(&value)?
                    .add(&Number::zero(), DIGITS)
                    .map(|n| n.format(DIGITS))
                    .map_err(|_| Failure::Raised),
                PrefixOp::Minus => Number::zero()
                    .sub(&number(&value)?, DIGITS)
                    .map(|n| n.format(DIGITS))
                    .map_err(|_| Failure::Raised),
                PrefixOp::Not => Ok(boolean(!logical(&value)?)),
            }
        }
        ExprKind::Binary { op, left, right } => {
            let left = eval(left, symbols)?;
            let right = eval(right, symbols)?;
            binary(*op, &left, &right)
        }
        _ => Err(Failure::Unsupported),
    }
}

fn binary(op: Operator, left: &str, right: &str) -> Result<String, Failure> {
    // Concatenation and the logical operators first, because neither goes
    // through `rexx-num` and neither parses its operands as numbers.
    match op {
        Operator::Abuttal | Operator::Concatenate => return Ok(format!("{left}{right}")),
        Operator::Blank => return Ok(format!("{left} {right}")),
        // Both operands are validated whatever the first one is, because the
        // interpreter does not short-circuit: measured, `1 | 0 || 0` raises
        // 34.901 on `"00"` even though the left operand is already 1, and
        // `2 = 3 & 4` raises on `4` even though the left operand is 0.
        //
        // Writing this as `logical(left)? && logical(right)?` passes the corpus
        // on 4,206 of 4,240 cases and fails the other 34, which is what caught
        // it: Rust's `&&` and `||` do short-circuit.
        Operator::And | Operator::Or | Operator::Xor => {
            let (left, right) = (logical(left)?, logical(right)?);
            return Ok(boolean(match op {
                Operator::And => left && right,
                Operator::Or => left || right,
                _ => left != right,
            }));
        }
        Operator::Backslash => return Err(Failure::Unsupported),
        _ => {}
    }
    if let Some(comparison) = compare_op(op) {
        // `compare` takes the source strings, not parsed numbers, because a
        // strict comparison is a byte compare and a numeric one falls back to
        // a string compare when either side does not convert.
        return rexx_num::compare(left, right, DIGITS, 0, comparison)
            .map(boolean)
            .map_err(|_| Failure::Raised);
    }
    let a = number(left)?;
    let b = number(right)?;
    let result = match op {
        Operator::Plus => a.add(&b, DIGITS),
        Operator::Subtract => a.sub(&b, DIGITS),
        Operator::Multiply => a.mul(&b, DIGITS),
        Operator::Divide => a.div(&b, DIGITS, DivOp::Divide),
        Operator::IntDiv => a.div(&b, DIGITS, DivOp::IntegerDivide),
        Operator::Remainder => a.div(&b, DIGITS, DivOp::Remainder),
        Operator::Power => a.pow(&b, DIGITS),
        _ => return Err(Failure::Unsupported),
    };
    result
        .map(|n| n.format(DIGITS))
        .map_err(|_| Failure::Raised)
}

/// The corpus, with the answer the interpreter gave for each expression.
const CORPUS: &str = include_str!("../../../../corpus/expr/precedence.tsv");

/// The error numbers that mean the interpreter rejected the expression at
/// translation time rather than while running it.
///
/// The generator emits a handful of syntactically invalid expressions, and
/// they are worth keeping: `2 \3` is one, because no blank token is emitted
/// before a `\` and so the `\` lands in a dyadic position, which is 35.1.
///
/// This list is wider than the corpus header's, which names 35, 36, 37, 19 and
/// 20. Those five are the ones the generator actually produces today; 6, 13 and
/// 25 are unmatched comment, invalid character and invalid subkeyword, which are
/// translation errors too and would have to be classified the same way if a
/// regenerated corpus ever emitted one. The list is deliberately the property,
/// every translation-time number, rather than an inventory of what is currently
/// exercised, because an inventory silently reclassifies a row the day the
/// generator changes.
const SYNTAX_ERRORS: &[&str] = &["6", "13", "19", "20", "25", "35", "36", "37"];

#[test]
fn every_generated_expression_evaluates_to_what_the_interpreter_evaluated() {
    let mut checked_value = 0;
    let mut checked_raise = 0;
    let mut checked_reject = 0;
    let mut wrong: Vec<String> = Vec::new();

    for line in CORPUS.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let text = fields.next().expect("split yields at least one field");
        let verdict = fields
            .next()
            .unwrap_or_else(|| panic!("no verdict for {text:?}"));
        let expected = fields.next().unwrap_or("");
        let rejected = verdict == "ERR" && SYNTAX_ERRORS.contains(&expected);

        let parsed = parse(text, Entry::Required);
        match (rejected, parsed) {
            // The interpreter rejected it, so the grammar must too, and with
            // the same major number.
            (true, Err(err)) => {
                checked_reject += 1;
                if err.code.to_string() != expected {
                    wrong.push(format!(
                        "{text:?}: interpreter error {expected}, this parse {}",
                        err.code
                    ));
                }
            }
            (true, Ok((expr, symbols))) => wrong.push(format!(
                "{text:?}: interpreter rejected it with error {expected}, this \
                 parse accepted it as {}",
                expr.shape(&symbols)
            )),
            (false, Err(err)) => {
                wrong.push(format!("{text:?}: failed to parse: {err:?}"));
            }
            (false, Ok((expr, symbols))) => match (verdict, eval(&expr, &symbols)) {
                ("OK", Ok(got)) => {
                    checked_value += 1;
                    if got != expected {
                        wrong.push(format!(
                            "{text:?}: interpreter {expected:?}, this parse {got:?} \
                             (shape {})",
                            expr.shape(&symbols)
                        ));
                    }
                }
                ("OK", Err(failure)) => wrong.push(format!(
                    "{text:?}: interpreter {expected:?}, this parse failed with \
                     {failure:?} (shape {})",
                    expr.shape(&symbols)
                )),
                ("ERR", Err(Failure::Raised)) => checked_raise += 1,
                ("ERR", Err(Failure::Unsupported)) => wrong.push(format!(
                    "{text:?}: the evaluator does not implement this, so the \
                     corpus is out of its bounds (shape {})",
                    expr.shape(&symbols)
                )),
                ("ERR", Ok(got)) => wrong.push(format!(
                    "{text:?}: interpreter raised error {expected}, this parse \
                     gave {got:?} (shape {})",
                    expr.shape(&symbols)
                )),
                (other, _) => wrong.push(format!("{text:?}: unknown verdict {other:?}")),
            },
        }
    }

    assert!(
        wrong.is_empty(),
        "{} of {} cases diverged:\n{}",
        wrong.len(),
        checked_value + checked_raise + checked_reject + wrong.len(),
        wrong.join("\n")
    );
    // A corpus that silently shrank would make a green run meaningless, so the
    // counts are asserted rather than merely reported.
    assert_eq!(checked_value, 2686, "corpus lost or gained a valued case");
    assert_eq!(checked_raise, 1550, "corpus lost or gained a raising case");
    assert_eq!(checked_reject, 4, "corpus lost or gained a rejected case");
}

#[test]
fn the_evaluator_is_wired_to_the_operators_it_claims() {
    // A negative control for the test above. If `eval` returned
    // `Unsupported` for everything, or `binary` ignored its operator, the
    // corpus check could still pass on the 1,554 error cases alone. These
    // pin a value per operator family so that cannot happen quietly.
    let cases: &[(&str, &str)] = &[
        ("2 + 3", "5"),
        ("2 - 3", "-1"),
        ("2 * 3", "6"),
        ("3 / 2", "1.5"),
        ("7 % 2", "3"),
        ("7 // 2", "1"),
        ("2 ** 3", "8"),
        ("'x' || 'y'", "xy"),
        ("(1)(2)", "12"),
        ("1 2", "1 2"),
        ("2 = 2", "1"),
        ("2 == '2 '", "0"),
        ("1 & 0", "0"),
        ("1 | 0", "1"),
        ("1 && 1", "0"),
        ("\\0", "1"),
        ("-'007'", "-7"),
        ("+' 2.50 '", "2.50"),
        ("a b c", "2 3 1"),
        ("s || t", "xy9"),
    ];
    for (text, expected) in cases {
        let (expr, symbols) = parse(text, Entry::Required)
            .unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
        assert_eq!(
            eval(&expr, &symbols).as_deref(),
            Ok(*expected),
            "{text:?} evaluated wrongly"
        );
    }
}

#[test]
fn the_evaluator_reports_the_forms_it_does_not_implement() {
    // The bound has to be visible: a silent `Unsupported` in the corpus check
    // would look like agreement.
    for text in ["f(1)", "a~b", "a.b", "(1,2)", ">a", "a[1]", ".true"] {
        let (expr, symbols) = parse(text, Entry::Required)
            .unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
        assert_eq!(
            eval(&expr, &symbols),
            Err(Failure::Unsupported),
            "{text:?} should be out of the evaluator's scope"
        );
    }
}
