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

/// An operand that spells an integer reads as the same integer the tag
/// carries, and one that spells it any other way does not.
#[test]
fn an_operand_that_spells_an_integer_reads_as_that_integer() {
    for value in [0i64, 1, -1, 9, -9, 10, -10, 1234, -1234, 999_999] {
        let tagged = ObjRef::small_int(value).expect("inside the tagged range");
        assert_eq!(small_int_operand(tagged), Some(value), "tagged {value}");
        let spelled = value.to_string();
        let text = ObjRef::inline_text(spelled.as_bytes()).expect("at most seven bytes");
        assert_eq!(small_int_operand(text), Some(value), "spelled {spelled}");
    }
    for spelling in ["05", "+5", " 5", "5 ", "5.0", "-0", "1e2", "", "5x", "-"] {
        let text = ObjRef::inline_text(spelling.as_bytes()).expect("at most seven bytes");
        assert_eq!(small_int_operand(text), None, "{spelling:?}");
    }
}
use super::*;
use crate::plan::{BodyKey, ProgramId};
use crate::{Activation, error::Failure};
use rexx_parse::{InstructionKind, Program, parse_program};
use std::rc::Rc;

/// Pushes a fresh top-level activation for `program`, the same setup
/// `Interp::run` does, so a test can drive `eval` through a live
/// activation without running the instruction loop. `step`'s
/// `Assignment` arm has handled all three targets (`Variable`/`Stem`/
/// `Compound`) since Task 9, so this is no longer needed to work around
/// a gap in `run` -- it stays because this module's tests are about
/// `eval` itself, isolated from `step`'s surrounding dispatch, the same
/// reason `run.rs`'s own test module keeps its own copy of the same
/// helper rather than sharing one through `run`.
fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
    let program = Rc::new(program);
    let program_id = ProgramId(interp.programs.len());
    interp.programs.push(Rc::clone(&program));
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let id = interp.next_activation_id();
    interp.push_activation(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    program
}

/// Parses `source` (one `SAY` of an expression), activates it, and
/// evaluates that expression -- the one piece of machinery almost
/// every test below needs.
fn eval_source(interp: &mut Interp, source: &[u8]) -> Result<ObjRef, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = activate(interp, program);
    let expr = match &program.main.instructions[0].kind {
        InstructionKind::Say {
            expression: Some(expr),
        } => expr,
        other => panic!("expected a SAY with an expression, got {other:?}"),
    };
    let code = Code {
        body: &program.main,
        symbols: &program.symbols,
        slots: &[],
        plan: None,
    };
    interp.eval(&code, expr)
}

fn eval_text(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    let value = eval_source(interp, source)
        .unwrap_or_else(|failure| panic!("expected {source:?} to evaluate, got {failure:?}"));
    interp.to_text(value).to_vec()
}

/// `eval_source`, against the activation already on top of the stack
/// rather than pushing a fresh one.
fn eval_in_place(interp: &mut Interp, source: &[u8]) -> Result<ObjRef, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let expr = match &program.main.instructions[0].kind {
        InstructionKind::Say {
            expression: Some(expr),
        } => expr,
        other => panic!("expected a SAY with an expression, got {other:?}"),
    };
    let code = Code {
        body: &program.main,
        symbols: &program.symbols,
        slots: &[],
        plan: None,
    };
    interp.eval(&code, expr)
}

fn eval_in_place_text(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    let value = eval_in_place(interp, source)
        .unwrap_or_else(|failure| panic!("expected {source:?} to evaluate, got {failure:?}"));
    interp.to_text(value).to_vec()
}

// ---- the small-integer fast path ----

/// Every answer the fast path gives is the answer the general path
/// gives, over a grid of operands, precisions and both `FORM`s.
#[test]
fn the_small_int_fast_path_answers_what_the_general_path_answers() {
    use rexx_num::Form;

    let operands: [i64; 25] = [
        0,
        1,
        -1,
        2,
        -2,
        3,
        5,
        -5,
        7,
        -7,
        25,
        -25,
        99,
        100,
        999,
        -999,
        1000,
        -1000,
        1001,
        12345,
        -12345,
        1 << 30,
        -(1 << 30),
        rexx_core::SMALL_INT_MAX,
        rexx_core::SMALL_INT_MIN,
    ];
    let precisions: [u64; 9] = [1, 2, 3, 5, 9, 15, 18, 19, 20];
    let forms = [Form::Scientific, Form::Engineering];

    // Pinned to `is_arithmetic` in the direction that is a correctness
    // claim: nothing listed here is outside the set `eval_arithmetic`
    // dispatches. The other direction is not asserted and does not need
    // to be -- an arithmetic operator missing from this list is one
    // `small_int_arith`'s own `_` arm declines, which costs speed and
    // cannot cost an answer.
    let ops = [
        Operator::Plus,
        Operator::Subtract,
        Operator::Multiply,
        Operator::Divide,
        Operator::IntDiv,
        Operator::Remainder,
        Operator::Power,
    ];
    assert!(ops.iter().all(|op| is_arithmetic(*op)));

    let mut interp = Interp::new();
    // Per operator rather than one total: `+` alone reaches the fast path
    // thousands of times, so an aggregate count is satisfied by a guard
    // that admits nothing else. Each operator has to be seen going fast
    // on its own.
    let mut compared = [0usize; 7];
    for (index, op) in ops.into_iter().enumerate() {
        for left in operands {
            for right in operands {
                for digits in precisions {
                    let Some(fast) = small_int_arith(op, left, right, digits) else {
                        continue;
                    };
                    let fast_text = interp.to_text(fast).to_vec();

                    let left_number =
                        Number::parse(&left.to_string()).expect("an i64 spelling parses");
                    let right_number =
                        Number::parse(&right.to_string()).expect("an i64 spelling parses");
                    let result = match op {
                        Operator::Plus => left_number.add(&right_number, digits),
                        Operator::Subtract => left_number.sub(&right_number, digits),
                        Operator::Multiply => left_number.mul(&right_number, digits),
                        Operator::Divide => left_number.div(&right_number, digits, DivOp::Divide),
                        Operator::IntDiv => {
                            left_number.div(&right_number, digits, DivOp::IntegerDivide)
                        }
                        Operator::Remainder => {
                            left_number.div(&right_number, digits, DivOp::Remainder)
                        }
                        Operator::Power => left_number.pow(&right_number, digits),
                        other => unreachable!("{other:?} is not on the fast path"),
                    }
                    .expect("no arithmetic error on the general path either");

                    // Both `FORM`s, because the fast path never reads
                    // `FORM` at all: the claim being tested is that under
                    // its own guard the two forms cannot disagree, which
                    // only an assertion over both can carry.
                    for form in forms {
                        let general = interp.number(result.clone(), saturate_digits(digits), form);
                        let general_text = interp.to_text(general).to_vec();
                        assert_eq!(
                            String::from_utf8_lossy(&fast_text),
                            String::from_utf8_lossy(&general_text),
                            "{left} {op:?} {right} at DIGITS {digits}, FORM {form:?}"
                        );
                    }
                    compared[index] += 1;
                }
            }
        }
    }
    // The grid is mostly refusals at the low precisions, and `/` and `**`
    // decline most of what they are handed by construction, so the floor
    // is the one every operator clears rather than one scaled to the
    // widest.
    for (index, op) in ops.into_iter().enumerate() {
        assert!(
            compared[index] > 100,
            "{op:?} reached the fast path only {} times over the grid",
            compared[index]
        );
    }
}

/// `/` is the one arithmetic operator whose exact answer need not be an
/// integer, so the fast path takes it only when the division comes out
/// even -- and must take it then, or the guard is just a refusal.
#[test]
fn division_goes_fast_only_when_it_is_exact() {
    let mut interp = Interp::new();

    assert!(small_int_arith(Operator::Divide, 1, 3, 9).is_none());
    assert!(small_int_arith(Operator::Divide, 7, 2, 9).is_none());

    let fast = small_int_arith(Operator::Divide, 6, 2, 9).expect("6 / 2 is exact");
    assert_eq!(&*interp.to_text(fast), b"3");
    let fast = small_int_arith(Operator::Divide, -1000000, 1000, 9).expect("this is exact too");
    assert_eq!(&*interp.to_text(fast), b"-1000");
}

/// A zero divisor leaves the fast path for all three division operators,
/// so the 42.3 the general path raises is still what a program sees.
#[test]
fn a_zero_divisor_leaves_the_fast_path() {
    for op in [Operator::Divide, Operator::IntDiv, Operator::Remainder] {
        assert!(
            small_int_arith(op, 7, 0, 9).is_none(),
            "{op:?} by zero was answered on the fast path"
        );
    }
}

/// The sign rule for `%` and `//` with a negative operand, which is the
/// half of this candidate a wrong `i64` intuition would get wrong
/// silently.
#[test]
fn integer_division_truncates_toward_zero_and_the_remainder_follows_the_dividend() {
    let mut interp = Interp::new();
    let cases: [(Operator, i64, i64, &[u8]); 8] = [
        (Operator::IntDiv, -7, 3, b"-2"),
        (Operator::IntDiv, 7, -3, b"-2"),
        (Operator::IntDiv, -7, -3, b"2"),
        (Operator::IntDiv, 7, 3, b"2"),
        (Operator::Remainder, -7, 3, b"-1"),
        (Operator::Remainder, 7, -3, b"1"),
        (Operator::Remainder, -7, -3, b"-1"),
        (Operator::Remainder, 7, 3, b"1"),
    ];
    for (op, left, right, expected) in cases {
        let fast = small_int_arith(op, left, right, 9)
            .unwrap_or_else(|| panic!("{left} {op:?} {right} should go fast"));
        assert_eq!(
            &*interp.to_text(fast),
            expected,
            "{left} {op:?} {right} at DIGITS 9"
        );
    }
}

/// `**` takes a whole non-negative exponent whose exact power fits, and
/// nothing else -- a negative exponent leaves the integers, and a power
/// too wide for `DIGITS` would have to render exponentially.
#[test]
fn power_goes_fast_only_for_an_exact_non_negative_exponent() {
    let mut interp = Interp::new();

    assert!(small_int_arith(Operator::Power, 2, -1, 9).is_none());
    assert!(small_int_arith(Operator::Power, 2, 1_000_000_000, 9).is_none());
    // 2 ** 30 is 1073741824, ten digits, so under DIGITS 9 it rounds and
    // renders as 1.07374182E+9.
    assert!(small_int_arith(Operator::Power, 2, 30, 9).is_none());

    let fast = small_int_arith(Operator::Power, 2, 30, 10).expect("ten digits is enough");
    assert_eq!(&*interp.to_text(fast), b"1073741824");
    let fast = small_int_arith(Operator::Power, -3, 3, 9).expect("a negative base is fine");
    assert_eq!(&*interp.to_text(fast), b"-27");
    let fast = small_int_arith(Operator::Power, 0, 0, 9).expect("Rexx defines this as 1");
    assert_eq!(&*interp.to_text(fast), b"1");
}

/// The measured pair the guard exists for, and its neighbour that must
/// still go fast.
#[test]
fn an_operand_too_wide_for_the_precision_leaves_the_fast_path() {
    assert!(small_int_arith(Operator::Subtract, 1000, 25, 3).is_none());

    let mut interp = Interp::new();
    let fast = small_int_arith(Operator::Subtract, 100, 25, 3)
        .expect("both operands fit three digits, and so does the result");
    assert_eq!(&*interp.to_text(fast), b"75");
}

// ---- terms ----

#[test]
fn a_literal_is_its_own_bytes() {
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say 123"), b"123");
}

#[test]
fn a_constant_is_its_own_upcased_spelling() {
    // say 1e5 -> 1E5 (measured against the oracle; D15's own example).
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say 1e5"), b"1E5");
}

#[test]
fn a_variable_reads_through_the_plan() {
    let mut interp = Interp::new();
    let program = parse_program(b"say x".to_vec()).expect("test program parses");
    let program = activate(&mut interp, program);
    let five = interp.text(b"5");
    let slot = interp.slot_of(b"X");
    let frame = interp.activation().frame;
    interp.roots.set_frame_slot(frame, slot, five);
    let expr = match &program.main.instructions[0].kind {
        InstructionKind::Say {
            expression: Some(expr),
        } => expr,
        other => panic!("expected a SAY with an expression, got {other:?}"),
    };
    let code = Code {
        body: &program.main,
        symbols: &program.symbols,
        slots: &[],
        plan: None,
    };
    let value = interp.eval(&code, expr).unwrap();
    assert_eq!(&*interp.to_text(value), b"5");
}

#[test]
fn a_bare_stem_reads_through_the_same_path_as_a_variable() {
    // w. = 'wd' ; say w. -> wd
    let mut interp = Interp::new();
    activate(
        &mut interp,
        parse_program(b"nop".to_vec()).expect("test program parses"),
    );
    let wd = interp.text(b"wd");
    interp.stem_assign(b"W.", wd);
    assert_eq!(eval_in_place_text(&mut interp, b"say w."), b"wd");
}

#[test]
fn a_compound_read_resolves_its_tail_and_looks_it_up() {
    // a.1 = 'x' ; say a.1 -> x
    let mut interp = Interp::new();
    activate(
        &mut interp,
        parse_program(b"nop".to_vec()).expect("test program parses"),
    );
    let x = interp.text(b"x");
    interp.stem_set(b"A.", b"1", x);
    assert_eq!(eval_in_place_text(&mut interp, b"say a.1"), b"x");
}

/// The names the parser resolves, which reach no directory at all.
#[test]
fn the_three_parse_time_dot_variables() {
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say .nil"), b"The NIL object");
    assert_eq!(eval_text(&mut interp, b"say .true"), b"1");
    assert_eq!(eval_text(&mut interp, b"say .false"), b"0");
}

/// Every other name goes through the resolution order, and each of its
/// outcomes is reachable from an expression: an environment entry, a name
/// nothing here answers that the oracle does, and a name neither answers.
#[test]
fn a_dot_variable_beyond_the_three_resolves_falls_back_or_is_loud() {
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say .array"), b"The Array class");
    assert_eq!(eval_text(&mut interp, b"say .foo"), b".FOO");
    let failure = eval_source(&mut interp, b"say .stdout").unwrap_err();
    assert!(matches!(failure, Failure::Loud(_)), "got {failure:?}");
}

// ---- prefix ----

#[test]
fn prefix_plus_and_minus_are_arithmetic_not_a_sign_flip() {
    // numeric digits 1 ; say -12345 -> -1E+4 ; say +12345 -> 1E+4
    // (measured: the same rounding `0 - 12345`/`0 + 12345` gives).
    // `NUMERIC` is not run through `step` (Task 9's instruction), so
    // `DIGITS` is set directly on the activation the same way Task 9's
    // own implementation eventually will.
    let mut interp = Interp::new();
    activate(
        &mut interp,
        parse_program(b"nop".to_vec()).expect("test program parses"),
    );
    interp
        .activation_mut()
        .settings
        .set_digits_str("1")
        .unwrap();
    assert_eq!(eval_in_place_text(&mut interp, b"say -12345"), b"-1E+4");
    assert_eq!(eval_in_place_text(&mut interp, b"say +12345"), b"1E+4");
}

#[test]
fn prefix_not_flips_a_logical_value() {
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say \\1"), b"0");
    assert_eq!(eval_text(&mut interp, b"say \\0"), b"1");
}

#[test]
fn prefix_not_on_a_non_logical_value_raises_34_901() {
    // say \'abc' -> Error 34.901
    let mut interp = Interp::new();
    let failure = eval_source(&mut interp, b"say \\'abc'").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 901));
    assert_eq!(raised.additional, vec![b"abc".to_vec()]);
}

// ---- arithmetic ----

#[test]
fn the_seven_arithmetic_operators() {
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say 1+2"), b"3");
    assert_eq!(eval_text(&mut interp, b"say 5-3"), b"2");
    assert_eq!(eval_text(&mut interp, b"say 2*3"), b"6");
    assert_eq!(eval_text(&mut interp, b"say 7/2"), b"3.5");
    assert_eq!(eval_text(&mut interp, b"say 7%2"), b"3");
    assert_eq!(eval_text(&mut interp, b"say 7//2"), b"1");
    assert_eq!(eval_text(&mut interp, b"say 2**3"), b"8");
}

#[test]
fn divide_by_zero_raises_42_3() {
    // say 1/0 -> Error 42.3, rc 214
    let mut interp = Interp::new();
    let failure = eval_source(&mut interp, b"say 1/0").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
}

#[test]
fn remainder_by_zero_also_raises_42_3() {
    // say 1//0 -> Error 42.3, rc 214 (the same DivideByZero as /)
    let mut interp = Interp::new();
    let failure = eval_source(&mut interp, b"say 1//0").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
}

#[test]
fn a_nonnumeric_operand_raises_41_1_with_its_own_text() {
    // say 'abc' + 1 -> Error 41.1, "Nonnumeric value (\"abc\")"
    let mut interp = Interp::new();
    let failure = eval_source(&mut interp, b"say 'abc'+1").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (41, 1));
    assert_eq!(raised.additional, vec![b"abc".to_vec()]);
}

#[test]
fn a_non_numeric_power_exponent_raises_26_8_not_41_1() {
    // say 2 ** 'x' -> Error 26.8, "found \"x\""
    // 'y' ** 2     -> Error 41.1 (the base's ordinary nonnumeric path)
    let mut interp = Interp::new();
    let failure = eval_source(&mut interp, b"say 2**'x'").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (26, 8));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);

    let base_failure = eval_source(&mut interp, b"say 'y'**2").unwrap_err();
    let Failure::Raised(base_raised) = base_failure else {
        panic!("expected Raised, got {base_failure:?}");
    };
    assert_eq!((base_raised.number, base_raised.sub), (41, 1));
}

#[test]
fn a_number_created_by_arithmetic_renders_under_the_digits_that_made_it() {
    // numeric digits 1 ; x = 15 + 0 ; x is 20, so x + 6 is 3E+1 while
    // 15 + 6 is 2E+1 -- D15's own SmallInt-admissibility transcript,
    // reachable end to end through eval now rather than constructed by
    // hand as Task 4 had to.
    let mut interp = Interp::new();
    activate(
        &mut interp,
        parse_program(b"nop".to_vec()).expect("test program parses"),
    );
    interp
        .activation_mut()
        .settings
        .set_digits_str("1")
        .unwrap();
    assert_eq!(eval_in_place_text(&mut interp, b"say 15+0"), b"2E+1");
    assert_eq!(eval_in_place_text(&mut interp, b"say 15+6"), b"2E+1");
}

// ---- concatenation ----

#[test]
fn the_three_concatenation_forms() {
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say 'a'||'b'"), b"ab");
    assert_eq!(eval_text(&mut interp, b"say 'a' 'b'"), b"a b");

    // Abuttal: two adjacent terms with no operator and no whitespace.
    // Not `'a'('b')` -- a quoted literal directly followed by `(...)`
    // is call syntax (`CallTarget::Literal`), measured: `say
    // 'a'('b')` is Error 43.1, "Could not find routine \"a\"", not
    // concatenation. `x'b'` (a variable directly followed by a
    // literal) is the real Abuttal shape, matching the oracle
    // transcript this task's report already verified (`x = 'a'; say
    // x'b'` -> `ab`).
    let program = parse_program(b"say x'b'".to_vec()).expect("test program parses");
    let program = activate(&mut interp, program);
    let a = interp.text(b"a");
    let slot = interp.slot_of(b"X");
    let frame = interp.activation().frame;
    interp.roots.set_frame_slot(frame, slot, a);
    let expr = match &program.main.instructions[0].kind {
        InstructionKind::Say {
            expression: Some(expr),
        } => expr,
        other => panic!("expected a SAY with an expression, got {other:?}"),
    };
    let code = Code {
        body: &program.main,
        symbols: &program.symbols,
        slots: &[],
        plan: None,
    };
    let value = interp.eval(&code, expr).unwrap();
    assert_eq!(&*interp.to_text(value), b"ab");
}

// ---- comparison ----

#[test]
fn the_thirteen_line_comparison_transcript() {
    // The plan's own measured block, verbatim (task-8-report.md has
    // the oracle run). The first row is the one that discriminates the
    // real string rule from "blank-pad the shorter on the right".
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say (' a' = 'a')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('09'x'a' = 'a')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('a' = 'a'||'09'x)"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('a' = 'a ')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('a b' = 'a  b')"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('' = ' ')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('01' = '1')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say (' 1 ' = 1)"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('a' = 1)"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('10' >> '9')"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('10' > '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('a' << 'a ')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('01' == '1')"), b"0");
}

#[test]
fn backslash_negated_and_synonym_forms() {
    // Not in the plan's own transcript; measured separately (report)
    // to pin the Operator -> CompareOp mapping, particularly that
    // \>/\< invert the *positive* comparison's sense rather than
    // getting a CompareOp of their own.
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say ('9' \\> '10')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\< '10')"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\>> '10')"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\<< '10')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' <> '10')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' >< '10')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\= '10')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\== '9')"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('9' >>= '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('8' >>= '9')"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('10' <<= '9')"), b"1");

    // **Equal operands, and every row above is unequal ones.** Seven of
    // the eighteen mapping rows survived a mutation run against the whole
    // suite: `\>` to `Less`, `\<` to `Greater`, `\>>` to `StrictLess`,
    // `\<<` to `StrictGreater`, `<<=` to `StrictLess`, `>=` to `Greater`,
    // and `<=` to `Less`. Equality is the only case that separates
    // `LessEqual` from `Less`, so unequal operands cannot tell those pairs
    // apart however many of them a test lists, and non-strict `>=`/`<=`
    // appeared in no test at all.
    assert_eq!(eval_text(&mut interp, b"say ('9' \\> '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\< '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\>> '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' \\<< '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' <<= '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' <= '9')"), b"1");
    assert_eq!(eval_text(&mut interp, b"say ('9' >= '9')"), b"1");
}

#[test]
fn a_comparison_reuses_an_already_parsed_num_cache() {
    // Not a behaviour difference visible from the answer alone (the
    // numeric family gives the same result whether or not the cache
    // was already warm) -- what this actually exercises is that
    // compare_values, reading `x` back out through a variable, still
    // takes the numeric path on an object whose `num` cache this test
    // itself already filled, rather than `to_number` inside
    // compare_values somehow needing a cold object to work at all.
    // 007 and 7 comparing numerically equal (not "0" -- a byte compare
    // would disagree with the leading zero) is what proves the
    // numeric path, not the string one, ran.
    let mut interp = Interp::new();
    activate(
        &mut interp,
        parse_program(b"nop".to_vec()).expect("test program parses"),
    );
    let x = interp.text(b"007");
    interp.to_number(x).expect("007 parses, filling its cache");
    let slot = interp.slot_of(b"X");
    let frame = interp.activation().frame;
    interp.roots.set_frame_slot(frame, slot, x);
    assert_eq!(eval_in_place_text(&mut interp, b"say (x = '7')"), b"1");
}

#[test]
fn strict_equality_compares_bytes_where_the_ordinary_form_compares_value() {
    // The contrast is the test: the same two operands answer 0 under `==`
    // and 1 under `=`, so this pins that `==` reaches `CompareOp`'s strict
    // row rather than the ordinary one.
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say ('01' == '1')"), b"0");
    assert_eq!(eval_text(&mut interp, b"say ('01' = '1')"), b"1");
}

// ---- logical ----

#[test]
fn the_three_logical_operators_and_their_truth_tables() {
    let mut interp = Interp::new();
    assert_eq!(eval_text(&mut interp, b"say (1 & 1)"), b"1");
    assert_eq!(eval_text(&mut interp, b"say (1 & 0)"), b"0");
    assert_eq!(eval_text(&mut interp, b"say (0 | 0)"), b"0");
    assert_eq!(eval_text(&mut interp, b"say (1 && 1)"), b"0");
    assert_eq!(eval_text(&mut interp, b"say (1 && 0)"), b"1");
}

#[test]
fn logical_operators_raise_34_901_on_a_non_logical_operand() {
    // ' 1 ' & 1 -> Error 34.901, found " 1 " (and '01'/'1.0'/'' alike,
    // measured in the report -- one representative here, the same
    // check `logical_value` gives all four).
    let mut interp = Interp::new();
    let failure = eval_source(&mut interp, b"say (' 1 ' & 1)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 901));
    assert_eq!(raised.additional, vec![b" 1 ".to_vec()]);
}

#[test]
fn logical_operators_do_not_short_circuit() {
    // say (0 & 'x') -> still raises on 'x', even though 0 already
    // decides the AND -- both operands are always evaluated and
    // checked. Contrast with ExprKind::Logical's comma list, below.
    let mut interp = Interp::new();
    let and_failure = eval_source(&mut interp, b"say (0 & 'x')").unwrap_err();
    assert!(
        matches!(and_failure, Failure::Raised(_)),
        "got {and_failure:?}"
    );
    let or_failure = eval_source(&mut interp, b"say (1 | 'x')").unwrap_err();
    assert!(
        matches!(or_failure, Failure::Raised(_)),
        "got {or_failure:?}"
    );
}

#[test]
fn logical_operators_check_the_left_operand_first() {
    // say ('y' & 'x') -> reports "y", the left operand, when both are
    // bad -- confirms evaluation order rather than assuming it.
    let mut interp = Interp::new();
    let failure = eval_source(&mut interp, b"say ('y' & 'x')").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(raised.additional, vec![b"y".to_vec()]);
}

// ---- ExprKind::Logical (the comma list) ----

/// Parses `if <cond> then nop`, extracts the `IF`'s own `condition`
/// (an `ExprKind::Logical` for a comma list, an ordinary `Expr`
/// otherwise) and evaluates it directly, bypassing `run`. The only way
/// to build one: comma-list syntax is gated to `IF`/`WHEN`/`GUARD`/
/// `WHILE`/`UNTIL` in the parser (`ast.rs`'s own doc comment on
/// `ExprKind::Logical`). `IF`/`WHEN`/`WHILE`/`UNTIL` all run since
/// Tasks 10/11, so a real program can reach one through `run` too --
/// this helper stays because it isolates `eval`'s own handling of the
/// condition from `step`'s surrounding dispatch, not because `run`
/// cannot reach one at all.
fn eval_condition(interp: &mut Interp, source: &[u8]) -> Result<ObjRef, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    // Activated like `eval_source`'s own programs, where this used to
    // evaluate against a bare `Interp`: `eval`'s own intermediate-value
    // gate reads the *running activation's* `TRACE` since Task 3 moved
    // `trace_mode` off `Interp`, so there has to be one.
    let program = activate(interp, program);
    let code = Code {
        body: &program.main,
        symbols: &program.symbols,
        slots: &[],
        plan: None,
    };
    let condition = match &program.main.instructions[0].kind {
        InstructionKind::If { condition, .. } => condition,
        other => panic!("expected an IF, got {other:?}"),
    };
    interp.eval(&code, condition)
}

fn eval_condition_text(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    let value = eval_condition(interp, source)
        .unwrap_or_else(|failure| panic!("expected {source:?} to evaluate, got {failure:?}"));
    interp.to_text(value).to_vec()
}

#[test]
fn a_comma_list_is_an_and_of_its_parts() {
    // if 1, 1, 1 then -> true ; if 1, 0, 1 then -> false (measured
    // against the oracle's own THEN/ELSE branch taken).
    let mut interp = Interp::new();
    assert_eq!(
        eval_condition_text(&mut interp, b"if 1, 1, 1 then nop"),
        b"1"
    );
    assert_eq!(
        eval_condition_text(&mut interp, b"if 1, 0, 1 then nop"),
        b"0"
    );
}

#[test]
fn a_comma_list_element_failure_raises_34_6_not_34_901() {
    // if 1, 'x' then -> Error 34.6, found "x" (checked left to right:
    // if 'x', 1 then also gives 34.6 on "x", the first element).
    let mut interp = Interp::new();
    let failure = eval_condition(&mut interp, b"if 1, 'x' then nop").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 6));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);

    let first_bad = eval_condition(&mut interp, b"if 'x', 1 then nop").unwrap_err();
    let Failure::Raised(first_bad) = first_bad else {
        panic!("expected Raised, got {first_bad:?}");
    };
    assert_eq!(first_bad.additional, vec![b"x".to_vec()]);
}

#[test]
fn a_comma_list_short_circuits_on_the_first_false_element() {
    // The opposite of `&`'s own rule
    // (logical_operators_do_not_short_circuit, above).
    let mut interp = Interp::new();
    // An activation, which the `'x'` version of this test did not need:
    // division reads the frame's own `NUMERIC DIGITS`, so the control case
    // below reaches `activation()` where a literal never would.
    activate(
        &mut interp,
        parse_program(b"nop".to_vec()).expect("test program parses"),
    );
    assert_eq!(
        eval_condition_text(&mut interp, b"if 0, (1/0) then nop"),
        b"0",
        "the second element must never be evaluated, not merely left unchecked"
    );
    assert_eq!(
        eval_condition_text(&mut interp, b"if 1, 0, (1/0) then nop"),
        b"0",
        "short-circuits at the second element, never reaching the third"
    );
    let reached = eval_condition(&mut interp, b"if 1, (1/0) then nop");
    let Err(Failure::Raised(raised)) = reached else {
        panic!("an element that IS reached must raise, or the two cases above prove nothing");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
}

// ---- D19's evaluation-depth limit ----

/// A left-deep chain of `terms` `SAY`-able terms, joined by `||''` --
/// the concatenation analogue `records_the_stack_cost_of_one_eval_frame`
/// (`tests/spike.rs`) already establishes recurses exactly once per
/// term (`outcome.stack.max_depth == TERMS`, asserted there), reused
/// here rather than a fresh arithmetic chain invented for this test:
/// one already-measured relationship between term count and `eval`
/// depth is worth more than two unrelated ones.
fn chain(terms: usize) -> Vec<u8> {
    let mut program = b"say 'a'".to_vec();
    for _ in 1..terms {
        program.extend_from_slice(b"||''");
    }
    program.push(b'\n');
    program
}

/// The same chain with one **call** in it, which is what makes the whole
/// expression reach `eval` at all.
fn eval_walked_chain(terms: usize) -> Vec<u8> {
    let mut program = b"say 'a'".to_vec();
    for term in 1..terms {
        // `substr('x', 1, 0)` is the empty string, so the chain's value is
        // unchanged and only its *shape* differs from `chain`'s.
        if term == 5 {
            program.extend_from_slice(b"||substr('x',1,0)");
        } else {
            program.extend_from_slice(b"||''");
        }
    }
    program.push(b'\n');
    program
}

/// **A native chain past the limit runs, and that is the point of the
/// limit rather than a hole in it.**
#[test]
fn a_native_chain_past_the_eval_limit_runs() {
    let outcome = crate::run_program(
        "depth-native.rex",
        chain(MAX_EVAL_DEPTH + 1),
        crate::Invocation::none(),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"a\n");
    assert!(
        outcome.stack.max_depth < MAX_EVAL_DEPTH,
        "a native chain recursed {} levels into eval, so this test no longer \
         says that a compiled chain does not recurse",
        outcome.stack.max_depth
    );
}

/// **Why this goes through `run_program` and not a direct `eval` call.**
/// The only sized stack in the workspace is inside `run_program`
/// (`lib.rs`'s own `INTERPRETER_STACK_BYTES`); a `cargo test` thread's
/// default 2 MiB is far smaller than what this depth needs, and `eval`
/// would die natively, as an unreported guard-page abort, long before
/// reaching either boundary this pair tests -- precisely the silent
/// failure D19's limit exists to prevent, so a plain `#[cfg(test)]`
/// unit test calling `eval` directly cannot exercise this at all. This
/// is a unit test file, not `tests/`, but the subject it is testing
/// (`run_program`'s own observable behaviour at the limit) is public
/// cross-crate surface, so an integration-shaped test of it is not the
/// thing the crate's own testing rule forbids (that rule is about
/// integration-testing a *private* subject, and there is not one
/// here) -- it simply lives beside the counter it defends rather than
/// in a separate file, since nothing about reaching `run_program`
/// requires a different module.
#[test]
fn eval_survives_exactly_max_eval_depth_terms_and_prints_the_oracles_own_answer() {
    let outcome = crate::run_program(
        "depth-boundary-at.rex",
        eval_walked_chain(MAX_EVAL_DEPTH),
        crate::Invocation::none(),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout, b"a\n",
        "the oracle's own answer for this exact depth (Deviation 2, phase-4-exclusions.txt)"
    );
    assert_eq!(outcome.stack.max_depth, MAX_EVAL_DEPTH);
}

/// **The off-by-one this task's own brief calls out by name**: one term
/// past the boundary the previous test pins must raise, not merely
/// "eventually" refuse something deeper. Kills a `>=` in place of `eval`'s
/// own `>` check, which would refuse the boundary case above instead of
/// this one.
#[test]
fn eval_raises_11_1_exactly_one_term_past_max_eval_depth() {
    let outcome = crate::run_program(
        "depth-boundary-past.rex",
        eval_walked_chain(MAX_EVAL_DEPTH + 1),
        crate::Invocation::none(),
    );
    assert_eq!(
        outcome.exit_code,
        245,
        "256 - 11, stderr: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout, b"",
        "SAY never runs: the whole expression must finish evaluating first, \
         and this one cannot"
    );
}

// ---- ExprKind::Call (Task 4, 4b): the internal-function expression
// form ----

/// The task brief's own Step 1: an internal routine called from inside
/// an expression, not a `CALL` clause of its own, returns its value into
/// the enclosing arithmetic.
#[test]
fn an_internal_function_returns_its_value_into_an_expression() {
    let outcome = crate::run_program(
        "call-expr-basic.rex",
        b"say f(1) + 1\nexit\nf: return 41\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"42\n");
}

/// A name that resolves to nothing -- not a label of the calling body,
/// not a builtin and not a `::ROUTINE` -- raises the oracle's own 43.1
/// rather than succeeding, crashing, or reporting a gap.
#[test]
fn an_unresolvable_name_raises_43_1() {
    let outcome = crate::run_program(
        "call-expr-builtin.rex",
        b"say zorkolo('abc')\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 213);
    assert!(
        String::from_utf8_lossy(&outcome.stderr)
            .contains(r#"Error 43.1:  Could not find routine "ZORKOLO"."#),
        "stderr: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"");
}

/// The neighbouring case the one above cannot pin on its own: a builtin
/// Phase 4 excludes **outright** is not "resolves to nothing", and must
/// stay loud rather than joining it at 43.1. The oracle answers
/// `RXQUEUE`, so a condition here would let a program expecting one pass
/// against a gap.
#[test]
fn a_wholly_excluded_builtin_stays_loud_rather_than_raising_43_1() {
    let outcome = crate::run_program(
        "call-expr-excluded.rex",
        b"say rxqueue('G')\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, crate::NOT_IMPLEMENTED_EXIT);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        "rexx-exec: routine \"RXQUEUE\" is not implemented (Phase 10)\n",
        "the refusal names the phase the exclusion table gives it"
    );
}

/// **`CallTarget::Literal` never searches the label table**, symmetric
/// with `CALL "SUB"` (Task 3). Measured independently on the oracle in a
/// clean directory (never the scratchpad root, which sits on the
/// external-routine search path and holds a stale `f.rex` -- the first
/// attempt at exactly this measurement found it and reported the wrong
/// answer): `say "f"(1)` with `f:` present is Error 43.1 rc 213, "Routine
/// not found", where `say f(1)` runs the label instead. That is this
/// crate's answer too now that the builtin and `::ROUTINE` steps behind
/// the label search exist: "not a label" and "not anything at all" are
/// separable, so the condition is the oracle's own rather than fabricated.
#[test]
fn a_literal_call_target_never_reaches_the_label_table() {
    let outcome = crate::run_program(
        "call-expr-literal.rex",
        b"say \"f\"(1)\nexit\nf: return 41\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 213);
    assert!(
        String::from_utf8_lossy(&outcome.stderr)
            .contains(r#"Error 43.1:  Could not find routine "f"."#),
        "stderr: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"", "the label must not have run");
}

/// **Step 3's own measurement**: a routine reached through the
/// expression form and returning nothing -- a bare `RETURN`, not the
/// same event as falling off the routine's own end without one (the next
/// test) -- raises Error 44.1 at rc 212. Measured on the oracle in a
/// clean directory: `say f(1)` into `f: return` gives exactly this,
/// "No data returned from function "F"." with the label's own upcased
/// spelling.
#[test]
fn a_routine_returning_no_value_in_expression_form_raises_44_1() {
    let outcome = crate::run_program(
        "call-expr-no-data.rex",
        b"say f(1)\nexit\nf: return\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 212, "256 - 44");
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert!(
        stderr.contains("Function or message did not return data.")
            && stderr.contains("No data returned from function \"F\"."),
        "stderr: {stderr:?}"
    );
    assert_eq!(
        outcome.stdout, b"",
        "SAY never runs: the argument raises before the clause completes"
    );
}

/// **Measured on the oracle**, in a clean directory: `EXIT` inside a
/// routine reached through the expression form ends the whole program
/// exactly as it does through `CALL`, at rc 5, with no stdout (`SAY`
/// never completes) and no stderr (an `EXIT` is not a condition, so
/// nothing is reported). This exercises `Failure::Exited`'s whole reason
/// for existing (`error.rs`): `eval`'s own return type has no `Flow` to
/// carry the event through the way `CALL`'s instruction form does, so it
/// travels as this `Failure` variant instead, unwound by every
/// enclosing `?` with no special handling needed, until `execute`
/// (`lib.rs`) reads it back as an ordinary successful exit.
#[test]
fn an_exit_inside_a_routine_reached_by_expression_call_ends_the_whole_program() {
    let outcome = crate::run_program(
        "call-expr-exit.rex",
        b"say f(1)\nexit 9\nf: exit 5\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 5, "stderr: {:?}", outcome.stderr);
    assert_eq!(outcome.stdout, b"");
    assert_eq!(outcome.stderr, b"");
}

/// **Review finding M4, Task 4 fix round 1.** `Failure::Exited` has two
/// producers -- an `EXIT` instruction (the test above) and a routine
/// falling off its own end (`run_activation`'s own `Ok(Ended::Exited(
/// None))` when its instruction loop runs out) -- and only the first had
/// a test. The two are genuinely distinct events reaching the same
/// `Ended::Exited` arm in `eval_call`, and the risk this test closes is
/// specific: a version that only checked for an explicit `Exit` value
/// while treating "no more instructions" as `Ended::Returned(None)`
/// would misroute this case into `Raised::no_data_returned` (Error
/// 44.1) instead of silently ending the program -- a real, measured
/// divergence (`error.rs`'s `no_data_returned` doc has the "**Not** the
/// same path" note this pins). Measured on the oracle in a clean
/// directory: `say f(1)` into `f: nop` at the very end of the file (no
/// `RETURN`, nothing after it) gives rc 0, empty stdout, empty stderr --
/// `SAY` never runs because the whole program ends before its argument
/// finishes evaluating, exactly as the explicit-`EXIT` case does.
#[test]
fn a_routine_falling_off_its_own_end_in_expression_form_also_ends_the_whole_program() {
    let outcome = crate::run_program(
        "call-expr-fall-off.rex",
        b"say f(1)\nexit\nf: nop\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(outcome.stdout, b"");
    assert_eq!(outcome.stderr, b"");
}

/// **Measured**: a caller's `RESULT` is unaffected by `f(1)` appearing in
/// an expression, unlike `CALL`, which settles it on every return
/// (`Interp::invoke_named_call`'s own doc, `run/call.rs`). `result = 'before'`
/// survives `zz = f(1)` untouched.
#[test]
fn an_internal_functions_expression_form_does_not_touch_result() {
    let outcome = crate::run_program(
        "call-expr-result.rex",
        b"result = 'before'\nzz = f(1)\nsay result\nexit\nf: return 99\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"before\n");
}
