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

use super::*;

// ---- NUMERIC ----

#[test]
fn numeric_digits_changes_rounding_and_resets_to_9_with_no_expression() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"numeric digits 3\nsay 1/3"),
        b"0.333\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"numeric digits 3\nnumeric digits\nsay 1/3"),
        b"0.333333333\n".to_vec(),
        "NUMERIC DIGITS alone resets to the package default, 9"
    );
}

#[test]
fn numeric_digits_reset_refuses_a_fuzz_the_operand_form_would_refuse() {
    // Both spellings raise 33.1 with the same two substitutions -- the default
    // that could not be stored and the FUZZ in force -- measured against the
    // oracle, running exactly these clauses.
    for source in [
        b"numeric digits 20\nnumeric fuzz 15\nnumeric digits".as_slice(),
        b"numeric digits 20\nnumeric fuzz 15\nnumeric digits 9".as_slice(),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (33, 1));
        assert_eq!(raised.additional, vec![b"9".to_vec(), b"15".to_vec()]);
    }

    // A reset whose default clears the FUZZ in force still stores it.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 20\nnumeric fuzz 5\nnumeric digits\nsay digits() fuzz()"
        ),
        b"9 5\n".to_vec(),
        "the reset stores the default and leaves a FUZZ below it alone"
    );
}

#[test]
fn numeric_fuzz_changes_comparison_and_resets_to_0_with_no_expression() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 5\nnumeric fuzz 3\nsay (1.001 = 1)"
        ),
        b"1\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 5\nnumeric fuzz 3\nnumeric fuzz\nsay (1.001 = 1)"
        ),
        b"0\n".to_vec(),
        "NUMERIC FUZZ alone resets to the package default, 0"
    );
}

#[test]
fn numeric_form_scientific_engineering_and_default() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"numeric form engineering\nsay 1e10 + 0"),
        b"10E+9\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"numeric form scientific\nsay 1e10 + 0"),
        b"1E+10\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric form engineering\nnumeric form\nsay 1e10 + 0"
        ),
        b"1E+10\n".to_vec(),
        "NUMERIC FORM alone resets to the package default, SCIENTIFIC"
    );
}

#[test]
fn numeric_form_value_spellings() {
    // The keyword VALUE and the implicit `(expr)` spelling both set the
    // same setting.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric form value 'ENGINEERING'\nsay 1e10 + 0"
        ),
        b"10E+9\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"numeric form ('ENGINEERING')\nsay 1e10 + 0"),
        b"10E+9\n".to_vec()
    );
}

#[test]
fn numeric_form_value_is_not_case_insensitive() {
    // set_form_str's own rule: the runtime VALUE path does no
    // uppercasing -- measured, `numeric form value 'engineering'` is
    // 25.11 under the oracle.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"numeric form value 'engineering'").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (25, 11));
}

/// F2 (branch review, Important): `NUMERIC DIGITS`/`FUZZ`/`FORM VALUE`
/// trace `>K>` when an expression is present, and a bare `NUMERIC
/// DIGITS`/`FUZZ` (no expression) traces nothing -- both measured
/// against the oracle (`numeric_operand`'s own doc comment has the two
/// transcripts this mirrors). Mutation killed: removing either
/// `trace_keyword` call (`numeric_operand`'s own, shared by `DIGITS`/
/// `FUZZ`, or `FormValue`'s own inline one) drops that setting's own
/// `>K>` line from `interp.trace` entirely, verified by reverting each
/// in turn.
#[test]
fn numeric_digits_fuzz_and_form_value_trace_k_only_with_an_expression() {
    let mut interp = Interp::new();
    say_output_traced(
        &mut interp,
        b"numeric digits 9\nnumeric fuzz 2\nnumeric form value 'SCIENTIFIC'",
    );
    assert_eq!(
        interp.trace,
        b"     1 *-* numeric digits 9\n       >K>   \"DIGITS\" => \"9\"\n     \
          2 *-* numeric fuzz 2\n       >K>   \"FUZZ\" => \"2\"\n     \
          3 *-* numeric form value 'SCIENTIFIC'\n       >K>   \"FORM\" => \"SCIENTIFIC\"\n"
            .to_vec()
    );

    let mut interp = Interp::new();
    say_output_traced(
        &mut interp,
        b"numeric digits 3\nnumeric digits\nnumeric fuzz",
    );
    assert_eq!(
        interp.trace,
        b"     1 *-* numeric digits 3\n       >K>   \"DIGITS\" => \"3\"\n     \
          2 *-* numeric digits\n     3 *-* numeric fuzz\n"
            .to_vec(),
        "a bare NUMERIC DIGITS/FUZZ, with no expression, traces nothing"
    );
}
