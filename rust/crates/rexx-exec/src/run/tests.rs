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
use crate::Activation;
use crate::plan::{BodyKey, ProgramId};
use rexx_parse::{Program, parse_program};

/// Pushes a fresh top-level activation for `program`, the same setup
/// `Interp::run` does, so a test can drive `step` through a live
/// activation without the full instruction loop. Copied rather than
/// shared, matching every other test module in this crate (`eval.rs`,
/// `plan.rs`, `stem.rs` each keep their own).
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
    interp.activations.push(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    program
}

/// Parses `source`, activates it, and runs its whole body -- through
/// `Interp::run_activation` itself, not through a miniature of it.
///
/// `slots` is not an empty map here: `run_activation` builds its `Code`
/// with `slots: &plan.by_symbol`, which `Plan::assign` populates, so
/// every test in this module runs through the plan's fast path rather
/// than around it.
///
/// That coverage is deliberate and is an improvement: these tests
/// exercise what production runs. `eval.rs`, `stem.rs` and `plan.rs`
/// still pass `&HashMap::new()` in their own helpers, so the by-name
/// fallback keeps its expression-level coverage; what this file gains is
/// whole-program coverage of the resolved path.
fn run_source(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = activate(interp, program);
    run_activated(interp, &program)
}

/// `run_source`, with the program's directives installed first.
///
/// **In `Interp::run`'s own order** -- install, then activate, then run --
/// because a `::CLASS` must exist before the main body's first clause can
/// send it a message, which is exactly what these programs do.
fn run_source_with_directives(
    interp: &mut Interp,
    source: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = Rc::new(program);
    let program_id = ProgramId(interp.programs.len());
    interp.programs.push(Rc::clone(&program));
    interp.install_directives(program_id, &program)?;
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
    interp.activations.push(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    run_activated(interp, &program)
}

/// [`run_source_with_directives`], keeping what the program printed.
fn say_output_with_directives(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source_with_directives(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

/// `run_source`'s second half, split out so `run_source_traced` can put a
/// `TRACE` setting on the activation between the push and the run.
///
/// **`run_activation` itself, not a miniature of it.** A hand-rolled
/// `run_bounded` loop here would reproduce `run_activation`'s own `Flow`
/// dispatch arm by arm, and a second copy drifts: it needs teaching about
/// every new `Flow` variant, and condition traps live in
/// `run_activation`'s loop, one offer per activation, so a copy without
/// them cannot trap at all -- eleven trap tests failed against exactly
/// that harness while every one of the same programs matched the oracle
/// byte for byte through `run_program`. A
/// test harness that cannot reach the code under test is the sharpest
/// version of a test that cannot fail.
///
/// `activate` above already pushes exactly the activation `Interp::run`
/// pushes, so there is nothing for a copy to supply. The
/// activation is deliberately **not** popped afterwards, matching what
/// this helper did before: several tests read `interp` after the run.
fn run_activated(interp: &mut Interp, _program: &Program) -> Result<Option<ObjRef>, Failure> {
    interp.run_activation().map(Ended::value)
}

fn say_output(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

/// `run_source`, with `TRACE R` already in force for the activation it
/// pushes.
///
/// **A helper rather than an `interp.set_trace_mode(...)` line before the
/// call, which is how every one of these tests used to read.** Task 3
/// moved `trace_mode` from `Interp` onto `Activation`, so there is
/// nothing to set it on until an activation exists, and the activation is
/// what `run_source` pushes. `TRACE R` is baked in rather than passed
/// because every caller wants exactly that; a second setting gets its own
/// helper rather than a parameter nobody varies.
fn run_source_traced(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = activate(interp, program);
    interp.set_trace_mode(mode_from_setting(b"r").expect("R is a valid TRACE setting"));
    run_activated(interp, &program)
}

fn say_output_traced(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source_traced(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

// ---- assignment ----

#[test]
fn assignment_to_a_variable_a_stem_and_a_compound() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"x = 5\nsay x"),
        b"5\n".to_vec(),
        "a simple variable"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"a. = 'wd'\nsay a.1"),
        b"wd\n".to_vec(),
        "a bare stem assignment, read through an unset tail"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"a.1 = 'one'\nsay a.1\nsay a.2"),
        b"one\nA.2\n".to_vec(),
        "a compound assignment mutates one tail and leaves the rest deriving its name"
    );
}

// ---- SAY ----

#[test]
fn say_of_each_value_kind_and_of_an_omitted_expression() {
    let mut interp = Interp::new();
    assert_eq!(say_output(&mut interp, b"say 'abc'"), b"abc\n".to_vec());
    assert_eq!(say_output(&mut interp, b"say 1 + 2"), b"3\n".to_vec());
    assert_eq!(
        say_output(&mut interp, b"say .nil"),
        b"The NIL object\n".to_vec()
    );
    // No expression at all: a blank line, not nothing.
    assert_eq!(say_output(&mut interp, b"say"), b"\n".to_vec());
}

// ---- DROP ----

#[test]
fn drop_of_a_variable_returns_it_to_unset() {
    // a = 5; drop a; say a -> A (the derived name, not left over from
    // before -- and never `.nil`, which is a value and not an absence).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"a = 5\ndrop a\nsay a"),
        b"A\n".to_vec()
    );

    // The `.nil`-versus-dropped distinction `RootSet::clear_frame_slot` exists
    // for: `x = .nil` renders "The NIL object"; a dropped variable
    // derives its own name instead, never that string.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"y = .nil\ndrop y\nsay y"),
        b"Y\n".to_vec()
    );
}

#[test]
fn drop_of_a_tail_tombstones_it_without_taking_the_default() {
    // u. = 'd'; u.1 = 'one'; drop u.1; say u.1 -> U.1 (tombstoned, not
    // falling back to the default); say u.2 -> d (an untouched tail
    // still does).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"u. = 'd'\nu.1 = 'one'\ndrop u.1\nsay u.1\nsay u.2"
        ),
        b"U.1\nd\n".to_vec()
    );
}

#[test]
fn drop_of_a_whole_stem_leaves_it_looking_untouched() {
    // x. = 'd'; x.1 = 'one'; drop x.; say x.1; say x. -> X.1, X. (exactly
    // what a never-touched stem would give).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"x. = 'd'\nx.1 = 'one'\ndrop x.\nsay x.1\nsay x."
        ),
        b"X.1\nX.\n".to_vec()
    );
}

#[test]
fn drop_of_the_indirect_form() {
    // A simple variable named by another's (upcased) value.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"v = 'x'\nx = 1\ndrop (v)\nsay x"),
        b"X\n".to_vec(),
        "the wrapper's value is upcased before it names a variable"
    );

    // A whole stem, named indirectly.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'A.'\na. = 'wd'\na.1 = 'one'\ndrop (v)\nsay a.1\nsay a."
        ),
        b"A.1\nA.\n".to_vec()
    );

    // One tail, named indirectly, joined-dots key taken verbatim rather
    // than re-resolved as source -- the discriminating transcript from
    // `drop_variable`'s own doc comment.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'A.1.2'\na.1.2 = 'x'\ndrop (v)\nsay a.1.2"
        ),
        b"A.1.2\n".to_vec()
    );
}

/// Fix-round: the indirect form's value is a **subsidiary list**, not a
/// single verbatim name. Every case here uses *set* targets (the trap
/// the review named: an unset target's cleared slot and its own derived
/// name render identically, so a test built on unset targets cannot
/// tell "resolved the whole value as one name" apart from "split,
/// validated and dropped two names" -- only a set value discriminates).
#[test]
fn drop_of_the_indirect_form_is_a_subsidiary_list_of_words() {
    // Two names, blank-separated, both set and both dropped.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 1\nb = 2\nv = 'a b'\ndrop (v)\nsay a\nsay b"
        ),
        b"A\nB\n".to_vec(),
        "a blank-separated list drops every word, not one name literally spelled 'A B'"
    );

    // A run of blanks between words, and leading/trailing blanks, both
    // collapse -- still exactly two words.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 1\nb = 2\nv = '  a    b  '\ndrop (v)\nsay a\nsay b"
        ),
        b"A\nB\n".to_vec()
    );

    // A tab (`'09'x`) separates two words exactly like a blank does.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 1\nb = 2\nv = 'a'||'09'x||'b'\ndrop (v)\nsay a\nsay b"
        ),
        b"A\nB\n".to_vec(),
        "a tab byte separates words the same way a blank does"
    );

    // A mix of shapes in one list: a whole stem and a simple variable.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a. = 'd'\na.1 = 'one'\nx = 5\nv = 'a. x'\ndrop (v)\nsay a.1\nsay x"
        ),
        b"A.1\nX\n".to_vec()
    );
}

#[test]
fn drop_of_the_indirect_form_validates_every_word() {
    // A digit-led word: 31.2.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"v = '9'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (31, 2));
    assert_eq!(raised.additional, vec![b"9".to_vec()]);

    // A dot-led word: 31.3.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"v = '.x'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (31, 3));
    assert_eq!(raised.additional, vec![b".x".to_vec()]);

    // A parenthesised word: 20.928, not a second round of indirection --
    // proves the list is not recursive.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"w = 1\nv = '(w)'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (20, 928));
    assert_eq!(raised.additional, vec![b"(w)".to_vec()]);

    // **A newline does not separate.** It is not whitespace for this
    // purpose: the whole of `a`, the newline and `b` form ONE word, which
    // then fails the character-set check, and the reported name carries
    // the raw newline. Measured byte for byte against the oracle,
    // substitution included.
    //
    // This assertion is why `split_indirect_words` tests space and tab
    // explicitly instead of calling `is_ascii_whitespace`. Without it a
    // mutant that used `is_ascii_whitespace` passed all 79 tests, because
    // no other case here carries a newline, and the distinction rested
    // entirely on an end-to-end diff nobody re-runs. Carriage return,
    // form feed and vertical tab behave as the newline does.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"v = 'a'||'0a'x||'b'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (20, 928));
    assert_eq!(raised.additional, vec![b"a\nb".to_vec()]);
}

#[test]
fn drop_of_the_indirect_form_validates_before_dropping_any_of_it() {
    // a=1; b=2; v='a 9 b'; drop (v) -> Error 31.2, and NEITHER a nor b
    // is dropped, even though `a` sits before the bad word -- measured
    // against the oracle (SIGNAL ON SYNTAX recovery there shows both
    // untouched). The whole list validates before any drop runs.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"a = 1\nb = 2\nv = 'a 9 b'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (31, 2));
    assert_eq!(raised.additional, vec![b"9".to_vec()]);

    // The activation is still on the stack (`run_source` does not pop
    // on error), so `a`'s slot is still directly inspectable.
    let a_slot = interp.slot_of(b"A");
    let frame = interp.activation().frame;
    let a_value = interp
        .roots
        .frame_slot(frame, a_slot)
        .expect("a must still be set");
    assert_eq!(
        &*interp.to_text(a_value),
        b"1",
        "a must not have been dropped before the list's third word failed validation"
    );
}

#[test]
fn drop_of_the_indirect_form_on_an_empty_or_blanks_only_value_is_a_no_op() {
    // Measured: `v=''`/`v='   '` both run clean under the oracle -- zero
    // words, nothing to validate or drop.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"v = ''\ndrop (v)\nsay 'after'"),
        b"after\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"v = '   '\ndrop (v)\nsay 'after'"),
        b"after\n".to_vec()
    );
}

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
fn numeric_digits_reset_reports_a_conflict_exactly_as_if_9_were_typed() {
    // numeric digits 20; numeric fuzz 15; numeric digits -> 33.1, ("9")
    // rejected against the still-15 fuzz -- measured against the oracle.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"numeric digits 20\nnumeric fuzz 15\nnumeric digits",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (33, 1));
    assert_eq!(raised.additional, vec![b"9".to_vec(), b"15".to_vec()]);
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

// ---- EXIT ----

#[test]
fn exit_with_and_without_an_expression() {
    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"exit").expect("bare exit runs");
    assert_eq!(value, None);

    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"exit 42").expect("exit with a result runs");
    let value = value.expect("EXIT 42 carries a result");
    assert_eq!(&*interp.to_text(value), b"42");
}

#[test]
fn exit_code_for_converts_the_result_the_way_the_oracle_does() {
    // The whole transcript this task's report re-verifies against the
    // oracle: a bare EXIT and a huge, fractional or non-numeric result
    // all leave the code at 0; a literal in i32 range converts exactly,
    // and an arithmetic result (EXIT's own unary minus counts) is
    // already rounded to the active NUMERIC DIGITS by the time it gets
    // here, which is where the negative-vs-positive asymmetry the report
    // measured against the oracle comes from.
    let mut interp = Interp::new();
    assert_eq!(interp.exit_code_for(None), 0, "a bare EXIT");

    let value = run_source(&mut interp, b"exit 2147483647")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        2147483647,
        "INT32_MAX exactly"
    );

    let value = run_source(&mut interp, b"exit 2147483648")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        0,
        "one past INT32_MAX falls back to 0"
    );

    let value = run_source(&mut interp, b"exit 5.9")
        .unwrap()
        .expect("a result");
    assert_eq!(interp.exit_code_for(Some(value)), 0, "fractional");

    let value = run_source(&mut interp, b"exit 5.0")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        5,
        "a whole number spelled with a point"
    );

    let value = run_source(&mut interp, b"exit 'abc'")
        .unwrap()
        .expect("a result");
    assert_eq!(interp.exit_code_for(Some(value)), 0, "non-numeric");

    // The asymmetry: -2147483647 is 0 - 2147483647, rounded to the
    // *active* DIGITS (9 by default) the moment it is created, landing
    // one past INT32_MIN; raising DIGITS before the subtraction removes
    // the rounding and the failure with it.
    let value = run_source(&mut interp, b"exit -2147483647")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        0,
        "rounded to 9 digits at creation, one past INT32_MIN"
    );

    let value = run_source(&mut interp, b"numeric digits 20\nexit -2147483647")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        -2147483647,
        "no rounding at DIGITS 20, exact"
    );
}

// ---- LABEL and NOP ----

#[test]
fn a_label_is_a_traced_no_op() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"here: say 'hit'"),
        b"hit\n".to_vec(),
        "the label itself produces no output of its own"
    );
}

#[test]
fn nop_is_a_no_op() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"nop\nsay 'after'"),
        b"after\n".to_vec()
    );
}

// ---- IF/THEN/ELSE ----

/// The discriminating shape for a wrong `false_target`/`then_exit`: a
/// true condition whose `ELSE` branch has a **different** side effect
/// than the `THEN` branch. A version that lets the true path fall
/// through into the `ELSE` marker without skipping it (the naive
/// fallthrough this task's whole design exists to avoid -- see
/// `run_bounded`'s doc comment) runs *both* assignments and prints
/// `abXc`. A version that never runs the `ELSE` branch on the false path
/// at all prints `aXc` for the companion case below. Only the correct
/// wiring prints `abc` and `aXc` respectively.
#[test]
fn if_then_else_runs_exactly_one_branch() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 1 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
        ),
        b"abc\n".to_vec(),
        "true: runs the THEN branch and skips the ELSE branch entirely"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 0 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
        ),
        b"aXc\n".to_vec(),
        "false: runs the ELSE branch and skips the THEN branch entirely"
    );
}

#[test]
fn if_then_with_no_else_falls_through_on_false() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 0 then a = a || 'b'\na = a || 'c'\nsay a"
        ),
        b"ac\n".to_vec()
    );
}

/// The `ast.rs`/`block.rs`-derived discriminating case: an `IF`/`ELSE
/// IF`/`ELSE` chain (no `DO`, which Task 11 owns) where each link's
/// `false_target` is the next link's own condition and the whole
/// chain's resume point sits after all of them. Run once per branch so
/// a wrong `false_target` (skipping or re-testing a link) and a wrong
/// `then_exit`/resume (landing inside a later link, matching
/// `if_else_chain.rex`'s own framing) are both visible.
#[test]
fn if_else_if_chain_takes_exactly_one_link() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\na = ''\nif n = 1 then a = a || 'one'\nelse if n = 2 then a = a || 'two'\nelse if n = 3 then a = a || 'three'\nelse a = a || 'other'\na = a || '-after'\nsay a"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "one-after\n"),
        ("2", "two-after\n"),
        ("3", "three-after\n"),
        ("4", "other-after\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}"
        );
    }
}

#[test]
fn if_condition_that_is_not_0_or_1_raises_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"if 'x' then nop").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 1));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// A comma list is 34.6 regardless of which element fails, never 34.1.
/// Measured against the oracle (brief's own transcript).
#[test]
fn if_condition_that_is_a_comma_list_raises_34_6_not_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"if 'x', 1 then nop").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 6));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

#[test]
fn a_true_comma_list_condition_is_an_and_of_its_parts() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"if 1, 1 then say 'hit'\nelse say 'miss'"),
        b"hit\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"if 1, 0 then say 'hit'\nelse say 'miss'"),
        b"miss\n".to_vec()
    );
}

// ---- SELECT / WHEN ----

#[test]
fn select_when_runs_exactly_the_first_matching_when() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\nr = ''\nselect\n  when n = 1 then r = r || 'one'\n  when n = 2 then r = r || 'two'\n  when n = 3 then r = r || 'three'\n  otherwise r = r || 'other'\nend\nr = r || '-after'\nsay r"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "one-after\n"),
        ("2", "two-after\n"),
        ("3", "three-after\n"),
        ("4", "other-after\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}"
        );
    }
}

/// The `select_when_bodies.rex` discriminating shape, reproduced without
/// `DO` (Task 11's): each `WHEN`'s consequence is a nested `SELECT`
/// spanning several flat instructions of its own
/// (`Select`/`When`/`Then`/two assignments/`End`), so a wrong exit that
/// lands even one instruction into a *later* `WHEN`'s span, rather than
/// cleanly past the whole outer `SELECT`, shows up as an extra or
/// missing accumulator update. Run for every `n` so a wrong exit wired
/// to (for instance) always resume after the *first* `WHEN` would still
/// be caught on `n = 2` or `n = 3`.
#[test]
fn select_when_wrong_exit_would_land_in_a_later_whens_multi_instruction_body() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\nr = ''\nselect\n  when n = 1 then select\n    when 1 = 1 then r = r || 'w1a'\n    otherwise nop\n  end\n  when n = 2 then select\n    when 1 = 1 then r = r || 'w2a'\n    otherwise nop\n  end\n  when n = 3 then select\n    when 1 = 1 then r = r || 'w3a'\n    otherwise nop\n  end\n  otherwise r = r || 'w4a'\nend\nr = r || '-done'\nsay r"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "w1a-done\n"),
        ("2", "w2a-done\n"),
        ("3", "w3a-done\n"),
        ("4", "w4a-done\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}: exactly one nested SELECT's body ran, and nothing else did"
        );
    }
}

/// `when 1 = 1 then` followed immediately by `when 2 = 2 then n = 42` is
/// the second `WHEN`'s absorption into the first's (empty) consequence
/// (`ast.rs`'s own doc comment on `Select::whens`): the second `WHEN` is
/// never collected, and its own `exit` is permanently `None`. Measured
/// against the oracle: prints `0`, rc 0 -- the absorbed `WHEN`'s own
/// condition being true (`2 = 2`) must not run `n = 42`, and `OTHERWISE`
/// must not run either, because one true `WHEN` (the outer one) already
/// ended the whole `SELECT`.
#[test]
fn an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\nselect\n  when 1 = 1 then\n    when 2 = 2 then n = 42\n  otherwise\n    n = 99\nend\nsay n"
        ),
        b"0\n".to_vec()
    );
}

/// The true-condition-variant is accepted at rc 0 (`ast.rs:776`, and the
/// brief's own transcript) -- the *false*-condition variant is a
/// separate, upstream oracle segfault (SF #2018) and is deliberately not
/// probed here.
#[test]
fn when_absorbing_a_when_parses_and_runs_at_rc_0() {
    let mut interp = Interp::new();
    assert_eq!(
        run_source(
            &mut interp,
            b"select\n  when 1 = 1 then\n    when 2 = 2 then nop\nend"
        )
        .expect("accepted, rc 0"),
        None
    );
}

/// **Critical, found by review, not by this task's own probes.** The
/// mutation this kills is exactly the one the old `Ok(Flow::Next)`
/// arm *was*: treating an absorbed `WHEN`'s own condition as never
/// evaluated at all, rather than evaluated and discarded. A version
/// that reverts `InstructionKind::When`'s arm to a bare `Ok(Flow::
/// Next)` makes this test hang around `run_source` returning `Ok`
/// (prints `after`, rc 0) where the oracle raises 42.3 at rc 214 --
/// `an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise`,
/// just above, cannot distinguish the two models (both give `n = 0`
/// for a side-effect-free true condition), which is exactly why every
/// probe before this review used one and missed this.
#[test]
fn an_absorbed_whens_raising_condition_escapes_even_though_its_own_consequence_never_runs() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select\n  when 1 = 1 then\n    when 1 / 0 then nop\n  otherwise nop\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
}

/// The companion half: a **true** absorbed condition with a printable
/// side effect still never runs its own consequence (matching the
/// existing `n = 0` test, restated with `SAY` so a wrong "the
/// absorbed WHEN's branch is taken" model would be caught by output
/// content rather than only by a variable's final value) -- measured,
/// this task's own report: `ABSORBED-RAN` never prints, only `after`.
#[test]
fn an_absorbed_whens_true_condition_still_never_runs_its_own_consequence() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select\n  when 1 = 1 then\n    when 2 = 2 then say 'ABSORBED-RAN'\nend\nsay 'after'"
        ),
        b"after\n".to_vec(),
        "a reverted fix would print ABSORBED-RAN first"
    );
}

/// F3, found by review: unlike a plain `WHEN`'s absorbed form, a
/// `WHEN CASE`'s own absorbed form branches to its own `false_target`
/// on a false match. The mutation this kills: reverting
/// `InstructionKind::WhenCase`'s arm to the old evaluate-and-discard
/// shape (matching `When`'s own, still correct for `When`) makes this
/// program print only `after`, where the oracle -- and this fix --
/// print `O` then `after`. Verified by mutation: reverting made this
/// test fail with exactly that wrong output, while
/// `an_absorbed_whens_true_condition_still_never_runs_its_own_
/// consequence` (a `WHEN`, not a `WHEN CASE`) stayed green, confirming
/// the fix is scoped to `WhenCase` alone.
#[test]
fn an_absorbed_whencases_false_condition_branches_to_its_own_false_target() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\nend\nsay 'after'"
        ),
        b"O\nafter\n".to_vec()
    );
}

/// The companion true-match case, matching the coordinator's own
/// phrase "matches on both sides": a true absorbed `WhenCase` still
/// never runs its own consequence, exactly like a plain `WHEN`'s own
/// true-absorbed case -- this is *not* new behaviour F3 introduced, it
/// is what this crate already did before the fix, re-pinned here so a
/// future change to the false-path fix cannot silently start running
/// the true path's own consequence too.
#[test]
fn an_absorbed_whencases_true_condition_still_never_runs_its_own_consequence() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 2\n  when 2 then\n    when 2 then say 'INNER'\n  otherwise say 'O'\nend\nsay 'after'"
        ),
        b"after\n".to_vec()
    );
}

#[test]
fn select_with_no_when_true_and_no_otherwise_raises_7_3() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 1 = 0 then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (7, 3));
    assert_eq!(raised.additional, Vec::<Vec<u8>>::new());
}

/// F3's own perimeter, found by review: an absorbed `WhenCase`'s
/// false-branch escape landing directly on `END` reports 7.3 at the
/// escape's own residual indent (4, the absorbed condition's own
/// depth of 6 minus 2), not `END`'s own lexical `static_indent` (0,
/// top-level `SELECT`). The mutation this kills: removing the
/// `self.pending_escape_indent = Some(...)` assignment in
/// `InstructionKind::WhenCase`'s own false-branch arm (or reverting
/// `record_failure_site` to ignore it) makes this test's own `indent`
/// read back `0` instead of `4`, while `select_with_no_when_true_and_
/// no_otherwise_raises_7_3`, just above -- the *ordinary*, non-
/// absorbed 7.3 path, unaffected by this fix -- stays green either
/// way, confirming the fix is scoped to the escape path alone.
#[test]
fn an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select case 2\n  when 2 then\n    when 3 then nop\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (7, 3));
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4);
}

/// The same shape, one `DO` level deeper -- `indent_offset`'s own doc
/// comment (`lib.rs`) has the full argument for why the answer stays
/// `6`, not `8`: the offset past an *ordinary* `SELECT`-level
/// construct's own depth is the constant `4`, not a function of how
/// deep the absorbed condition itself sits, and both grow by the same
/// amount together under nesting. The mutation this kills: reverting
/// `indent_offset`'s own assignment to `self.clause_state.current_value_indent.
/// saturating_sub(2)` (an earlier, wrong version of this same fix)
/// makes this test read back `6` (`8 - 2`) instead of `4`, while the
/// non-nested version just above still reads back the right answer
/// either way (`6 - 2` and the constant `4` coincide at the top
/// level) -- this is the one test that distinguishes them.
#[test]
fn an_absorbed_whencases_escape_to_end_reports_the_same_constant_offset_nested() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"do i = 1 to 1\n  select case 2\n    when 2 then\n      when 3 then nop\n  end\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (7, 3));
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 6);
}

/// **F-EX1, Important, found by the whole-branch review, not by this
/// task's own probes.** An absorbed `WhenCase`'s own false-branch
/// escape landing on `OTHERWISE` used to leave `Select`'s own arm
/// through a bare `Flow::Goto` that `leave_select` had no way to
/// recognise, so `OTHERWISE`'s own body ran under whichever *outer*
/// construct received that `Goto` -- with no `SELECT` frame on the
/// search a `LEAVE` naming the enclosing `SELECT LABEL` needs to find.
/// The mutation this kills: reverting the escape-redirect check in
/// `Select`'s own arm (the `if let Flow::Goto(target) = flow && *
/// otherwise == Some(target)` branch, calling `run_otherwise` instead
/// of forwarding the bare `Goto`) makes `leave s` search *outward* from
/// outside this `SELECT` and find nothing, raising 28.3 at rc 228
/// instead of resuming past the `SELECT` cleanly -- reproduced by
/// actually reverting it (this task's report has the transcript).
#[test]
fn an_absorbed_whencases_escape_to_otherwise_still_finds_the_enclosing_selects_own_label() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select label s case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\n  \
              leave s\nend\nsay 'after'"
        ),
        b"O\nafter\n".to_vec()
    );
}

/// The companion half of F-EX1: a **named `ITERATE`** inside the same
/// escaped `OTHERWISE`, naming the enclosing `SELECT LABEL`, is 28.5
/// (matches the name, but a `SELECT` is never a repetitive block) --
/// not 28.4 (no match at all), which is what it read as before this
/// fix, because the search never reached `leave_select`'s own name
/// check at all. Distinguishes "the frame is restored" from "the frame
/// is restored, and the specific consumption rule inside it still
/// applies", which the `LEAVE` test above alone cannot.
#[test]
fn an_absorbed_whencases_escape_to_otherwise_reports_a_named_iterate_as_28_5_not_28_4() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select label s case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\n  \
          iterate s\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 5));
}

#[test]
fn select_with_no_when_true_and_an_otherwise_runs_it_without_error() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
        ),
        b"lo\n".to_vec()
    );
}

#[test]
fn when_condition_that_is_not_0_or_1_raises_34_2() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 'x' then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 2));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// **The coordinator's own finding, fixed after the first round.** A
/// `WHEN`'s own `step` arm is a pure no-op (`Select`'s own arm reads it
/// as data instead), so a raise while evaluating its *condition* never
/// went through a `step_in_temps_frame` call for the `WHEN` itself --
/// the first version of this task attributed it to the enclosing
/// `SELECT`, wrong clause *and* wrong line, measured against the
/// oracle. `record_failure_site`'s own doc comment on `Select`'s call
/// sites has the fix. Checks `failure_site` directly (line and clause
/// text), which `raised.number`/`.sub` alone -- every other test in
/// this file -- cannot: both were already unaffected by the bug, since
/// the bug is entirely in *which clause* gets echoed, not in which
/// condition failed.
#[test]
fn a_when_conditions_own_failure_is_attributed_to_the_when_not_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 'x' then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp
        .failure_site
        .expect("a raised condition always resolves a site when source is Some")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 2, "the WHEN's own line, not the SELECT's (line 1)");
    assert_eq!(
        text,
        b"when 'x' ".to_vec(),
        "the WHEN's own clause text, not \"select\""
    );
}

/// The line has to move with the failing `WHEN`, not merely differ from
/// the `SELECT`'s -- a test whose expected line is the first `WHEN`'s
/// cannot tell a correct resolution from one that defaults to
/// "whichever `WHEN` this loop happens to be looking at first".
#[test]
fn the_second_of_two_whens_own_failure_moves_the_line_with_it() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\nwhen 'x' then nop\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 3, "the second WHEN's own line, not the first's (2)");
    assert_eq!(text, b"when 'x' ".to_vec());
}

/// A `SELECT CASE` expression that itself raises is the `SELECT`'s own
/// clause -- confirmed against the oracle rather than assumed, since
/// `case` is evaluated directly inside `Select`'s own `step` call and
/// the coordinator asked this be checked, not taken on faith.
#[test]
fn a_select_cases_own_expression_failure_is_attributed_to_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select case (1/0)\nwhen 1 then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 1);
    assert_eq!(text, b"select case (1/0)".to_vec());
}

/// A `WhenCase` value expression that raises is the `WHEN`'s own
/// clause, the same rule as a plain `WHEN`'s condition -- both go
/// through `Select`'s own explicit-match-and-record path, never
/// through `step_in_temps_frame` for the `When`/`WhenCase` node itself.
#[test]
fn a_whencase_values_own_failure_is_attributed_to_the_when_not_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select case 1\nwhen (1/0) then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 2);
    assert_eq!(text, b"when (1/0) ".to_vec());
}

/// A raise inside an `OTHERWISE` branch was already correct before this
/// round's fix (`OTHERWISE`'s own body runs through the outer loop's
/// ordinary `step_in_temps_frame`, never through `Select`'s own
/// explicit-match path), and stays that way -- checked because the
/// coordinator asked for it explicitly, not assumed from the `WHEN` fix.
#[test]
fn a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\notherwise\n  say 1/0\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 4);
    assert_eq!(text, b"say 1/0".to_vec());
}

/// A raise inside a matched `WHEN`'s **body** (not its condition) has to
/// go through `run_bounded`'s own `step_in_temps_frame` calls with
/// `source` actually threaded through, which is a different path from
/// every other test in this section: those all check a condition/value
/// expression `Select`'s own arm evaluates directly, and
/// `a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause`
/// runs through the *outer* loop's `step_in_temps_frame`, never through a
/// `run_bounded` nested inside `If`/`Select`'s own arm. That last
/// distinction matters and an earlier wording of it was wrong: in the
/// test harness `run_source` routes everything through its own top-level
/// `run_bounded` with `source` supplied directly, so "never through
/// `run_bounded` at all" is true of the production outer loop only.
///
/// This is round 1's own defect class -- an error escaping a nested
/// `run_bounded` call misattributed to the enclosing construct -- and no
/// existing test exercises the path that would regress if `source`
/// stopped being threaded into `run_bounded`. Confirmed by mutation, not
/// assumed: passing `None` in place of `source` at the two call sites,
/// which are `If`'s and `Select`'s and not two of `Select`'s own, made
/// this test and the `IF` one below fail while leaving all 102
/// pre-existing tests green. Restored immediately after.
#[test]
fn a_raise_inside_a_matched_whens_body_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 1 = 1 then\n  say 1/0\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        line, 3,
        "the WHEN's own body clause, not the SELECT's (line 1)"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

/// The `IF` analogue of the `WHEN`-body test above: a raise inside the
/// matched `THEN` branch's own body, which likewise only ever reaches
/// `step_in_temps_frame` through `run_bounded`. Confirmed by the same
/// mutation (`None` for `source` at `If`'s own `run_bounded` call site
/// made this fail too, alongside the `WHEN`-body test, both restored
/// after).
#[test]
fn a_raise_inside_an_ifs_then_body_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"if 1 = 1 then\n  say 1/0").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        line, 2,
        "the THEN branch's own body clause, not the IF's (line 1)"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

// ---- SELECT CASE / WhenCase ----

/// **The central `WhenCase` rule.** `select case 2` / `when 1, 2 then`
/// matches (an OR of `==`), while a plain `select` / `when 1, 2` on the
/// same non-logical value is 34.6 (an AND, each element checked for
/// `0`/`1`) -- the two commas parse into the same-looking node and mean
/// opposites (`ast.rs:801-815`, and the brief's own framing).
#[test]
fn whencase_comma_is_an_or_of_equals_the_opposite_of_a_plain_whens_and() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 2\n  when 1, 2 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"hit\n".to_vec(),
        "SELECT CASE: an OR of == comparisons"
    );

    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 1, 2 then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(
        (raised.number, raised.sub),
        (34, 6),
        "plain SELECT: 2 is not a logical value, an AND-with-a-check, not an OR-of-=="
    );
    assert_eq!(raised.additional, vec![b"2".to_vec()]);
}

/// `==` is strict: byte-for-byte, no padding, no numeric awareness.
/// Measured (D15's own example, restated in the brief): `'007'` does not
/// match `when 7`, because the two are not byte-identical.
#[test]
fn select_case_compares_with_strict_equality_not_numeric_equality() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case '007'\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"miss\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 7\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"hit\n".to_vec()
    );
}

#[test]
fn select_case_evaluates_its_own_expression_and_runs_exactly_the_matching_when() {
    let source = |v: &str| -> Vec<u8> {
        format!("select case {v}\n  when 1 then say 'one'\n  when 2 then say 'two'\n  otherwise say 'other'\nend")
            .into_bytes()
    };
    for (v, expected) in [("1", "one\n"), ("2", "two\n"), ("3", "other\n")] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(v)),
            expected.as_bytes().to_vec(),
            "v = {v}"
        );
    }
}

// ---- SELECT with a LABEL, and a nested SELECT/IF ----

#[test]
fn a_labelled_select_with_otherwise_runs_normally() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select label s\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
        ),
        b"lo\n".to_vec()
    );
}

#[test]
fn a_select_nested_inside_an_ifs_then_branch_is_fully_resolved_before_resuming() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 1 then select\n  when 1 = 1 then a = a || 'b'\nend\na = a || 'c'\nsay a"
        ),
        b"abc\n".to_vec()
    );
}

// ---- DO/LOOP: every LoopKind ----

#[test]
fn a_simple_do_block_runs_its_body_exactly_once_with_no_control() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do\nsay 'once'\nend"),
        b"once\n".to_vec()
    );
}

/// `LOOP` alone is `DO FOREVER`; `DO` alone is a block, not a loop
/// (`ast.rs`'s own doc comment on `LoopKind::Simple`/`Forever`) --
/// proven here by the fact that only the `LOOP` form is stopped by a
/// bare `LEAVE`.
#[test]
fn loop_alone_is_forever_and_do_alone_is_a_block_not_a_loop() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\nloop\nn = n + 1\nif n = 3 then leave\nend\nsay n"
        ),
        b"3\n".to_vec()
    );
}

#[test]
fn do_forever_runs_until_a_bare_leave() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\ndo forever\nn = n + 1\nif n = 3 then leave\nend\nsay n"
        ),
        b"3\n".to_vec()
    );
}

#[test]
fn do_count_repeats_exactly_the_evaluated_number_of_times() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"n = 0\ndo 3\nn = n + 1\nend\nsay n"),
        b"3\n".to_vec()
    );
    // The repeat count is an expression, evaluated once.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"n = 0\ndo 1 + 2\nn = n + 1\nend\nsay n"),
        b"3\n".to_vec()
    );
    // Zero repetitions is legal and runs the body no times at all.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"n = 0\ndo 0\nn = n + 1\nend\nsay n"),
        b"0\n".to_vec()
    );
}

/// F1, found by review: a bare count `DO n` traced no `>K>` line at
/// all, where the oracle traces it tagged `FOR` -- the same tag an
/// explicit `DO ... FOR n` gets, measured (this task's own report).
/// A bare count reaches that line through `HeaderRole::Count`'s `FOR`
/// answer in `HeaderRole::keyword`, which is what `echo_header_value`
/// hands `trace_keyword`. It is asserted here because the report's own
/// verification claimed `>K>` was checked while never actually running a
/// bare-count program through it.
#[test]
fn a_bare_repeat_count_traces_as_for_the_same_as_an_explicit_one() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"do 2\nnop\nend");
    // `>K>` fires exactly once, on the first pass, matching every
    // other single-evaluation control-setup keyword (`TO`/`BY`/
    // `OVER`) -- the third `do 2` re-echo is the exit-check pass
    // `run_repeating`'s own per-pass re-echo already covers, verified
    // by running this exact assertion once and reading the bytes
    // back rather than hand-composing them (`this file's own report
    // has the trap: a hand-guessed expectation here was short by
    // exactly this one pass on the first attempt).
    assert_eq!(
        interp.trace,
        b"     1 *-* do 2\n       >K>   \"FOR\" => \"2\"\n     2 *-*   nop\n     \
          3 *-* end\n     1 *-* do 2\n     2 *-*   nop\n     3 *-* end\n     1 *-* do 2\n"
            .to_vec()
    );
}

/// F4, found by review: a comma-list condition traced no `>>>` for its
/// own elements at all, only the list's overall result, under
/// `TRACE R` (not merely `TRACE I`) -- measured, the oracle shows
/// *three* `>>>` lines for `if 1, 1 then`, one per element plus one
/// for the list, and `eval_logical_list`'s own fix (`eval.rs`) is what
/// this test defends. The mutation it kills: removing that function's
/// `self.trace_result(indent, &text)` call drops the middle two lines,
/// leaving only the third (`eval_condition`'s own, unaffected) --
/// caught by this test and by neither of the pre-existing comma-list
/// tests (`if_condition_that_is_a_comma_list_raises_34_6_not_34_1`,
/// the short-circuit test), since neither one traces anything.
#[test]
fn a_comma_list_conditions_own_elements_each_trace_their_result_under_trace_r() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"if 1, 1 then nop");
    assert_eq!(
        interp.trace,
        b"     1 *-* if 1, 1 \n       >>>   \"1\"\n       >>>   \"1\"\n       >>>   \"1\"\n     \
          1 *-*   then\n     1 *-*     nop\n"
            .to_vec()
    );
}

/// Found while re-verifying F4 rather than assumed clean: fixing the
/// comma-list `>>>` gap exposed that `DO UNTIL`'s own re-echo was
/// wired to the wrong site. The mutation this kills is two-sided --
/// removing `is_until_loop`'s own gate on the top-of-loop echo makes
/// a multi-pass `UNTIL` loop echo its clause **twice** per pass (once
/// there, once at `UNTIL`'s own site); removing `UNTIL`'s own
/// unconditional echo instead makes it echo **zero** times on the
/// first pass (the top-of-loop site is `first_pass`-gated and never
/// fires before the loop has gone around once). Only the fix in
/// between gets exactly one echo per completed pass, matching the
/// oracle (`t13_until_multi.rex`, this task's report).
#[test]
fn do_until_re_echoes_its_clause_exactly_once_per_pass_not_twice_or_zero() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"n = 0\ndo until n = 2\nn = n + 1\nend\nsay n");
    assert_eq!(
        interp.trace,
        b"     1 *-* n = 0\n       >>>   \"0\"\n     2 *-* do until n = 2\n     \
          3 *-*   n = n + 1\n       >>>     \"1\"\n     4 *-* end\n     2 *-* do until n = 2\n       \
          >K>     \"UNTIL\" => \"0\"\n     3 *-*   n = n + 1\n       >>>     \"2\"\n     4 *-* end\n     \
          2 *-* do until n = 2\n       >K>     \"UNTIL\" => \"1\"\n     5 *-* say n\n       >>>   \"2\"\n"
            .to_vec()
    );
}

#[test]
fn do_with_takes_the_loud_path() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do with index i over 'x'\nsay i\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Review finding I1: `instruction.kind` here is `InstructionKind::Do`,
    // which this crate implements -- `lib.rs`'s `owned_message` must not
    // attribute this to a phase (there is none to blame; `DO WITH` is
    // Phase 5's *reason*, but the message names the construct, not the
    // reason). Mutation-kill for deleting the `None` carve-out in
    // `owned_message`: this assertion is what turns that deletion into a
    // failure here.
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

#[test]
fn do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do counter c i = 1 to 3\nnop\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Same mutation-kill as `do_with_takes_the_loud_path`, above.
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

// ---- DO i = TO/BY/FOR (controlled), and DO OVER ----

#[test]
fn a_controlled_loop_runs_to_then_by_then_stops() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 1 to 3\nsay i\nend"),
        b"1\n2\n3\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 10 to 1 by -4\nsay i\nend"),
        b"10\n6\n2\n".to_vec(),
        "measured against the oracle, do_loop_forms.rex's own transcript"
    );
}

/// A non-whole control value is legal (Step 2's own table): `do i = 1.5
/// to 3` steps by fractional values, not an error.
#[test]
fn a_controlled_loops_own_values_need_only_be_numeric_not_whole() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 1.5 to 3\nsay i\nend"),
        b"1.5\n2.5\n".to_vec()
    );
}

/// `BY 0` loops forever -- behaviour to reproduce, not an error
/// (Step 2's own table) -- bounded here by a `LEAVE` so the test itself
/// terminates.
#[test]
fn a_controlled_loop_with_by_0_loops_forever() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 0\ndo j = 1 by 0 to 3\ni = i + 1\nif i > 5 then leave\nend\nsay i"
        ),
        b"6\n".to_vec(),
        "measured against the oracle"
    );
}

#[test]
fn a_controlled_loops_own_for_caps_the_iteration_count_independent_of_to() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\ndo i = 1 to 100 for 3\nn = n + 1\nend\nsay n"
        ),
        b"3\n".to_vec()
    );
}

/// The control variable is bound to its own header value **before** the
/// loop's own bound test, even for a loop that ends up running zero
/// iterations -- measured against the oracle.
#[test]
fn the_control_variable_is_bound_even_when_the_loop_never_runs_its_body() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 5 to 3\nsay 'never'\nend\nsay i"),
        b"5\n".to_vec()
    );
}

/// A compound variable as a `DO` control variable is bound on every
/// pass, through the same tail resolution `say cv.j` already uses, not
/// stored as a simple variable literally named `CV.J`. Measured against
/// the oracle: `1`/`2`/`3` inside the loop, then `4` twice -- the
/// loop's own final bound-test value, read back through the same tail
/// both by `cv.j` and by the literal `cv.7`.
#[test]
fn a_compound_control_variable_is_bound_on_every_pass() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"j = 7\ndo cv.j = 1 to 3\nsay cv.j\nend\nsay cv.j\nsay cv.7"
        ),
        b"1\n2\n3\n4\n4\n".to_vec()
    );
}

/// **Pairs with the test above**: an ordinary simple control variable
/// is unaffected by routing a stem or compound one through
/// `assign_expr_target` -- the fast, unmodified slot write is still the
/// only path a simple control ever takes.
#[test]
fn a_simple_control_variable_still_binds_through_the_fast_slot_path() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do ii = 1 to 3\nend\nsay ii"),
        b"4\n".to_vec()
    );
}

/// The compound's tail re-resolves fresh on every pass, against
/// whichever *current* value the tail variable holds -- not the tail
/// the loop's header resolved once at setup. Measured against the
/// oracle: the body's own `i = i + 1` moves which tail of `a.` this
/// loop's own read-and-increment step resolves to on the very next
/// pass, so `TO 3` keeps comparing against a fresh, still-default `0`
/// tail instead of the one `a.i` incremented, and the loop runs to `i
/// = 8` (the `LEAVE` bound) rather than stopping at `i = 4`.
#[test]
fn a_compound_control_variables_tail_re_resolves_every_pass() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a. = 0\ni = 1\ndo a.i = 1 to 3\nif i > 7 then leave\ni = i + 1\nend\nsay i"
        ),
        b"8\n".to_vec()
    );
}

/// **Crosses TO-bound-only termination with a pre-existing stem default
/// that masks a missing write** (review round 1, I3) -- the shape
/// `a_compound_control_variables_tail_re_resolves_every_pass` above does
/// not reach, because that test's own termination is an independent
/// `LEAVE` on `i`, not the loop's own `TO` bound. Here nothing but `TO 7`
/// ends the loop, and `a.`'s own pre-existing default (`a. = 0`) is
/// exactly what would paper over a write that lands nowhere: a `stem_
/// get` miss falls back to that default rather than to `NOVALUE`'s
/// derived-name text, so a broken write does not fail loudly here, it
/// makes the read-back see `0` on every pass and the loop never reach
/// its bound at all.
///
/// Reproduces `DO::test_DO_standardTest2P`'s own mechanism (`i`
/// flip-flopping which tail of `a.` the control resolves to, so `c`
/// counts passes while the bound is carried by the tail each `i` visits
/// in turn) rather than a synthetic shape, because that is the `base/
/// keyword` body a write-side-only regression hung on for real (fix
/// round 1's own report has the transcript). `FOR 1000` is a safety
/// cap, not a behavioural change: the oracle and this crate both end the
/// loop via `TO 7` at pass 14, so the cap never fires on correct code,
/// and it is what turns "hangs forever" into "counts 1000 instead of
/// 14" if this ever regresses -- a `DO` loop in a permanent test must
/// not be able to hang the suite that runs it.
#[test]
fn a_compound_controls_to_bound_survives_a_masking_stem_default() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 1\na. = 0\nc = 0\ndo a.i = 1 to 7 for 1000\nc = c + 1\nif i = 1 then i = 2\nelse i = 1\nend\nsay c"
        ),
        b"14\n".to_vec()
    );
}

/// A bare stem as a `DO` control variable (`do cv. = 13`) binds through
/// `stem_assign`, the same "replace and rebind" an ordinary `cv. = 13`
/// assignment uses. Paired with the compound tests above because a
/// stem's own spelling has no tail to resolve: it is the shape this fix
/// must leave working, not the one it corrects.
///
/// **The second assertion is the one that can actually fail (review
/// round 1, I1/I2).** `read_stem` returns whatever object sits in the
/// slot with no check that it is a `Body::Stem`, so a flat scalar write
/// there is invisible to a bare-stem *read* of the same name -- the
/// first assertion's `say cv.` cannot tell a correct `stem_assign` write
/// from the old flat write apart, and measured directly against the
/// pre-fix tree (`git archive 1c2300d9`), it does not: the pre-fix build
/// prints the identical `24`/`11`. A *tail* of the same stem, touched
/// anywhere else in the body, goes through `stem_set`/`stem_get`
/// instead, both of which `expect` a `Body::Stem` at that slot and
/// panic otherwise -- measured on the pre-fix tree, `do cv. = 13\ncv.1 =
/// 99\nleave\nend\nsay cv.1\nsay cv.` panics at `stem.rs:288:60: a live
/// value`, rc 101, where the oracle and this crate's fixed build both
/// answer `99`/`13`.
#[test]
fn a_stem_control_variable_binds_through_stem_assign() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 0\ndo cv. = 13\nif i > 10 then leave\ni = i + 1\nend\nsay cv.\nsay i"
        ),
        b"24\n11\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do cv. = 13\ncv.1 = 99\nleave\nend\nsay cv.1\nsay cv."
        ),
        b"99\n13\n".to_vec()
    );
}

/// **A stem-controlled `DO` inside an `INTERPRET`, which is where the loop
/// runs with no precomputed slot at all.**
///
/// The neighbour above is the same loop written in the compiled body,
/// where the write and the re-test's read each take the stem's slot off
/// `Code::compound`'s entry. A fragment carries no plan (`Code::plan`'s own
/// doc says why), so both fall back to resolving the name -- the path this
/// program is the witness for, and the one where a slot must **not** be
/// carried across a body boundary.
///
/// Both halves are here because only the pair pins it. `ZQ.` is named by
/// the enclosing body, so the plan holds it and the fragment's own writes
/// land on the plan's slot; `ZT.` is named by no clause of the body at
/// all, so nothing but the activation's run-time bindings can reach it.
/// Measured on an interpreter instrumented to print which of `slot_of`'s
/// three sources answered: `ZT.` grows into `extra` once, when the
/// fragment's plan is built, and is found there afterwards, on both
/// engines.
///
/// The oracle prints `4` for each of the three: a controlled `DO` writes
/// the value that failed its `TO` test, and a bare stem write makes that
/// value the stem's default, which is what an unwritten tail then answers.
///
/// **This passes unchanged before the slot was taken, and that is what it
/// is for.** A fragment's `Code::plan` is `None` on both sides, so neither
/// side has a slot to get wrong. It guards the design that was **not**
/// shipped -- routing a fragment's own translated slot, which lives in the
/// activation's `extra` rather than in any plan, into a stem write -- and
/// under that design this program is the one the whole workspace suite
/// otherwise had no case for.
#[test]
fn a_stem_control_inside_a_fragment_writes_the_enclosing_frames_slot() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"interpret \"do zq. = 1 to 3; nop; end\"\nsay zq.\n\
              interpret \"do zt. = 1 to 3; nop; end\"\n\
              interpret \"say zt.\"\ninterpret \"say zt.9\"\n"
        ),
        b"4\n4\n4\n".to_vec()
    );
}

/// `LEAVE`/`ITERATE` naming a compound control variable is unaffected
/// by this fix: the name match that selects which loop to unwind is
/// against the control's own spelling, decided independently of
/// whether that spelling's tail is ever resolved. Measured against the
/// oracle for both instructions.
#[test]
fn leave_and_iterate_by_name_still_reach_a_compound_controlled_loop() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"c = 0\nj = 1\ndo i.j = 0 to 6\nc = c + 1\nif c = 2 then leave i.j\nend i.j\nsay c\nsay i.1"
        ),
        b"2\n1\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"c = 0\nj = 1\ndo i.j = 0 to 6\nc = c + 1\nif c = 2 then iterate i.j\nend i.j\nsay c\nsay i.1"
        ),
        b"7\n7\n".to_vec()
    );
}

/// A compound control variable traces its own `>C>` before the setup's
/// `>=>` and again before the re-tested pass's `>V>`, exactly the order
/// `eval_node`'s own `Compound` arm uses for an ordinary read -- the
/// same order this fix's `bind_control` and its read-back both reuse
/// rather than reimplement.
#[test]
fn a_compound_control_variable_traces_its_own_c_line() {
    let mut interp = Interp::new();
    let program =
        parse_program(b"j = 1\ndo cv.j = 1 to 2\nnop\nend".to_vec()).expect("test program parses");
    let program = activate(&mut interp, program);
    interp.set_trace_mode(mode_from_setting(b"i").expect("I is a valid setting"));
    run_activated(&mut interp, &program).expect("test program runs");
    let trace = String::from_utf8_lossy(&interp.trace);
    assert!(
        trace.contains(">C>   CV.J => \"CV.1\"") && trace.contains(">C>     CV.J => \"CV.1\""),
        "expected a >C> line at both the setup indent and the re-tested-pass indent, \
         resolving CV.J to CV.1; trace was:\n{trace}"
    );
}

/// `DO name OVER expr` on a **non-stem** target: a string and a number
/// each iterate exactly once, yielding themselves -- Deviation 1 keeps
/// a stem target out of scope, and this test uses none.
#[test]
fn do_over_a_non_stem_target_iterates_once_yielding_itself() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do x over 'hello'\nsay x\nend"),
        b"hello\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do x over 42\nsay x\nend"),
        b"42\n".to_vec()
    );
}

/// `DO OVER ... FOR` on a non-stem target: `FOR 0` skips the one
/// iteration entirely; any `FOR` at least `1` still runs it exactly
/// once (there is only ever one item). Not independently measured
/// against the oracle (no oracle transcript pins `OVER ... FOR` on a
/// non-stem target specifically); implemented as the direct, minimal
/// extension of `FOR`'s own general "caps the iteration count" rule,
/// and named as a judgement call in the report.
#[test]
fn do_over_for_0_skips_the_single_non_stem_iteration() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do x over 'hello' for 0\nsay x\nend\nsay 'after'"
        ),
        b"after\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do x over 'hello' for 5\nsay x\nend"),
        b"hello\n".to_vec()
    );
}

/// **The oracle's own bytes for `DO OVER ... FOR`'s count**, captured
/// 2026-08-14 under both `TRACE I` and `TRACE R`, from a fresh oracle run
/// wrapped exactly as `rust/CLAUDE.md` specifies. The oracle prints
/// `>K>   "FOR" => "1"` for the count, and
/// `HeaderRole::OverFor::keyword()` is what decides whether this crate
/// emits it.
///
/// **What this pins is that the bytes are the oracle's**, which is the
/// whole of its job.
#[test]
fn a_do_over_for_echoes_the_for_keyword_the_oracle_prints() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace i\nzs = 'abc'\ndo qq over zs for 1\n  leave\nend\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* zs = 'abc'\n",
            "       >L>   \"abc\"\n",
            "       >>>   \"abc\"\n",
            "       >=>   ZS <= \"abc\"\n",
            "     3 *-* do qq over zs for 1\n",
            "       >V>   ZS => \"abc\"\n",
            "       >K>   \"OVER\" => \"abc\"\n",
            "       >L>   \"1\"\n",
            "       >K>   \"FOR\" => \"1\"\n",
            "       >=>     QQ <= \"abc\"\n",
            "     4 *-*   leave\n",
        )
    );

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace r\nzs = 'abc'\ndo qq over zs for 1\n  leave\nend\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* zs = 'abc'\n",
            "       >>>   \"abc\"\n",
            "     3 *-* do qq over zs for 1\n",
            "       >K>   \"OVER\" => \"abc\"\n",
            "       >K>   \"FOR\" => \"1\"\n",
            "     4 *-*   leave\n",
        )
    );
}

/// Deviation 1 (`phase-4-exclusions.txt`): `DO OVER` on a stem does not
/// reproduce the oracle's traversal order, and takes the loud path
/// instead -- detected from `target`'s own syntax (a bare `NAME.`
/// parses as `ExprKind::Stem`), never by evaluating it.
#[test]
fn do_over_a_stem_target_takes_the_loud_path() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"a.1 = 'x'\ndo v over a.\nsay v\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Same mutation-kill as `do_with_takes_the_loud_path` (above, in this
    // module): `DO`/`LOOP` is implemented regardless of which deviation
    // routed this particular clause to the loud path.
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

/// A stem target wrapped in parens is **also** caught -- corrected after
/// review, which found this task's own comment on the `Over` arm
/// claimed the opposite (`over (a.)` "is not detected"). It is detected:
/// a single parenthesised sub-expression collapses to that
/// sub-expression's own `ExprKind` rather than wrapping it in
/// `ExprKind::List`, so `(a.)` is already `ExprKind::Stem` by the time
/// `loop_header_plan`'s own `matches!` check sees it, with nothing extra
/// needed. The safe direction either way (loud, never a silent
/// divergence), but the comment was wrong about which one it is.
#[test]
fn do_over_a_parenthesised_stem_target_is_also_caught() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"a.1 = 'x'\ndo v over (a.)\nsay v\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Same mutation-kill as `do_with_takes_the_loud_path` (above, in this
    // module).
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

/// F1 (branch review, Important): `initial`/`to`/`by` are rounded under
/// the *entry* `NUMERIC DIGITS`, once, not stored as their exact parse
/// -- `round_via_unary_plus`'s own doc comment has the oracle citation
/// and both transcripts this mirrors exactly. Masked while `DIGITS`
/// stays constant (every later use re-rounds to the same width, so the
/// exact and the rounded value render identically); this widens
/// `DIGITS` *inside* the loop body, after entry, so only a value
/// rounded at entry -- not the exact parse -- can still show `1.23` on
/// every pass. Mutation killed: removing either `round_via_unary_plus`
/// call in `setup_controlled` (the one on `current`, or the one in
/// `ControlExpr::By`'s own arm) makes the affected probe's second and
/// third lines read the exact, unrounded value instead (`2.23456`/
/// `3.23456` for `current`, or accumulating by the exact `1.2345`
/// instead of the rounded `1.23` for `by`) -- verified by reverting
/// each in turn.
#[test]
fn a_controlled_loops_header_values_round_at_entry_not_the_exact_parse() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 3\ndo i = 1.23456 to 3\nsay i\nnumeric digits 9\nend"
        ),
        b"1.23\n2.23\n".to_vec(),
        "measured against the oracle: current is rounded once at entry \
         (1.23456 -> 1.23), and widening digits inside the loop must \
         not un-round it -- the exact parse would give 2.23456"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 3\ndo i = 1 to 9 by 1.2345\nsay i\nnumeric digits 9\nend"
        ),
        b"1\n2.23\n3.46\n4.69\n5.92\n7.15\n8.38\n".to_vec(),
        "measured against the oracle: by is rounded to 1.23 once at \
         entry and accumulated at that width, not the exact 1.2345 -- \
         the loop stops after 8.38 because the next value, 9.61, is \
         past the bound"
    );
}

// ---- DO/LOOP header errors (Step 2's own table, re-measured) ----

#[test]
fn a_non_numeric_control_value_raises_41_1() {
    for (source, found) in [
        (&b"do i = 'a' to 3\nnop\nend"[..], "a"),
        (&b"do i = 1 to 'x'\nnop\nend"[..], "x"),
        (&b"do i = 1 by 'x'\nnop\nend"[..], "x"),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (41, 1), "{source:?}");
        assert_eq!(
            raised.additional,
            vec![found.as_bytes().to_vec()],
            "{source:?}"
        );
    }
}

#[test]
fn a_bad_for_expression_raises_26_3() {
    for (source, found) in [
        (&b"do i = 1 to 3 for 'x'\nnop\nend"[..], "x"),
        (&b"do i = 1 to 3 for -1\nnop\nend"[..], "-1"),
        (&b"do i = 1 to 3 for 1.5\nnop\nend"[..], "1.5"),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 3), "{source:?}");
        assert_eq!(
            raised.additional,
            vec![found.as_bytes().to_vec()],
            "{source:?}"
        );
    }
}

#[test]
fn a_bad_repetition_count_raises_26_2() {
    for (source, found) in [
        (&b"do 'a'\nnop\nend"[..], "a"),
        (&b"do -1\nnop\nend"[..], "-1"),
        (&b"do 2.5\nnop\nend"[..], "2.5"),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 2), "{source:?}");
        assert_eq!(
            raised.additional,
            vec![found.as_bytes().to_vec()],
            "{source:?}"
        );
    }
}

/// F3 (branch review, Important): a bare `DO` count and a `FOR` count
/// are validated under the *current* `NUMERIC DIGITS`, not a fixed
/// width -- `whole_nonneg`'s own doc comment has the oracle citation
/// and the two oracle-measured transcripts this mirrors
/// (`numeric digits 3; do 12345; end` is 26.2 rc 230; the `FOR` shape
/// is 26.3). Mutation killed: reverting `whole_nonneg` to convert
/// under `rexx_num::ARGUMENT_DIGITS` (18) instead of the activation's
/// own `settings.digits()` makes both of these run clean (`after`
/// prints, no error) instead of raising, since `12345`/`54321` both
/// fit comfortably under 18 digits and only fail to fit under 3.
#[test]
fn a_repetition_or_for_count_is_validated_under_the_current_digits_not_a_fixed_width() {
    let mut interp = Interp::new();
    let failure =
        run_source(&mut interp, b"numeric digits 3\ndo 12345\nend\nsay 'after'").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (26, 2));

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"numeric digits 3\ndo i = 1 to 99999 for 12345\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (26, 3));
}

// ---- WHILE/UNTIL ----

#[test]
fn do_while_tests_the_condition_before_the_body_and_do_until_after() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"i = 0\ndo while i < 2\ni = i + 1\nsay i\nend"),
        b"1\n2\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 5\ndo until i >= 7\ni = i + 1\nsay i\nend"
        ),
        b"6\n7\n".to_vec(),
        "measured against the oracle, do_loop_forms.rex's own transcript"
    );
}

#[test]
fn a_while_condition_that_is_not_0_or_1_raises_34_3_not_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 3));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

#[test]
fn an_until_condition_that_is_not_0_or_1_raises_34_4() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 4));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// A comma-list `WHILE`/`UNTIL` condition is 34.6, never 34.3/34.4 --
/// `eval_condition`'s own rule, reused unchanged from `IF`/`WHEN`.
#[test]
fn a_comma_list_while_condition_raises_34_6_not_34_3() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do while 'x', 1\nnop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 6));
}

/// **The discriminating measurement for this whole section**: `UNTIL`
/// is tested at the *bottom* of the loop, so its own failure is
/// attributed to the `END`'s own clause, not the `DO`'s -- a loop that
/// evaluated `UNTIL` eagerly, at the top like `WHILE`, would still
/// produce the same raised number and the same exit code, and only the
/// echoed clause reveals the difference. Measured against the oracle:
/// `do until 'x' / end` echoes `end`, `do while 'x' / end` echoes the
/// `do` line.
#[test]
fn until_is_attributed_to_the_end_clause_while_while_is_attributed_to_the_do_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 3, "the END's own line");
    assert_eq!(text, b"end".to_vec(), "the END's own clause, not the DO's");

    let mut interp = Interp::new();
    run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 1, "the DO's own line");
    assert_eq!(text, b"do while 'x'".to_vec(), "the DO's own clause");
}

/// **`ITERATE` jumps to the loop's own bottom-of-iteration bookkeeping,
/// not to the top of the next pass** -- measured against the oracle,
/// this exact program terminates immediately with `n` still `1`, rather
/// than looping forever. A design that instead skipped `UNTIL` for the
/// interrupted iteration would hang on this program (`n` would keep
/// advancing past `1` and `UNTIL n = 1` would never hold again), which
/// is the mutation this test is built to catch -- not run, for the
/// obvious reason a hang cannot be asserted against, but predicted and
/// confirmed absent by construction: `do_body_outcome`'s own `Iterate`
/// arm answers `Ok(None)`, the *same* value `Flow::Next` answers, so
/// `run_repeating`'s own `UNTIL` check below it runs on both paths
/// identically.
#[test]
fn iterate_reaches_untils_own_bottom_of_iteration_test_rather_than_skipping_it() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\ndo until n = 1\nn = n + 1\nif n = 1 then iterate\nsay 'unreached'\nend\nsay 'done' n"
        ),
        b"done 1\n".to_vec()
    );
}

// ---- LEAVE/ITERATE: the block-stack rules ----

#[test]
fn bare_leave_stops_the_innermost_loop() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 5\nif i = 3 then leave\nsay i\nend"
        ),
        b"1\n2\n".to_vec()
    );
}

#[test]
fn bare_iterate_skips_the_rest_of_the_current_pass() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 4\nif i = 2 then iterate\nsay i\nend"
        ),
        b"1\n3\n4\n".to_vec()
    );
}

/// A bare `LEAVE`/`ITERATE` skips **transparently past** an unlabelled
/// `DO` block on its way to the nearest enclosing loop -- measured
/// against the oracle: this program prints `1` then `after`, not
/// `unreached`, because the innermost `DO` (a block, not a loop) never
/// intercepts the bare `LEAVE` at all.
#[test]
fn bare_leave_passes_transparently_through_an_unlabelled_simple_block() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 3\ndo\nsay i\nleave\nsay 'unreached'\nend\nend\nsay 'after'"
        ),
        b"1\nafter\n".to_vec()
    );
}

/// **Bare `LEAVE` in a simple `DO` block is 28.1** (not a loop, and
/// unlabelled, so nothing on the enclosing chain ever matches) -- but a
/// **labelled** simple block is leavable by its own explicit name,
/// which the next test pins.
#[test]
fn bare_leave_in_an_unlabelled_simple_block_with_nothing_enclosing_it_raises_28_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do\nleave\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 1));
}

#[test]
fn a_labelled_simple_block_is_leavable_by_its_own_name() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do label blk\nsay 'a'\nleave blk\nsay 'b'\nend\nsay 'after'"
        ),
        b"a\nafter\n".to_vec()
    );
}

/// **An ordinary clause label does not name a loop or a block.**
/// `outer:` here is a plain `Label` instruction, entirely separate from
/// the `DO`'s own `label` field (which is `i`, the control variable,
/// since no `LABEL` keyword was written) -- measured, `leave outer` is
/// 28.3, not a hit, exactly as `leave_nested_outer.rex`'s own comment
/// states.
#[test]
fn an_ordinary_clause_label_does_not_name_the_loop_it_precedes() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"outer: do i = 1 to 3\nleave outer\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 3));
    assert_eq!(raised.additional, vec![b"OUTER".to_vec()]);

    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"outer: do i = 1 to 3\niterate outer\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 4));
    assert_eq!(raised.additional, vec![b"OUTER".to_vec()]);
}

/// **What does name a loop for `LEAVE`/`ITERATE`**: a controlled loop's
/// own control variable, as an automatic label, with no `LABEL` keyword
/// needed at all. `leave i`/`iterate i` from inside a *nested* loop
/// reaches the outer one and unwinds the inner one on the way --
/// `leave_nested_outer.rex`'s own transcript, reproduced here.
#[test]
fn a_controlled_loops_own_control_variable_is_an_automatic_label() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then leave outer\nsay 'inner' outer inner\nend\nsay 'after inner' outer\nend\nsay 'after outer'"
        ),
        b"inner 1 1\nafter outer\n".to_vec()
    );
}

#[test]
fn iterate_naming_the_outer_loops_control_variable_cuts_every_outer_pass_short() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then iterate outer\nsay 'l' outer inner\nend\nsay 'outer-after' outer\nend"
        ),
        b"l 1 1\nl 2 1\nl 3 1\n".to_vec(),
        "outer-after never prints for any pass, since every one is cut short at inner = 2"
    );
}

/// **`leave sel` exits a `SELECT` only when it was written
/// `SELECT LABEL sel`.** An ordinary clause label in front of a
/// `SELECT` does not make it leavable (28.3, exactly like a loop).
#[test]
fn leave_by_name_exits_a_select_only_when_it_was_given_that_label() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"s: select label s\nwhen 1 = 1 then\ndo\nleave s\nend\notherwise\nnop\nend\nsay 'after'"
        ),
        b"after\n".to_vec()
    );

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select\nwhen 1 = 1 then\ndo\nleave sel\nend\notherwise\nnop\nend",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 3));
    assert_eq!(raised.additional, vec![b"SEL".to_vec()]);
}

/// A named `LEAVE` reaching a `SELECT LABEL` from inside its
/// `OTHERWISE` branch -- the fix that routes `OTHERWISE`'s own body
/// through `Select`'s own `run_bounded`/`leave_select`, not the plain
/// `Goto` it used before Task 11 (`Select`'s own arm has the full
/// argument). Kills a version that still lets `OTHERWISE` fall through
/// to the outer loop directly: that version would propagate this
/// `LEAVE` all the way to `run_activation`'s own top level and raise
/// 28.3 instead of the clean exit this test asserts.
#[test]
fn leave_by_name_reaches_a_select_label_from_inside_its_otherwise_branch() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select label s\nwhen 1 = 0 then nop\notherwise\nsay 'o'\nleave s\nsay 'unreached'\nend\nsay 'after'"
        ),
        b"o\nafter\n".to_vec()
    );
}

/// **`ITERATE` never accepts a labelled block or a labelled `SELECT`,
/// only a loop -- 28.5, not silently skipped, when the name matches but
/// the kind is wrong.**
#[test]
fn a_named_iterate_matching_a_labelled_simple_block_raises_28_5_not_28_4() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do label x\nsay 1\niterate x\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 5));
    assert_eq!(raised.additional, vec![b"X".to_vec()]);
}

#[test]
fn a_named_iterate_matching_a_select_label_raises_28_5() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select label s\nwhen 1 = 1 then\ndo\niterate s\nend\notherwise\nnop\nend",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 5));
    assert_eq!(raised.additional, vec![b"S".to_vec()]);
}

#[test]
fn iterate_from_inside_a_select_nested_in_a_loop_skips_only_the_current_pass() {
    // iterate_from_select.rex's own transcript: an ITERATE inside a
    // SELECT's WHEN body must skip straight to the loop's own next
    // pass, past both the SELECT's own resume point and back up to the
    // enclosing DO, printing exactly one "skip"/"keep" pair per
    // iteration and never both for the same i.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 4\nselect\nwhen i = 2 then do\nsay 'skip' i\niterate\nend\notherwise nop\nend\nsay 'keep' i\nend"
        ),
        b"keep 1\nskip 2\nkeep 3\nkeep 4\n".to_vec()
    );
}

/// Two real, unnamed loops nested two deep, neither matching `zz`: both
/// own a search frame (`is_loop` true for a `Controlled` loop), so both
/// get popped and both reset the residual to their own `static_indent`
/// -- the outer one last, to `0` (top level), which is what survives to
/// the exhausted-search report. This is **not** "28.1-28.4 always
/// report zero" (that rule was wrong -- see `n1`/`n2`/`n3` below, and
/// `LeaveOrigin`'s own doc comment for the corrected one): it is zero
/// here specifically because the outermost popped frame happens to sit
/// at top level, the same way it did in every one of this task's own
/// original probes, which is exactly how the wrong rule looked right
/// for as long as it did.
#[test]
fn leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"do i = 1 to 3\ndo j = 1 to 3\nleave zz\nend\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 0);
}

/// The one intervening construct is an *unlabelled* `Simple` block,
/// which owns no search frame and is fully transparent -- so nothing
/// ever resets the residual, and it stays at the `ITERATE`'s own full
/// lexical depth all the way to the match (the outer labelled block,
/// which does not reset on a match either). Two real loops in
/// `leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent`
/// above land on the *same* final answer as this test for an entirely
/// different reason -- neither test tells the two rules apart, which is
/// this task's own original mistake and why the corrected rule below has
/// its own dedicated tests.
#[test]
fn iterate_wrong_kind_through_a_transparent_unlabelled_block_reports_full_lexical_depth() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do label x\ndo\niterate x\nend\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4, "two DO frames deep, matching the oracle");
}

/// **The corrected 28.x indent rule, all fourteen points**: nine from
/// the reviewer's own probe (`p1`/`p2`/`p5`/`p8`/`p9`/`p10`/`p11`/`p12`/
/// `p13`, their own naming, kept so the report's cross-reference to
/// this table still resolves) plus five re-measured independently by
/// this task against the oracle before touching any code (`n1`-`n3`
/// entirely new shapes, `p1`/`p11` re-run to confirm the two that
/// already matched still do). Every row was captured with `cat -A`
/// against `build/bin/rexx`, byte for byte, not inferred.
///
/// **The rule the whole table obeys**: start at the `LEAVE`/`ITERATE`'s
/// own full lexical depth; walk outward; every `SELECT` (always) or
/// `DO`/`LOOP` (unless an unlabelled `Simple` block) that is examined
/// and does *not* match resets the residual to *its own*
/// `static_indent`; a match stops the walk without resetting anything
/// itself; report whatever the residual is at that point. `p11`/`p1`
/// are the two rows where the very first frame examined is the match,
/// so nothing ever resets and the reported value is the origin's own
/// unmodified full depth -- the case the original, wrong rule
/// generalised from.
#[test]
fn the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes() {
    for (name, source, expect) in [
        ("p5", &b"if 1=1 then leave"[..], 4),
        ("p9", &b"if 1=1 then do\nleave\nend"[..], 6),
        ("p12", &b"do\nleave\nend"[..], 2),
        ("p8", &b"if 1=1 then do i=1 to 3\nleave zz\nend"[..], 4),
        (
            "p2",
            &b"do label x\nselect\nwhen 1=1 then iterate x\notherwise nop\nend\nend"[..],
            2,
        ),
        (
            "p10",
            &b"do label x\nselect\nwhen 1=1 then do\niterate x\nend\notherwise nop\nend\nend"[..],
            2,
        ),
        (
            "p13",
            &b"do label x\ndo label y\niterate x\nend\nend"[..],
            2,
        ),
        (
            "p1",
            &b"do label x\nif 1=1 then do\niterate x\nend\nend"[..],
            8,
        ),
        (
            "p11",
            &b"select label s\nwhen 1=1 then iterate s\notherwise nop\nend"[..],
            6,
        ),
        // Independently added by this task, not in the reviewer's own
        // table: a bare LEAVE reaching only an unlabelled SELECT
        // (SELECT owns a frame unconditionally, even unlabelled, so
        // the pop still happens and still resets to its own indent).
        (
            "n1",
            &b"select\nwhen 1=1 then leave\notherwise nop\nend"[..],
            0,
        ),
        // Three real loops nested three deep, none matching: each pop
        // resets in turn, the outermost (top level) wins.
        (
            "n2",
            &b"do i=1 to 3\ndo j=1 to 3\ndo k=1 to 3\nleave zz\nend\nend\nend"[..],
            0,
        ),
        // A named ITERATE crossing one unlabelled real loop and one
        // unlabelled SELECT, matching neither: both pop, the outer
        // loop's own indent (0, top level) is what survives.
        (
            "n3",
            &b"do i=1 to 3\nselect\nwhen 1=1 then iterate zz\notherwise nop\nend\nend"[..],
            0,
        ),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).unwrap_err();
        let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
        else {
            panic!("not a clause site")
        };
        assert_eq!(indent, expect, "{name}: {source:?}");
    }
}

/// **The `run_bounded` `Goto`-absorption trap, named in `Flow::Leave`'s
/// own doc comment.** A `DO` with an `ITERATE` in its body, nested
/// inside an `IF`'s `THEN`, run enough times that a version which
/// implemented repetition as a `Goto` back to the loop's own top
/// (rather than an internal `run_repeating` loop that never returns
/// mid-construct) would have that `Goto`'s target absorbed by the
/// enclosing `IF`'s own `run_bounded` -- which owns the whole `THEN`
/// range the `DO` sits inside -- and re-enter the `DO`'s own arm as a
/// fresh first pass, with its running total reset to `Simple`/`Count`'s
/// own initial state every time. That bug's own observable signature
/// is `n` staying stuck at whatever the first `ITERATE`d pass produced,
/// forever, or (depending on exactly how the re-entry is shaped)
/// hanging outright, rather than the small, exact total this test
/// asserts.
#[test]
fn leave_and_iterate_survive_a_do_nested_in_an_ifs_then_iterating_repeatedly() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\nif 1 = 1 then do i = 1 to 5\nif i // 2 = 0 then iterate\nn = n + i\nend\nsay n"
        ),
        b"9\n".to_vec(),
        "1 + 3 + 5, the odd i's only -- a Goto-based re-entry would not \
         accumulate this total across the DO's own five iterations"
    );
}

// ---- the LEAVE/ITERATE search stops at the run_fragment boundary ----

/// The measured rule, all four families at once: a `LEAVE`/`ITERATE`
/// inside `INTERPRET` text never sees the enclosing loop, so an
/// enclosing `DO` that would have consumed it does not, and it is the
/// exhausted search instead. `run_fragment`'s own doc comment has the
/// oracle transcripts these numbers come from.
///
/// **The bare rows are the ones that decide the design**, and until
/// this was measured the code did the opposite: a bare `Flow::Leave`
/// forwarded out of the fragment and the enclosing `DO` swallowed it,
/// which is what "the fragment runs inside the enclosing activation"
/// predicts and what the oracle does not do. Mutation-kill: restore
/// `Ok(flow)` for the `None` arms in `run_fragment` and the two bare
/// rows here run to completion with no error at all.
#[test]
fn a_fragments_leave_or_iterate_never_reaches_the_enclosing_loop() {
    for (source, number, sub, additional) in [
        (
            &b"do kk = 1 to 3\ninterpret \"leave\"\nend\n"[..],
            28u16,
            1u16,
            Vec::new(),
        ),
        (
            &b"do kk = 1 to 3\ninterpret \"iterate\"\nend\n"[..],
            28,
            2,
            Vec::new(),
        ),
        (
            &b"do label outer while 1\ninterpret \"leave outer\"\nend\n"[..],
            28,
            3,
            vec![b"OUTER".to_vec()],
        ),
        (
            &b"do label outer kk = 1 to 3\ninterpret \"iterate outer\"\nend\n"[..],
            28,
            4,
            vec![b"OUTER".to_vec()],
        ),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised for {source:?}, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (number, sub), "{source:?}");
        assert_eq!(raised.additional, additional, "{source:?}");
    }
}

/// The name in 28.3/28.4 is resolved against the **fragment's** symbol
/// table, which is the half of F-EX2 that survives the rule above.
///
/// `run_fragment` gives `"leave foo"` its own fresh `SymbolTable`, so
/// `foo` interns at id 0 there regardless of what the enclosing
/// program's own table looks like. This program's own table also has
/// exactly one symbol -- `BAR`, also id 0, from the assignment on the
/// first line -- chosen deliberately so the two tables collide on the
/// same id with *different* names, which is what makes resolving
/// against the wrong table give a wrong answer rather than a panic.
/// Mutation-kill: resolve through the enclosing `code.symbols` instead
/// and 28.3 names `"BAR"`, the enclosing program's own symbol 0, not the
/// one `leave` actually named.
#[test]
fn a_fragments_named_leave_is_resolved_against_the_fragments_own_table() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"bar = 1\ninterpret \"leave foo\"\n").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 3));
    assert_eq!(raised.additional, vec![b"FOO".to_vec()]);
}

/// Review finding I1(a): `INTERPRET` traces `>>>` on the text it is about
/// to run, like every other value-producing arm, and it shipped without
/// doing so.
///
/// Oracle, verbatim, for the first program (rc 0, empty stdout):
///
/// ```text
///      1 *-* trace r
///      2 *-* zz = 'nop'
///        >>>   "nop"
///      3 *-* interpret zz
///        >>>   "nop"
///      3 *-* nop
/// ```
///
/// The whole transcript
/// is compared byte for byte below, including that the fragment's clause
/// echoes as line **3** -- the enclosing `INTERPRET`'s line, not the
/// fragment's own line 1, which is what `Interp::clause_line_override`
/// exists for and what a naive `Some(&fragment.source)` gets wrong.
/// (`say_output` drives `trace_mode` directly instead of running a `trace
/// r` clause, so the program below is the oracle's minus its first line
/// and every line number is one lower.)
///
/// The second program pins the **indent**, which is the part a wrong fix
/// would still get wrong: one `DO` deeper, the oracle's `>>>` picks up
/// that construct's own two spaces, and it does so because the arm reads
/// `current_value_indent` rather than recomputing anything. Mutation
/// killed both ways: dropping the `trace_result` call empties the `>>>`
/// lines, and moving it after `run_fragment` reports the *fragment's* last
/// indent instead of this clause's.
///
/// It now pins the fragment echo's indent too, which is the **delta-0**
/// measurement: the oracle prints `     3 *-*   nop` -- the enclosing
/// clause's own two spaces and no more. An implementation that gave the
/// fragment a level's worth of extra indent (the two spaces a *called
/// routine* really does get, measured) prints four here and fails.
#[test]
fn interpret_traces_the_text_it_is_about_to_run() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"zz = 'nop'\ninterpret zz");
    assert_eq!(
        interp.trace,
        concat!(
            "     1 *-* zz = 'nop'\n",
            "       >>>   \"nop\"\n",
            "     2 *-* interpret zz\n",
            "       >>>   \"nop\"\n",
            "     2 *-* nop\n",
        )
        .as_bytes()
    );

    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"do kk = 1 to 1\ninterpret \"nop\"\nend");
    for expected in [&b"       >>>     \"nop\"\n"[..], &b"     2 *-*   nop\n"[..]] {
        assert!(
            interp
                .trace
                .windows(expected.len())
                .any(|window| window == expected),
            "expected a line carrying the enclosing DO's own two spaces \
             ({:?}), got {:?}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&interp.trace)
        );
    }
}

/// The other side of the boundary rule: a loop written *inside* the
/// fragment consumes its own `LEAVE` normally, so the rule above is a
/// statement about crossing the boundary and not a blanket refusal.
/// Measured: this program prints two lines and exits 0.
#[test]
fn a_leave_inside_the_fragments_own_loop_is_consumed_there() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"interpret \"do jj = 1 to 5; say 'frag' jj; if jj = 2 then leave; end\"\nsay 'after'\n"
        ),
        b"frag 1\nfrag 2\nafter\n".to_vec()
    );
}

// ---- Task 11's own indentation quantity ----

#[test]
fn one_two_and_three_enclosing_dos_indent_by_two_four_and_six() {
    for (source, spaces) in [
        (&b"do i = 1 to 3\nsay 1/0\nend"[..], 2),
        (&b"do i = 1 to 3\ndo j = 1 to 3\nsay 1/0\nend\nend"[..], 4),
        (
            &b"do i = 1 to 3\ndo j = 1 to 3\ndo k = 1 to 3\nsay 1/0\nend\nend\nend"[..],
            6,
        ),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).unwrap_err();
        let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
        else {
            panic!("not a clause site")
        };
        assert_eq!(indent, spaces, "{source:?}");
    }
}

/// **The test the coordinator's own design review asked for by name**:
/// a raise *after* a loop has already exited, at a shallower lexical
/// depth than the loop's own body -- exactly the shape a live,
/// imperfectly-unwound `Interp` counter would over-indent and a purely
/// static function of `(instructions, index)` cannot, because there is
/// no state left over from the loop's own three completed iterations
/// for anything to have failed to unwind.
#[test]
fn the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do i = 1 to 3\nsay i\nend\nsay 1/0").unwrap_err();
    let FailureSite::Clause { indent, text, .. } =
        interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        indent, 0,
        "top level, after the loop, not the loop's own two"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

/// `DO`'s own control-setup expressions (`TO`/`BY`/`FOR`/the repeat
/// count/`OVER`'s target) are evaluated **before** the loop's own body
/// is entered, and are unindented even though the `DO` clause they
/// belong to sits at the same lexical position the body's own two
/// spaces would apply to -- the case the brief's own bullet 4 names
/// (`do i = 1 to 'x'` gets none), re-measured across the sibling forms
/// the brief did not enumerate.
#[test]
fn control_setup_expressions_are_unindented_unlike_the_loop_body_they_precede() {
    for source in [
        &b"do i = 1 to 'x'\nsay 1\nend"[..],
        &b"do 1/0\nsay 1\nend"[..],
        &b"do i = 1 to 3 for 1/0\nsay 1\nend"[..],
        &b"do i = 1 to 3 by 1/0\nsay 1\nend"[..],
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).unwrap_err();
        let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
        else {
            panic!("not a clause site")
        };
        assert_eq!(indent, 0, "{source:?}");
    }
}

/// `WHILE`/`UNTIL` are tested **inside** the loop's own frame, unlike
/// the header's control-setup expressions -- measured, `do while 1/0`
/// is indented two at top level, not zero.
#[test]
fn while_and_until_are_indented_inside_the_loops_own_frame() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do while 1/0\nsay 1\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 2);

    let mut interp = Interp::new();
    run_source(&mut interp, b"do until 1/0\nsay 1\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 2);
}

/// A `SELECT`'s own scan (evaluating each `WHEN`'s condition) is
/// indented two, present even before any `WHEN` matches -- the finding
/// this task made that the brief did not state (measured: `select /
/// when 1/0 then nop / end` indents the failing `WHEN`'s own condition
/// by two, not zero).
#[test]
fn a_whens_own_condition_is_indented_at_the_selects_own_two_spaces() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 1/0 then nop\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 2);
}

/// A matched `WHEN`'s own `THEN` body indents six (`SELECT`'s two, plus
/// the `WHEN`-`THEN` shape's own four, the same as an `IF`'s matched
/// branch); `OTHERWISE`'s own body indents only four, **not** six --
/// the second finding this task made that the brief did not state,
/// because `OTHERWISE` is not built from the same double-frame `IF`-
/// shaped machinery a `WHEN`'s `THEN` is.
#[test]
fn a_matched_whens_then_body_indents_six_but_otherwises_body_indents_only_four() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 1 = 1 then say 1/0\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 6);

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\notherwise\nsay 1/0\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        indent, 4,
        "OTHERWISE's own body, one frame, not the WHEN-THEN shape's two"
    );
}

/// An `IF`'s matched branch is four spaces, for **both** `THEN` and a
/// plain `ELSE` (not only the "else if" chain shape the brief's own
/// example used) -- and an "else if" chain nests two such branches,
/// giving eight.
#[test]
fn an_ifs_matched_then_or_else_branch_indents_four_and_an_else_if_chain_indents_eight() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"if 1 = 1 then say 1/0").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4, "THEN");

    let mut interp = Interp::new();
    run_source(&mut interp, b"if 1 = 0 then say 2\nelse say 1/0").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4, "a plain ELSE, not part of an else-if chain");

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"if 1 = 0 then say 2\nelse if 1 = 1 then say 1/0",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        indent, 8,
        "the outer ELSE's own four, plus the inner IF's THEN's own four"
    );
}

/// Task 13's own four `static_indent` fixes, found while building
/// `TRACE`'s clause echo -- **not a second computation of the
/// indentation quantity**, a bug in the existing one, for a case
/// Task 10/11 structurally could not exercise: none of `THEN`/`ELSE`/
/// `OTHERWISE`/a `WHEN`'s own `THEN` carries an expression, so none of
/// them can ever raise a condition and become a `FailureSite` -- the
/// only way anything before this task ever asked `static_indent` a
/// question. `TRACE` echoes every stepped instruction, markers
/// included, which is what finally asks.
///
/// Each expected number is the oracle's, read with `cat -A` against
/// `build/bin/rexx` under `trace r` (the report has the full
/// transcripts): a marker clause sits at exactly half the indent its
/// own body gets, all the way down through nesting -- confirmed by
/// `ThenInstruction.cpp`/`ElseInstruction.cpp`'s own `execute`
/// (`indent(); trace; indent();`, so the marker traces after the first
/// bump and the body after the second) and by `OtherwiseInstruction.cpp`
/// (`trace; indent();`, one bump, so `OTHERWISE`'s own clause sits at
/// the `SELECT`'s scan level, the same as a `WHEN`'s own condition).
///
/// This calls `static_indent` directly rather than through a raise,
/// because a marker clause cannot raise -- there is no `FailureSite` to
/// read one back from.
/// `all_indents` fills what `static_indent` computes, for every position
/// of every program in the corpus.
///
/// The two are separate traversals of the same rules -- `static_indent`
/// walks the body once per position, `fill_indents` assigns every
/// position in one walk -- so the transcription can be wrong where the
/// original is right, and only running both over real programs shows it.
/// The corpus is the source rather than examples written here for the
/// usual reason: it grows when a construct lands, and a table of
/// hand-picked shapes does not.
///
/// Programs that do not parse are skipped, since the corpus holds
/// deliberate syntax errors and there is no instruction list to compare.
#[test]
fn all_indents_fills_what_static_indent_computes_for_every_corpus_program() {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let mut compared = 0usize;
    let mut positions = 0usize;
    let mut directories = vec![corpus];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rex") {
                continue;
            }
            let bytes = std::fs::read(&path).expect("a readable corpus program");
            let Ok(program) = parse_program(bytes) else {
                continue;
            };
            let instructions = &program.main.instructions;
            let filled = all_indents(instructions);
            let expected: Vec<usize> = (0..instructions.len())
                .map(|index| static_indent(instructions, index))
                .collect();
            assert_eq!(
                filled.as_ref(),
                expected.as_slice(),
                "{} disagrees",
                path.display()
            );
            compared += 1;
            positions += instructions.len();
        }
    }
    assert!(
        compared > 40 && positions > 500,
        "only {compared} programs and {positions} positions were compared, \
         which is too little of the corpus to have tested anything"
    );
}

#[test]
fn a_then_else_when_then_or_otherwise_markers_own_clause_indents_half_its_bodys() {
    // `IF`'s own `THEN`: `then_start` used to fall into the body's `+4`
    // branch (the branch's own recursive call returns 0 for the very
    // first position of its range) and answer 4, not 2.
    let instructions = instructions_of(b"if 1 = 1 then say 'x'");
    let then_start = if_then_start(&instructions, 0);
    assert_eq!(static_indent(&instructions, then_start), 2, "IF's own THEN");

    // `IF`'s own `ELSE`: `target == false_target` used to fall through
    // this whole arm entirely (`pc = else_end; continue`, which passes
    // straight over the marker's own index in the enclosing walk) and
    // answer 0, the enclosing level, not 2.
    let instructions = instructions_of(b"if 1 = 0 then say 'x'\nelse say 'y'");
    let if_index = 0;
    let InstructionKind::If { false_target, .. } = &instructions[if_index].kind else {
        panic!("index 0 is the IF")
    };
    let else_index = false_target.expect("this IF has an ELSE");
    assert_eq!(static_indent(&instructions, else_index), 2, "IF's own ELSE");

    // A `WHEN`'s own `THEN`, sharing `InstructionKind::Then` with `IF`
    // (`instruction.rs`'s `if_instruction` builds both): `target ==
    // body_start` used to match the loop's own `>=` and answer 6, the
    // body's value, not 4.
    let instructions = instructions_of(b"select\nwhen 1 = 1 then say 'x'\nend");
    let when_index = 1;
    let then_start = if_then_start(&instructions, when_index);
    assert_eq!(
        static_indent(&instructions, then_start),
        4,
        "WHEN's own THEN"
    );

    // `OTHERWISE`'s own clause -- **this one used to abort the process**,
    // not merely answer a wrong number: `target == *otherwise_index`
    // matched neither the `whens` loop nor the body check below it, and
    // fell to `unreachable!("a resolved SELECT's own range holds only
    // its WHENs and OTHERWISE")`. Reproduced against the tree before
    // this fix (`cargo test` aborted this exact test with that message,
    // `run.rs`'s panic site named in the fix's own commit) rather than
    // inferred.
    let instructions = instructions_of(b"select\nwhen 1 = 0 then nop\notherwise\nsay 'y'\nend");
    let otherwise_index = instructions
        .iter()
        .find_map(|i| match &i.kind {
            InstructionKind::Select { otherwise, .. } => *otherwise,
            _ => None,
        })
        .expect("this SELECT has an OTHERWISE");
    assert_eq!(
        static_indent(&instructions, otherwise_index),
        2,
        "OTHERWISE's own marker clause"
    );
}

/// `if_instruction` (`rexx-parse`'s `instruction.rs`) gives both `IF`
/// and a `WHEN`'s own `THEN` the identical shape: the `Then` marker
/// sits immediately after the condition-bearing instruction at
/// `condition_index`. A free function rather than inlined three times
/// in the test above, since all three call sites need the exact same
/// index arithmetic and nothing else about `Instruction` to find it.
fn if_then_start(instructions: &[Instruction], condition_index: usize) -> usize {
    assert!(
        matches!(
            instructions[condition_index].kind,
            InstructionKind::If { .. }
                | InstructionKind::When { .. }
                | InstructionKind::WhenCase { .. }
        ),
        "condition_index must be an IF, a WHEN or a WHEN CASE"
    );
    condition_index + 1
}

/// Parses `source` and returns its top-level instruction list, owned --
/// the marker-index tests above need to read `InstructionKind` fields
/// directly (`false_target`, `otherwise`) rather than only run the
/// program to a raise, since a marker clause cannot raise at all.
fn instructions_of(source: &[u8]) -> Vec<Instruction> {
    parse_program(source.to_vec())
        .expect("test program parses")
        .main
        .instructions
}

/// Nesting a `SELECT` (with a matched `WHEN`) inside a `DO`: two plus
/// two plus four -- confirming the additive model composes across
/// different construct kinds, not only same-kind nesting.
#[test]
fn a_select_nested_inside_a_do_composes_the_two_constructs_own_contributions() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"do i = 1 to 3\nselect\nwhen 1 = 1 then say 1/0\notherwise nop\nend\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 8);
}

// ---- End arm ----

#[test]
fn a_do_loops_own_end_is_a_pure_marker_never_independently_dispatched() {
    // Reached at all only if something jumped straight onto it, which
    // nothing in this crate does -- covered indirectly by every DO/LOOP
    // test above completing without the loud failure this arm used to
    // give before Task 11. Direct coverage: a labelled LOOP closes
    // cleanly (EndStyle::LabeledDo is exercised on the same code path
    // EndStyle::Do/Loop are).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do label x\nsay 'ok'\nend"),
        b"ok\n".to_vec()
    );
}

// ---- CALL and RETURN (Task 3) ----

/// D9r's default, and the one property a witness without variables in it
/// cannot check: a callee with no `PROCEDURE` reads the caller's
/// variables **and its writes survive the return**. An implementation
/// that gave every callee a fresh pool passes `call sub` / `sub: say
/// 'callee'` and fails this.
#[test]
fn a_routine_without_procedure_shares_the_callers_pool() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'caller-v'\ncall sub\nsay 'caller sees:' v w\nexit\n\
              sub:\nsay 'callee sees v:' v\nw = 'callee-w'\nreturn\n",
        ),
        b"callee sees v: caller-v\ncaller sees: caller-v callee-w\n".to_vec()
    );
}

/// The body selector's own reason to exist: the callee runs *its* clauses,
/// from the label, and the caller resumes after the `CALL` rather than at
/// the top.
#[test]
fn a_called_label_runs_its_own_clauses_not_the_main_body() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'main'\nexit\nsub: say 'callee'\nreturn\n"
        ),
        b"callee\nmain\n".to_vec()
    );
}

/// A name bound at run time inside the callee is part of the same shared
/// pool, which needs the *name* to cross the return as well as the slot
/// -- `Activation::extra`, cloned in and moved back out. Measured on the
/// oracle both ways round; this is the direction that fails if `extra`
/// is left behind with the callee.
#[test]
fn a_name_bound_by_interpret_inside_a_callee_survives_the_return() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\ninterpret \"say zork\"\nexit\nsub:\ninterpret \"zork = 42\"\nreturn\n",
        ),
        b"42\n".to_vec()
    );

    // And inward, which is the half that already worked: a name the
    // caller bound at run time is readable in the callee.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"interpret \"zork = 42\"\ncall sub\nexit\nsub:\ninterpret \"say zork\"\nreturn\n",
        ),
        b"42\n".to_vec()
    );
}

/// `RESULT` is settled on **return**, not at the call: the callee sees
/// the caller's own pre-call value, and only the return overwrites it.
/// A bare `return` drops it, so it reads back as its own derived name.
#[test]
fn result_is_settled_on_return_and_a_bare_return_drops_it() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"result = 'before'\ncall sub\nsay 'after result=' result\n\
              call bare\nsay 'after bare result=' result\nexit\n\
              sub:\nsay 'inside result=' result\nreturn 42\nbare:\nreturn\n",
        ),
        b"inside result= before\nafter result= 42\nafter bare result= RESULT\n".to_vec()
    );
}

/// The two ways out of a callee that are *not* a return, both measured:
/// an explicit `EXIT`, and the body simply running out of instructions.
/// Either ends the program, so the caller's next clause never runs --
/// which is why `Ended` distinguishes them from `Returned` at all.
#[test]
fn exiting_or_falling_off_the_end_inside_a_callee_ends_the_program() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'never'\nsub:\nsay 'in sub'\nexit\n"
        ),
        b"in sub\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'never'\nexit\nsub:\nsay 'in sub'\n"
        ),
        b"in sub\n".to_vec()
    );
}

/// A `RETURN` in the main body, with no active call: it ends the program
/// with its value, exactly like `EXIT`. Measured at rc 5.
#[test]
fn a_return_in_the_main_body_ends_the_program_with_its_value() {
    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"say 'a'\nreturn 5\nsay 'b'\n").expect("the program runs");
    assert_eq!(interp.out, b"a\n".to_vec());
    assert_eq!(interp.exit_code_for(value), 5);

    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"say 'a'\nreturn\nsay 'b'\n").expect("the program runs");
    assert_eq!(interp.out, b"a\n".to_vec());
    assert_eq!(interp.exit_code_for(value), 0);
}

/// `RETURN` unwinds to the **activation** boundary and past every block
/// frame in between -- which is exactly what `LEAVE` does not do. Both
/// enclosing constructs here would consume a `Flow::Leave`; neither may
/// consume a `Flow::Return`.
#[test]
fn a_return_escapes_every_enclosing_block_in_the_callee() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'back'\nexit\nsub:\ndo forever\nreturn\nend\n",
        ),
        b"back\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'back'\nexit\nsub:\nselect\nwhen 1 = 1 then return\notherwise nop\nend\n",
        ),
        b"back\n".to_vec()
    );
}

/// `NUMERIC` is inherited at call time and never written back -- measured
/// with `digits 7` outside and `digits 3` inside. The second `say` is the
/// discriminating one: an implementation that shared one `Settings`
/// would print `3` there.
#[test]
fn numeric_settings_are_inherited_by_a_callee_and_not_written_back() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 7\ncall sub\nsay 1/3\nexit\n\
              sub:\nsay 1/3\nnumeric digits 3\nsay 1/3\nreturn\n",
        ),
        b"0.3333333\n0.333\n0.3333333\n".to_vec()
    );
}

/// I4: `TRACE` moved onto `Activation`, so a callee's own `trace off`
/// dies with the callee. The caller's clauses echo again afterwards,
/// which is what a single `Interp`-wide field could not do.
#[test]
fn a_callees_trace_setting_does_not_survive_its_return() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"call sub\nsay 'after'\nexit\nsub:\ntrace off\nsay 'quiet'\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    4 *-*   sub:\n\
          \x20    5 *-*   trace off\n\
          \x20    2 *-* say 'after'\n\
          \x20      >>>   \"after\"\n\
          \x20    3 *-* exit\n"
            .to_vec(),
        "the callee's `trace off` must silence only the callee"
    );
}

/// **The two trace-line indents Task 9 added, pinned here because
/// nothing else in the tree can pin them** (review round 1, F1).
/// `tests/trace_oracle.rs`'s witnesses and `tests/corpus.rs` both
/// compare through DEVIATION 0's `normalize_stderr`, which collapses
/// exactly the space run these lines differ in -- measured, replacing
/// the `do_indent`/`loop_indent` split in `loop_advance` with
/// `loop_indent` alone, or emitting `>F>` two columns in, leaves both
/// harnesses green while diverging from the oracle byte for byte. A
/// unit test's own `assert_eq!` is outside either comparison function,
/// which is what makes this the instrument for the job -- the same
/// argument `phase-4-exclusions.txt`'s DEVIATION 0 already makes for
/// the pinned shallow-depth indent witnesses.
///
/// Every byte below is the oracle's, captured with `cat -A` from the
/// two programs run verbatim. They carry their own `TRACE I`
/// instruction rather than having a mode forced onto the activation, so
/// that the first clause is untraced on both sides and the two
/// transcripts are directly comparable.
#[test]
fn task_9s_two_new_indents_are_the_oracles_own_and_normalisation_cannot_see_them() {
    // A `Controlled` loop: the setup assignment prints at the `DO`
    // clause's own indent (3 blanks after `>=>`) and every re-tested
    // pass's four lines print two further in (5 blanks). Collapsing the
    // two to one number is what this fails on.
    let mut interp = Interp::new();
    run_source(&mut interp, b"trace i\ndo ii = 1 to 2\n  nop\nend\n").expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* do ii = 1 to 2\n",
            "       >L>   \"1\"\n",
            "       >L>   \"2\"\n",
            "       >K>   \"TO\" => \"2\"\n",
            "       >=>   II <= \"1\"\n",
            "     3 *-*   nop\n",
            "     4 *-* end\n",
            "     2 *-* do ii = 1 to 2\n",
            "       >V>     II => \"1\"\n",
            "       >>>     \"1\"\n",
            "       >>>     \"2\"\n",
            "       >=>     II <= \"2\"\n",
            "     3 *-*   nop\n",
            "     4 *-* end\n",
            "     2 *-* do ii = 1 to 2\n",
            "       >V>     II => \"2\"\n",
            "       >>>     \"2\"\n",
            "       >>>     \"3\"\n",
            "       >=>     II <= \"3\"\n",
        )
    );

    // `>F>`: the caller's own indent, where the callee's own `>>>` for
    // the same value sits two further in on the line above it.
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace i\nzz = twice(2)\nexit\ntwice:\nuse arg nn\nreturn nn + nn\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* zz = twice(2)\n",
            "       >L>   \"2\"\n",
            "       >A>   \"2\"\n",
            "     4 *-*   twice:\n",
            "     5 *-*   use arg nn\n",
            "       >>>     \"2\"\n",
            "       >=>     NN <= \"2\"\n",
            "     6 *-*   return nn + nn\n",
            "       >V>     NN => \"2\"\n",
            "       >V>     NN => \"2\"\n",
            "       >O>     \"+\" => \"4\"\n",
            "       >>>     \"4\"\n",
            "       >F>   TWICE => \"4\"\n",
            "       >>>   \"4\"\n",
            "       >=>   ZZ <= \"4\"\n",
            "     3 *-* exit\n",
        )
    );
}

/// D2r's rule, at **three** caller indents rather than one: a callee's
/// clauses echo at the calling clause's own printed indent plus two.
/// `2 x depth` agrees with all of this at caller indent 0 and predicts 2
/// where the truth is 4 and 6, which is why one shape is not enough.
/// Every byte here was captured from the oracle.
#[test]
fn a_callees_clauses_echo_at_the_calling_clauses_indent_plus_two() {
    // Flat caller, two levels deep: 0 -> 2 -> 4.
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"aa = 1\ncall one\nexit\none:\nbb = 2\ncall two\nreturn\ntwo:\ncc = 3\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* aa = 1\n\
          \x20      >>>   \"1\"\n\
          \x20    2 *-* call one\n\
          \x20    4 *-*   one:\n\
          \x20    5 *-*   bb = 2\n\
          \x20      >>>     \"2\"\n\
          \x20    6 *-*   call two\n\
          \x20    8 *-*     two:\n\
          \x20    9 *-*     cc = 3\n\
          \x20      >>>       \"3\"\n\
          \x20   10 *-*     return\n\
          \x20    7 *-*   return\n\
          \x20    3 *-* exit\n"
            .to_vec()
    );

    // Caller two `DO` blocks deep: the callee echoes at 6, where
    // `2 x depth` predicts 2.
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"do\ndo\ncall sub\nend\nend\nexit\nsub:\ndd = 4\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* do\n\
          \x20    2 *-*   do\n\
          \x20    3 *-*     call sub\n\
          \x20    7 *-*       sub:\n\
          \x20    8 *-*       dd = 4\n\
          \x20      >>>         \"4\"\n\
          \x20    9 *-*       return\n\
          \x20    4 *-*   end\n\
          \x20    5 *-* end\n\
          \x20    6 *-* exit\n"
            .to_vec()
    );
}

/// **Review finding C1, Task 4 fix round 1.** `current_value_indent` is
/// a fourth piece of level state `Interp::invoke_call` must restore on
/// the way out, alongside `activation_indent`/`indent_offset`/
/// `clause_line_override` (that function's own doc comment) -- and this
/// is the shape that tells a version missing the restore apart from a
/// correct one: **two** internal-function calls inside *one* clause
/// (`ExprKind::Call`, Task 4). Before Task 4 at most one activation
/// could be entered per clause, and the *next* clause's own
/// `step_in_temps_frame` re-set the field before anything read it, so
/// the gap was unobservable through `CALL` alone. Without the restore,
/// `g`'s own base indent -- and everything computed from it: its
/// clauses, its `RETURN`'s own value trace, and the enclosing `say`
/// clause's own final `>>>` -- is derived from `f`'s last clause instead
/// of the caller's own.
///
/// Byte-exact against the oracle in a clean directory (source measured
/// with a leading `trace r` clause enabling tracing, then every line
/// number decremented by one to match `run_source_traced`'s own
/// externally-set mode, which consumes no line of its own -- the same
/// transformation this file's other `run_source_traced` expectations
/// already rely on, checked here against `a_callees_clauses_echo_at_
/// the_calling_clauses_indent_plus_two`'s own source with a real
/// `trace r` prepended). This assertion compares raw `interp.trace`
/// bytes and is **not** reachable by `corpus.rs`'s `normalize_stderr`
/// (DEVIATION 0), which collapses exactly this class of indent
/// difference -- see `phase-4b.txt`'s own entry for `lang/
/// call_expression.rex` for why the corpus differential cannot be
/// trusted to catch this at all.
#[test]
fn current_value_indent_is_restored_after_a_nested_expression_call() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say f(1) + g(2)\nexit\nf: return 1\ng: return 2\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say f(1) + g(2)\n\
          \x20    3 *-*   f:\n\
          \x20    3 *-*   return 1\n\
          \x20      >>>     \"1\"\n\
          \x20    4 *-*   g:\n\
          \x20    4 *-*   return 2\n\
          \x20      >>>     \"2\"\n\
          \x20      >>>   \"3\"\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );
}

/// `current_value_indent`'s own sibling field, found the identical way
/// (Task 6 fix round 2): `current_clause_line` (bundled with it into
/// `ClauseState`, whose own doc comment states the property that puts
/// both fields in one save/restore rather than two) is a piece of level
/// state `Interp::invoke_call` must restore on the way out, and shipped
/// once already without that restore -- the second field of this exact
/// shape to do so, after `current_value_indent` itself went unrestored
/// until the test just above this one caught it at Task 4.
///
/// The same shape catches it: two internal-function calls inside *one*
/// clause. Without the restore, `g`'s own `SIGL` (`set_sigl`, reading
/// `current_clause_line`) reads `f`'s own last line (`return 1`, line 5)
/// instead of the calling clause's own (line 1) -- measured against the
/// oracle in a clean directory, `rexx-run` on this exact source once
/// read `sigl in g: 5` before this fix and `sigl in g: 1` after it, and
/// the oracle has always answered `1`.
#[test]
fn current_clause_line_is_restored_after_a_nested_expression_call() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say f(1) + g(2)\n\
              exit\n\
              f:\n\
              say 'in f'\n\
              return 1\n\
              g:\n\
              say 'sigl in g:' sigl\n\
              return 2\n",
        ),
        b"in f\nsigl in g: 1\n3\n".to_vec()
    );
}

/// A returned value traces **twice**, at two different indents: once as
/// the `RETURN`'s own value in the callee, once as the call's result in
/// the caller. A bare `return` traces neither.
#[test]
fn a_returned_value_traces_in_the_callee_and_again_in_the_caller() {
    let mut interp = Interp::new();
    run_source_traced(&mut interp, b"call sub\nexit\nsub:\nreturn 42\n").expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   return 42\n\
          \x20      >>>     \"42\"\n\
          \x20      >>>   \"42\"\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );

    let mut interp = Interp::new();
    run_source_traced(&mut interp, b"call sub\nexit\nsub:\nreturn\n").expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   return\n\
          \x20    2 *-* exit\n"
            .to_vec(),
        "a bare return produces no value line at either indent"
    );
}

/// The composition nobody had measured: a `CALL` **inside** an
/// `INTERPRET` fragment. Each activation's echo carries its *own* line,
/// so the enclosing fragment's line override has to be cleared for the
/// callee -- leaving it in force prints the callee's three clauses as
/// line 1 instead of 3, 4 and 5.
#[test]
fn a_call_inside_a_fragment_echoes_each_activations_own_line() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"interpret \"call sub\"\nexit\nsub:\nff = 7\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* interpret \"call sub\"\n\
          \x20      >>>   \"call sub\"\n\
          \x20    1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   ff = 7\n\
          \x20      >>>     \"7\"\n\
          \x20    5 *-*   return\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );
}

/// The other direction: an `INTERPRET` **inside** a called routine. The
/// fragment adds no indent of its own on top of the activation's, so all
/// four of the callee's lines sit at 2.
#[test]
fn an_interpret_inside_a_callee_runs_at_the_callees_own_level() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"call sub\nexit\nsub:\ninterpret \"gg = 8\"\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   interpret \"gg = 8\"\n\
          \x20      >>>     \"gg = 8\"\n\
          \x20    4 *-*   gg = 8\n\
          \x20      >>>     \"8\"\n\
          \x20    5 *-*   return\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );
}

/// I12's `CALL` half: each activation seals its own level, so the report
/// echoes one clause per activation, innermost first, each at its own
/// indent. Two calls deep from inside a `DO` gives 6, 4, 2 -- the same
/// three-deep transcript the oracle prints.
#[test]
fn the_report_echoes_one_clause_per_activation_innermost_first() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"do\ncall one\nend\nexit\none:\ncall two\nreturn\ntwo:\nsay 1/0\nreturn\n",
    )
    .unwrap_err();
    let sealed: Vec<(usize, Vec<u8>, usize)> = interp
        .failure_sites
        .iter()
        .chain(interp.failure_site.iter())
        .map(|s| {
            (
                s.line().expect("a clause site"),
                s.text().to_vec(),
                s.indent().expect("a clause site"),
            )
        })
        .collect();
    assert_eq!(
        sealed,
        vec![
            (9, b"say 1/0".to_vec(), 6),
            (6, b"call two".to_vec(), 4),
            (2, b"call one".to_vec(), 2),
        ]
    );
}

/// Arguments are evaluated in the caller, before the callee starts. Not
/// observable through `USE ARG`/`ARG()` in this phase -- both still fail
/// loudly -- but a failing argument is: the condition is 42.3 and it is
/// reported against the `CALL` clause, not against anything in `sub`.
#[test]
fn a_calls_arguments_are_evaluated_in_the_caller() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"call sub 1/0\nexit\nsub:\nreturn\n").unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 42),
        "an argument that raises must surface as its own condition, not run the callee: {failure:?}"
    );
    let site = interp.failure_site.expect("a site was resolved");
    assert_eq!((site.line(), site.text()), (Some(1), &b"call sub 1/0"[..]));
}

/// The quoted form bypasses the label search entirely, so `call "SUB"`
/// with `sub:` present is *not* a call. This crate's answer is the loud
/// builtin/external fallback naming `4c`; the oracle's own is Error 43.1,
/// which is a claim about what is *not* a builtin either and so not
/// this crate's to make.
#[test]
fn a_quoted_call_name_never_reaches_the_label_table() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call \"SUB\"\nexit\nsub:\nsay 'ran'\nreturn\n",
    )
    .unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 43 && raised.sub == 1),
        "expected the oracle's own 43.1, got {failure:?}"
    );
    assert!(interp.out.is_empty(), "the label must not have run");

    // The unquoted spelling of the same name does reach it, which is
    // what makes the assertion above about the quotes and not about
    // `sub:` being unreachable.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"call SUB\nexit\nsub:\nsay 'ran'\nreturn\n"),
        b"ran\n".to_vec()
    );
}

/// `CALL (expr)` searches the label table, but with the value **verbatim**
/// -- the two halves pull opposite ways from the quoted form and both are
/// measured. Lower case finds nothing even though `sub:` is stored
/// upcased.
#[test]
fn a_dynamic_call_target_searches_labels_with_the_value_verbatim() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"nm = 'SUB'\ncall (nm)\nexit\nsub:\nsay 'ran'\nreturn\n"
        ),
        b"ran\n".to_vec()
    );

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"nm = 'sub'\ncall (nm)\nexit\nsub:\nsay 'ran'\nreturn\n",
    )
    .unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 43 && raised.sub == 1),
        "the unupcased value must not match the upcased label: {failure:?}"
    );
    assert!(interp.out.is_empty(), "the label must not have run");
}

/// `SIGNAL` out of a `DO`, landing on a label past it. Unlike `LEAVE`,
/// there is no search and no name to match -- every enclosing construct
/// is abandoned unconditionally, so the loop's own later iterations
/// never happen and neither does the clause right after `END`.
///
/// I1 (Task 6 fix round 1): a direct regression for `Flow::Signal`
/// versus reusing `Flow::Goto`. Collapsing the two `Ok(Flow::Signal(
/// target))` sites in the `Signal` step arm to `Ok(Flow::Goto(target))`
/// left every test in this file, and the whole workspace, green --
/// including every other `SIGNAL` test above and below this one, none
/// of which happens to have a fragment whose own instruction count
/// reaches the enclosing label's own index. This one does, on purpose:
/// `here:` sits at enclosing body index 2 (`say 'A'` is 0, `interpret`
/// is 1), and the fragment `"nop; signal here; say 'WRONG BRANCH RAN'"`
/// has 3 instructions, so the escaping target (2) satisfies
/// `run_bounded`'s own absorption guard (`target >= start(0) && target
/// <= end(3)`) against the *fragment's* range. A `Goto`-collapsed build
/// -- verified directly, reverted before committing -- absorbs the jump
/// as its own, resumes stepping the fragment's own third instruction,
/// and prints `WRONG BRANCH RAN` in the middle; the correct build
/// escapes past the fragment entirely and never prints it.
///
/// **No second, self-referential ("g2") variant is added here.** The
/// review that found this also found a shape where the escaping target
/// lands on the fragment's *own* `SIGNAL` instruction (a label at
/// enclosing index 0, a one-instruction fragment `"signal top"`) --
/// under `Goto` reuse that does not print a wrong answer, it spins
/// forever: `run_bounded`'s `while pc < end` loop has no iteration
/// budget, and landing back on the same deterministic instruction
/// reproduces the identical `Goto` every pass. That is true of *any*
/// collision where the absorbed target is at or before the `SIGNAL`'s
/// own position inside the fragment, not only the minimal one -- moving
/// the target forward past the `SIGNAL` (this test's own shape) is what
/// makes the wrong run terminate at all. There is no bounded encoding of
/// the backward/self-referential shape as a live-executed test, only a
/// choice between not testing it and risking a hang the moment this
/// regression guard itself regresses; this crate's own precedent
/// (`MAX_ACTIVATION_DEPTH`, D19/I6) is to convert an unbounded case into
/// a bounded, reportable one rather than accept an unbounded test, and
/// nothing here does that for a bare `while` loop's own iteration count.
/// Documented instead of encoded: this doc comment and `Flow::Signal`'s
/// own are where the fact lives.
#[test]
fn signal_out_of_a_fragment_does_not_collide_with_the_fragments_own_index_space() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'A'\n\
              interpret \"nop; signal here; say 'WRONG BRANCH RAN'\"\n\
              here: say 'landed correctly'\n",
        ),
        b"A\nlanded correctly\n".to_vec()
    );
}

/// I3 (Task 6 fix round 1): `SIGNAL` out of a `SELECT`, the one Step 1
/// shape the original landing measured for `DO` and `INTERPRET` but
/// never for `SELECT` -- `leave_select` (`run.rs`) is one of the
/// forwarding sites `Flow::Signal`'s own design argument depends on, and
/// it was the only one with no witness. Measured (source with a leading
/// `trace r` clause, lines decremented by one as every other traced test
/// in this file already does): `SELECT` is abandoned exactly like `DO`
/// is, unconditionally, and `SIGL` (C1, this same fix round) reads back
/// the `WHEN`'s own line, all three sub-clauses (`WHEN`, `THEN`,
/// `SIGNAL`) sharing that one source line.
#[test]
fn signal_out_of_a_select_unwinds_it_and_lands_on_its_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say 'before'\n\
          select\n\
          \x20 when 1 = 1 then signal there\n\
          \x20 otherwise\n\
          \x20   say 'not reached'\n\
          end\n\
          say 'after select, not reached'\n\
          exit\n\
          there:\n\
          say 'reached there sigl:' sigl\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say 'before'\n\
          \x20      >>>   \"before\"\n\
          \x20    2 *-* select\n\
          \x20    3 *-*   when 1 = 1 \n\
          \x20      >>>     \"1\"\n\
          \x20    3 *-*     then\n\
          \x20    3 *-*       signal there\n\
          \x20    9 *-* there:\n\
          \x20   10 *-* say 'reached there sigl:' sigl\n\
          \x20      >>>   \"reached there sigl: 3\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"before\nreached there sigl: 3\n".to_vec());
}

/// C1 (Task 6 fix round 1): `SIGL`. The oracle's own `RexxActivation::
/// signalTo`/`internalCall` (`execution/RexxActivation.cpp`, read
/// directly) both call `new_integer(lineNum)` at the point of transfer
/// -- an integer object, which is why `set_sigl` uses `self.text`
/// rather than `self.number`: measured, a `SIGL` value of `22` still
/// renders `22` under `NUMERIC DIGITS 1`, where the identical magnitude
/// as an arithmetic result would round to `2E+1`.
///
/// Five shapes, each measured against the oracle in a clean directory
/// before being pinned here:
#[test]
fn sigl_is_set_at_every_control_transfer() {
    // Uninitialised until the first transfer, like any other variable;
    // `SIGNAL` sets it to its own line.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'sigl before:' sigl\nsignal there\nsay 'no'\nthere:\nsay 'sigl after:' sigl\n",
        ),
        b"sigl before: SIGL\nsigl after: 2\n".to_vec()
    );

    // `CALL` sets it too, visible inside the callee (D9r's shared pool)
    // and left set after the callee returns -- an ordinary variable,
    // never restored at the activation boundary.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'before:' sigl\n\
              call sub\n\
              say 'after call:' sigl\n\
              exit\n\
              \n\
              sub:\n\
              say 'in sub sigl:' sigl\n\
              return\n",
        ),
        b"before: SIGL\nin sub sigl: 2\nafter call: 2\n".to_vec()
    );

    // From inside an `INTERPRET` fragment, `SIGL` reads the *enclosing*
    // `INTERPRET` clause's own line, not any line internal to the
    // fragment -- matching the oracle's own `signalTo`, which delegates
    // a `SIGNAL` fired inside an interpret-created activation to its
    // parent, whose own currently-executing instruction is the
    // `INTERPRET` itself (this crate reproduces the observable answer
    // through `current_clause_line`/`clause_line_override` instead,
    // without adopting that nested-activation architecture).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'before:' sigl\n\
              interpret \"signal there\"\n\
              say 'no'\n\
              there:\n\
              say 'sigl after signal-in-fragment:' sigl\n",
        ),
        b"before: SIGL\nsigl after signal-in-fragment: 2\n".to_vec()
    );

    // The expression-call form (`f(1)`, Task 4) sets it exactly like
    // `CALL` does.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'before:' sigl\n\
              say f(1)\n\
              say 'after:' sigl\n\
              exit\n\
              f: return 'called, sigl=' || sigl\n",
        ),
        b"before: SIGL\ncalled, sigl=2\nafter: 2\n".to_vec()
    );

    // `PROCEDURE` isolates it exactly like any other variable: the
    // callee's own `SIGL` starts uninitialised in its own frame, and
    // the caller's `SIGL` (already set by the `CALL` itself, before the
    // callee ever ran) is unaffected by whatever the isolated callee
    // does with its own copy.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\n\
              say 'main sigl after sub returns:' sigl\n\
              exit\n\
              \n\
              sub:\n\
              procedure\n\
              say 'sub sigl (isolated):' sigl\n\
              return\n",
        ),
        b"sub sigl (isolated): SIGL\nmain sigl after sub returns: 1\n".to_vec()
    );
}

/// **Fires on the loop's first pass, deliberately** -- a second pass
/// through a `Controlled` (`TO`-style) `DO`/`LOOP` retraces two further
/// `>>>` lines this crate does not reproduce (the documented "KNOWN
/// GAP" at `loop_advance`'s own `Controlled` arm, unrelated to `SIGNAL`),
/// and a witness that reached a second
/// pass would be asserting that gap's own wrong output rather than
/// `SIGNAL`'s. Measured (source with a leading `trace r` clause, every
/// line number then decremented by one to match `run_source_traced`'s
/// own externally-set mode -- `current_value_indent_is_restored_after_a_
/// nested_expression_call`'s own doc comment has the transformation):
/// the `SIGNAL` clause itself traces with no `>>>` line, unlike `CALL`'s
/// dynamic form or `RETURN` with a value, because it produces nothing.
#[test]
fn signal_unwinds_a_nested_do_and_lands_on_its_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say 'before'\n\
          do i = 1 to 3\n\
          \x20 if i = 1 then signal there\n\
          \x20 say 'i=' i\n\
          end\n\
          say 'after loop, not reached'\n\
          exit\n\
          there:\n\
          say 'reached there'\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say 'before'\n\
          \x20      >>>   \"before\"\n\
          \x20    2 *-* do i = 1 to 3\n\
          \x20      >K>   \"TO\" => \"3\"\n\
          \x20    3 *-*   if i = 1 \n\
          \x20      >>>     \"1\"\n\
          \x20    3 *-*     then\n\
          \x20    3 *-*       signal there\n\
          \x20    8 *-* there:\n\
          \x20    9 *-* say 'reached there'\n\
          \x20      >>>   \"reached there\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"before\nreached there\n".to_vec());
}

/// A `SIGNAL` target that matches no label in the running activation's
/// own body is Error 16.1, "Label not found" -- unlike `CALL`'s own
/// unresolved name, which still has a builtin/external fallback to defer
/// to (`Interp::resolve_call`'s own doc), `SIGNAL` has none, so this is
/// the oracle's real answer and not a loud gap.
#[test]
fn signal_to_an_undefined_label_raises_16_1() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"say 'before'\nsignal nowhere\nsay 'not reached'\n",
    )
    .unwrap_err();
    assert!(
        matches!(
            &failure,
            Failure::Raised(raised)
                if raised.number == 16
                    && raised.sub == 1
                    && raised.additional == vec![b"NOWHERE".to_vec()]
        ),
        "expected 16.1 naming \"NOWHERE\", got {failure:?}"
    );
    assert_eq!(
        interp.out,
        b"before\n".to_vec(),
        "the clause after SIGNAL must not run"
    );
}

/// A quoted `SIGNAL` target searches the label table -- unlike a quoted
/// `CALL` target, which never does at all (`a_quoted_call_name_never_
/// reaches_the_label_table`, above) -- but case-sensitively against the
/// label's own upcased spelling, so a lowercase quoted spelling still
/// misses and raises 16.1 naming the verbatim quoted text, rather than
/// taking `CALL`'s own loud unresolved-call fallback. A bare symbol is upcased at
/// parse time regardless of its own case, and an uppercase quoted
/// spelling matches for the same reason a lowercase one does not.
#[test]
fn a_quoted_signal_label_searches_case_sensitively_unlike_a_quoted_call() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal \"sub\"\nsay 'not reached'\nsub:\nsay 'reached'\n",
    )
    .unwrap_err();
    assert!(
        matches!(
            &failure,
            Failure::Raised(raised)
                if raised.number == 16
                    && raised.sub == 1
                    && raised.additional == vec![b"sub".to_vec()]
        ),
        "expected 16.1 naming the verbatim quoted text \"sub\": {failure:?}"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal Sub\nsay 'not reached'\nsub:\nsay 'reached'\n"
        ),
        b"reached\n".to_vec(),
        "a bare, mixed-case symbol is upcased at parse time and matches"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal \"SUB\"\nsay 'not reached'\nsub:\nsay 'reached'\n"
        ),
        b"reached\n".to_vec(),
        "the uppercase quoted spelling matches -- it is the case, not the quoting"
    );
}

/// The composition nobody had measured: `SIGNAL` from inside a called
/// routine, targeting a label written back in the *caller's* own text.
///
/// **It reaches, and that is not `SIGNAL` crossing an activation
/// boundary on its own.** At this phase every internal `CALL` target
/// shares its caller's exact body and label table (`resolve_and_run_
/// call`'s own D9r comment: no `::routine` directive gives a callee one
/// of its own yet), so `resolve_signal_target`'s "search the running
/// activation's own body" finds `caller_label:` from inside `sub` for
/// the mundane reason that `sub`'s own body *is* the caller's. Measured
/// against the oracle rather than assumed, per this phase's own method.
///
/// **And it never returns to the original `CALL`'s own next clause.**
/// `SIGNAL`, unlike `RETURN`, never pops the activation it fires in --
/// so once the label's own code runs out of further instructions, the
/// *callee's* activation falls off the end, which ends the whole
/// program (`Ended::Exited`'s own doc comment), not merely the call.
#[test]
fn signal_from_a_called_routine_reaches_a_label_in_the_shared_body_and_never_returns() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\n\
              say 'after call, not reached'\n\
              exit\n\
              \n\
              sub:\n\
              say 'in sub'\n\
              signal caller_label\n\
              say 'sub not reached'\n\
              return\n\
              \n\
              caller_label:\n\
              say 'caller label reached'\n",
        ),
        b"in sub\ncaller label reached\n".to_vec()
    );
}

/// `SIGNAL` out of an `INTERPRET` fragment reaches an enclosing label --
/// unlike `LEAVE`/`ITERATE`, whose own search stops dead at the fragment
/// boundary (`run_fragment`'s own doc comment has that transcript
/// table), `SIGNAL` is forwarded like `Exit`/`Return` because it is a
/// new `Flow` variant with no arm of its own at that boundary
/// (`Flow::Signal`'s own doc comment). Measured (source with a leading
/// `trace r` clause, lines decremented by one to match `run_source_
/// traced`, as above): the fragment's own clause echoes at the enclosing
/// `INTERPRET`'s line, then control resumes at the enclosing label's own
/// line, exactly like `interpret "call sub"` already does for `CALL`.
#[test]
fn signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say 'before'\n\
          interpret \"signal there\"\n\
          say 'not reached'\n\
          exit\n\
          there:\n\
          say 'reached there'\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say 'before'\n\
          \x20      >>>   \"before\"\n\
          \x20    2 *-* interpret \"signal there\"\n\
          \x20      >>>   \"signal there\"\n\
          \x20    2 *-* signal there\n\
          \x20    5 *-* there:\n\
          \x20    6 *-* say 'reached there'\n\
          \x20      >>>   \"reached there\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"before\nreached there\n".to_vec());
}

/// The failure twin of the test above: a `SIGNAL` inside an `INTERPRET`
/// fragment that matches no label reports **both** clauses, innermost
/// first, each carrying the enclosing `INTERPRET`'s own line -- the same
/// shape `run_fragment`'s own doc comment tables for `LEAVE`/`ITERATE`,
/// now measured for `SIGNAL` too.
#[test]
fn signal_to_an_undefined_label_inside_a_fragment_reports_both_clauses() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"say 'before'\ninterpret \"signal nowhere\"\nsay 'not reached'\n",
    )
    .unwrap_err();
    assert!(matches!(
        &failure,
        Failure::Raised(raised) if raised.number == 16 && raised.sub == 1
    ));
    let sealed: Vec<(usize, Vec<u8>)> = interp
        .failure_sites
        .iter()
        .chain(interp.failure_site.iter())
        .map(|s| (s.line().expect("a clause site"), s.text().to_vec()))
        .collect();
    assert_eq!(
        sealed,
        vec![
            (2, b"signal nowhere".to_vec()),
            (2, b"interpret \"signal nowhere\"".to_vec()),
        ]
    );
    assert_eq!(interp.out, b"before\n".to_vec());
}

/// `SIGNAL VALUE`'s own `>K>` line -- `"VALUE" => text`, at the clause's
/// own indent with no extra `+2` the way `WHILE`/`UNTIL` carry, since
/// nothing here is evaluated as part of an *enclosing* instruction's own
/// step the way a loop's condition is. Measured one `DO` deep (lines
/// decremented by one as above): once traced, the rendered value is
/// searched exactly like a bare `SIGNAL label`'s own bytes.
#[test]
fn signal_value_traces_its_own_keyword_line_and_then_searches_like_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"target = 'THERE'\n\
          do i = 1 to 1\n\
          \x20 signal value target\n\
          end\n\
          say 'not reached'\n\
          exit\n\
          there:\n\
          say 'reached, one do deep'\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* target = 'THERE'\n\
          \x20      >>>   \"THERE\"\n\
          \x20    2 *-* do i = 1 to 1\n\
          \x20      >K>   \"TO\" => \"1\"\n\
          \x20    3 *-*   signal value target\n\
          \x20      >K>     \"VALUE\" => \"THERE\"\n\
          \x20    7 *-* there:\n\
          \x20    8 *-* say 'reached, one do deep'\n\
          \x20      >>>   \"reached, one do deep\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"reached, one do deep\n".to_vec());
}

/// `SIGNAL VALUE`'s target gets **no shape check at all** before the
/// label search: a number, an empty string and an ordinary non-label
/// string all raise 16.1 naming that exact rendered text, none of them a
/// different error -- and the search is case-sensitive exactly like a
/// quoted `SIGNAL label`'s own (`a_quoted_signal_label_searches_case_
/// sensitively_unlike_a_quoted_call`, above), so a lowercase value does
/// not match a label stored upcased even under the identical spelling.
#[test]
fn signal_value_targets_that_match_no_label_all_raise_16_1_naming_the_rendered_text() {
    for (expr, expected) in [
        ("123", "123"),
        ("''", ""),
        ("'no such label'", "no such label"),
    ] {
        let mut interp = Interp::new();
        let source = format!("target = {expr}\nsignal value target\nsay 'not reached'\n");
        let failure = run_source(&mut interp, source.as_bytes()).unwrap_err();
        assert!(
            matches!(
                &failure,
                Failure::Raised(raised)
                    if raised.number == 16
                        && raised.sub == 1
                        && raised.additional == vec![expected.as_bytes().to_vec()]
            ),
            "{expr}: expected 16.1 naming {expected:?}, got {failure:?}"
        );
    }

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"target = 'there'\n\
          signal value target\n\
          say 'not reached'\n\
          exit\n\
          there:\n\
          say 'reached'\n",
    )
    .unwrap_err();
    assert!(
        matches!(
            &failure,
            Failure::Raised(raised)
                if raised.number == 16
                    && raised.sub == 1
                    && raised.additional == vec![b"there".to_vec()]
        ),
        "a lowercase value must not match the upcased label THERE: {failure:?}"
    );
}

/// The body selector's `Some(index)` half, which no execution path can
/// construct yet (`Activation::body`'s own doc has why). Exercised
/// directly so the arm is not merely written: a parsed `::routine` has a
/// body, and the selector resolves to *that* body rather than to `main`.
#[test]
fn the_body_selector_resolves_a_routine_directive_and_rejects_a_bad_index() {
    let program = parse_program(b"call foo\nexit\n::routine foo\nsay 'in foo'\n".to_vec()).unwrap();
    let main = body_of(&program, None).expect("the main body always resolves");
    assert_eq!(main.instructions.len(), program.main.instructions.len());

    let routine = body_of(&program, Some(0)).expect("the routine directive has a body");
    assert_eq!(
        routine.instructions.len(),
        1,
        "the routine's own body is its one `say`, not the main body's two clauses"
    );
    assert!(
        !std::ptr::eq(routine, main),
        "a routine selector must not resolve to the main body"
    );

    assert!(
        body_of(&program, Some(1)).is_none(),
        "an out-of-range selector resolves to nothing rather than panicking"
    );
}

// ---- PROCEDURE, PROCEDURE EXPOSE, USE and the variable reference ----
//
// Every program below runs its `PROCEDURE` through a real `CALL`, and
// not because a `CALL` reads better: `run_source` drives the body
// through `run_bounded`, and only `run_activation` grants the
// first-instruction permission a `PROCEDURE` needs. A `PROCEDURE`
// reached any other way is error 17.1 -- which is the oracle's own
// answer too, measured, and what
// `a_procedure_that_is_not_a_calls_first_instruction_raises_17_1`
// asserts.
//
// **No value in these programs equals its own variable's derived name.**
// An unexposed unset read yields the name, so a witness whose exposed
// variable holds, say, `W` in `w` cannot tell exposure from
// non-exposure. Every literal here is a hyphenated word no derived name
// can collide with.

#[test]
fn procedure_isolates_and_expose_aliases_the_caller_entry() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'caller-v'\ncall sub\nsay v w\nexit\n\
              sub: procedure expose w\nv = 'callee-v'\nw = 'callee-w'\nreturn\n",
        ),
        b"caller-v callee-w\n".to_vec(),
        "V is the callee's own variable and must not have escaped; W is \
         exposed and must have"
    );
}

/// Exposure is transitive: `a` exposes `n` to `b`, `b` exposes the same
/// `n` to `c`, and `c`'s write is visible in `a`.
///
/// Measured on the oracle. Binding `c`'s `n` to `b`'s frame instead of
/// chasing `b`'s own alias passes at one level and gives `from-a` here.
#[test]
fn exposure_is_transitive_through_an_intermediate_procedure() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 'from-a'\ncall bee\nsay 'a sees:' n\nexit\n\
              bee: procedure expose n\ncall cee\nsay 'bee sees after c:' n\nreturn\n\
              cee: procedure expose n\nn = 'set-by-cee'\nreturn\n",
        ),
        b"bee sees after c: set-by-cee\na sees: set-by-cee\n".to_vec()
    );
}

/// **The program that refuted "a bitset plus one target `SlotFrame`".**
///
/// One `PROCEDURE` exposes two names that live in two different frames:
/// `n` chases through `bee`'s alias up to `a`, while `m` is `bee`'s own
/// local and stops there. Measured on the oracle -- `bee` sees both of
/// `cee`'s writes and `a` sees only `n`'s.
///
/// Any design carrying a single target frame per callee gets exactly one
/// of the two names right, whichever frame it picked, so this is the
/// test that cannot pass by accident. It is also why `RootSet`'s
/// redirect is a per-slot `Vec<Option<usize>>`.
#[test]
fn one_procedure_can_expose_names_living_in_two_different_frames() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 'from-a'\nm = 'from-a-m'\ncall bee\nsay 'a sees:' n m\nexit\n\
              bee: procedure expose n\nm = 'from-bee-m'\ncall cee\n\
              say 'bee sees:' n m\nreturn\n\
              cee: procedure expose n m\nn = 'set-by-cee'\nm = 'set-by-cee-m'\nreturn\n",
        ),
        b"bee sees: set-by-cee set-by-cee-m\na sees: set-by-cee from-a-m\n".to_vec(),
        "N must reach A and M must stop at BEE, from one PROCEDURE"
    );
}

/// `EXPOSE (v)` is plural and also exposes `v` itself. Both halves are
/// measured; `DROP (v)` has the identical shape.
#[test]
fn the_indirect_expose_form_is_plural_and_exposes_its_own_selector() {
    // Plural, with GAMMA as the control: it is never named and must not
    // be exposed, so a version that exposed everything passes the first
    // two assertions and fails on it.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"list = 'ALPHA BETA'\nalpha = 'a-in-caller'\nbeta = 'b-in-caller'\n\
              gamma = 'g-in-caller'\ncall sub\nsay alpha beta gamma\nexit\n\
              sub: procedure expose (list)\n\
              alpha = 'a-set'\nbeta = 'b-set'\ngamma = 'g-set'\nreturn\n",
        ),
        b"a-set b-set g-in-caller\n".to_vec()
    );

    // The selector itself. The callee reads `v` as `zzz` (the caller's
    // value, so `v` is exposed) and writes both names, and the caller
    // sees both writes.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'zzz'\nzzz = 'z-in-caller'\ncall sub\n\
              say 'caller v:' v 'caller zzz:' zzz\nexit\n\
              sub: procedure expose (v)\nsay 'callee v:' v 'callee zzz:' zzz\n\
              v = 'v-set'\nzzz = 'zzz-set'\nreturn\n",
        ),
        b"callee v: zzz callee zzz: z-in-caller\ncaller v: v-set caller zzz: zzz-set\n".to_vec()
    );
}

/// The five stem transcripts D9r records, all measured on the oracle.
///
/// Their common point is that `EXPOSE` aliases the caller's **variable
/// entry**, not the stem *object*, which is what the `drop` pair pins:
/// `drop a.` in the callee rebinds the caller's entry to a fresh stem,
/// while a second variable holding the old object still sees the old
/// tail. That is why `stem_drop`'s `replace_stem(name, None)` shape is
/// correct under exposure and must not become a slot clear.
#[test]
fn an_exposed_stem_aliases_the_callers_entry_not_the_object() {
    for (source, expected, why) in [
        (
            &b"a.1 = 'kept'\ncall sub\nsay a.1\nexit\n\
               sub: procedure expose a.\na.1 = 'changed'\nreturn\n"[..],
            &b"changed\n"[..],
            "a tail written in the callee is visible through the caller's stem",
        ),
        (
            b"a.1 = 'kept'\ncall sub\nsay a.1\nexit\n\
              sub: procedure expose a.\na. = 'wiped'\nreturn\n",
            b"wiped\n",
            "a whole-stem assignment in the callee replaces the caller's stem",
        ),
        (
            b"a.1 = 'kept'\ncall sub\nsay a.1\nexit\n\
              sub: procedure expose a.\ndrop a.\nreturn\n",
            b"A.1\n",
            "DROP of an exposed stem leaves the caller's stem looking untouched",
        ),
        (
            b"a.1 = 'orig'\nkeep. = a.\ncall sub\nsay a.1 keep.1\nexit\n\
              sub: procedure expose a.\ndrop a.\nreturn\n",
            b"A.1 orig\n",
            "the DROP rebinds the entry; KEEP. still holds the old object, which \
             is what distinguishes rebinding from clearing",
        ),
        (
            b"a.1 = 'from-caller'\nother.1 = 'not-exposed'\ncall sub\nexit\n\
              sub: procedure expose a.\nsay 'callee reads:' a.1 other.1\nreturn\n",
            b"callee reads: from-caller OTHER.1\n",
            "the callee reads the exposed stem's tail and derives the name of the \
             unexposed one",
        ),
    ] {
        let mut interp = Interp::new();
        assert_eq!(say_output(&mut interp, source), expected.to_vec(), "{why}");
    }
}

/// A name the plan never saw, exposed through the indirect form, has to
/// keep resolving to the same slot on both sides of the return.
///
/// `ZQXW` appears in no instruction of either routine -- only inside
/// string literals -- so the plan has no slot for it and both sides
/// reach it only through a run-time binding. Measured on the oracle.
/// This is the case `exec_procedure`'s write-back of `extra` exists for,
/// and the one that would otherwise need a non-top `grow_slots`.
#[test]
fn a_computed_expose_of_a_name_no_instruction_mentions_survives_the_return() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"nm = 'ZQXW'\ncall sub\ninterpret \"say 'caller:' zqxw\"\nexit\n\
              sub: procedure expose (nm)\ninterpret \"zqxw = 'set-in-callee'\"\nreturn\n",
        ),
        b"caller: set-in-callee\n".to_vec()
    );
}

/// 17.1 at every shape but the legal one, and labels are transparent.
///
/// All five measured on the oracle. The `nop` case beside the two-label
/// case is what shows the rule is "first instruction *executed*" with
/// labels not counting, rather than "first instruction in the body".
#[test]
fn a_procedure_that_is_not_a_calls_first_instruction_raises_17_1() {
    for (source, why) in [
        (
            &b"say 'top'\nprocedure\n"[..],
            "at top level, with no call at all",
        ),
        (
            b"say 'main'\nsub:\nprocedure\n",
            "fallen into rather than called",
        ),
        (
            b"call sub\nexit\nsub:\nnop\nprocedure\nreturn\n",
            "after a NOP in a called routine",
        ),
        (
            b"call sub\nexit\nsub: interpret \"procedure\"\nreturn\n",
            "inside a fragment, which does not inherit its host's permission",
        ),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised for {why}, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (17, 1), "{why}");
    }

    // Two labels between the CALL and the PROCEDURE: legal, measured.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"outer = 'caller'\ncall sub\nsay outer\nexit\n\
              sub:\nlbl2:\nprocedure\nouter = 'callee'\nreturn\n",
        ),
        b"caller\n".to_vec(),
        "a label neither grants the permission nor spends it, and the PROCEDURE \
         still isolated"
    );
}

/// A `::ROUTINE` is **not** an internal call, so a `PROCEDURE` first in
/// one is 17.1 -- reached by `CALL` here, and as a function below.
///
/// **The exact stderr, because nothing else in the suite can see these
/// bytes.** `support::normalize_stderr` (DEVIATION 0) collapses the
/// space run between a trace line's marker and its content, so
/// `tests/corpus.rs` compares this program's two echoed clauses equal to
/// the same two echoed at any other indent. The ordering it does see:
/// the failing clause first, at the routine's own indent 0, and the
/// *calling* clause second.
///
/// The expectation is the oracle's own transcript for this program, `cat
/// -A`'d in a clean directory, at rc 239 with an empty stdout -- the
/// `say zz` after the `call` never runs on either side.
#[test]
fn a_procedure_first_in_a_called_routine_is_17_1_with_the_calling_clause_second() {
    const PATH: &str = "/tmp/proc-in-routine-call.rex";
    let outcome = crate::run_program(
        PATH,
        b"call sub\n\
          zz = 1\n\
          say zz\n\
          exit 0\n\
          ::routine sub\n\
          procedure\n\
          say 'in sub'\n"
            .to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 239, "256 - 17");
    assert_eq!(outcome.stdout, b"");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        format!(
            "\x20    6 *-* procedure\n\
             \x20    1 *-* call sub\n\
             Error 17 running {PATH} line 6:  Unexpected PROCEDURE.\n\
             Error 17.1:  PROCEDURE is valid only when it is the first \
             instruction executed after an internal CALL or function \
             invocation.\n"
        )
    );
}

/// The same routine reached as a **function** rather than by `CALL`.
///
/// A separate test rather than a row in the one above, because the two
/// shapes corrupt the frame stack differently when the instruction is
/// admitted: the `CALL` shape leaves the caller's next unbound name
/// growing a frame that is no longer the top one, and this one fails on
/// the way out instead, popping a frame that is not the top. A program
/// containing both only ever reaches the first.
#[test]
fn a_procedure_first_in_a_routine_reached_as_a_function_is_17_1_too() {
    const PATH: &str = "/tmp/proc-in-routine-function.rex";
    let outcome = crate::run_program(
        PATH,
        b"qq = sub()\n\
          say 'main' qq\n\
          exit 0\n\
          ::routine sub\n\
          procedure\n\
          return 'in sub'\n"
            .to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 239, "256 - 17");
    assert_eq!(outcome.stdout, b"");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        format!(
            "\x20    5 *-* procedure\n\
             \x20    1 *-* qq = sub()\n\
             Error 17 running {PATH} line 5:  Unexpected PROCEDURE.\n\
             Error 17.1:  PROCEDURE is valid only when it is the first \
             instruction executed after an internal CALL or function \
             invocation.\n"
        )
    );
}

/// A `PROCEDURE` first in an internal label reached as a **function** is
/// legal, which is the neighbouring success the two refusals above are
/// paired with.
///
/// Both routes into a label are separate admissions, and a rule keyed on
/// "was this a `CALL` instruction" rather than on the entry kind refuses
/// this one while leaving the `CALL` route working.
#[test]
fn a_procedure_first_in_a_label_reached_as_a_function_still_runs() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"outer = 'caller'\nqq = sub()\nsay outer qq\nexit\n\
              sub: procedure\nouter = 'callee'\nreturn 'func-ok'\n",
        ),
        b"caller func-ok\n".to_vec()
    );
}

/// An isolated callee's frame is released on the way out, on the error
/// path as well as the ordinary one.
///
/// Asserted against the root set's own slot count rather than through
/// output, because a leak is invisible in a program's bytes: the run
/// would still be correct and would simply hold one frame per call
/// forever, which `do 100000; call sub; end` turns into 100,000 rooted
/// frames. Both paths are checked here because they share the one
/// `pop_slots` call whose position in `Interp::invoke_call` is the whole
/// point -- outside the `Ok` arm, not inside it.
///
/// The property is that frames balance, so this counts **frames** and not
/// slots. A slot count moves for reasons that are not leaks -- the first
/// `CALL` in a program that never writes `RESULT` grows the top frame by
/// one to hold it, measured while writing this test -- and it moves by a
/// *different* amount on the error path, which never reaches that write.
/// `RootSet::live_frames` has no such confounder.
///
/// `run_source` leaves the top-level frame standing (only `Interp::run`
/// pops that one), so one frame is the correct answer for a balanced run
/// and each unreleased callee would add one more.
#[test]
fn an_isolated_callees_frame_is_released_on_both_paths() {
    let mut interp = Interp::new();
    say_output(
        &mut interp,
        b"zz = 1\ncall sub\ncall sub\ncall sub\nexit\nsub: procedure\nyy = 2\nreturn\n",
    );
    assert_eq!(
        interp.roots.live_frames(),
        1,
        "three isolated calls must leave only the top-level frame open"
    );

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"zz = 1\ncall sub\nexit\nsub: procedure\nyy = 2\nsay 1/0\nreturn\n",
    )
    .unwrap_err();
    assert!(
        matches!(failure, Failure::Raised(_)),
        "the callee must have raised, or this proves nothing about the error path"
    );
    assert_eq!(
        interp.roots.live_frames(),
        1,
        "a raise inside an isolated callee must still release its frame"
    );

    // The shared-pool path, which is the `else` branch of the same
    // `owns_frame` test and is reached by neither block above: a callee
    // with no PROCEDURE pushes no frame of its own (D9r), so it must not
    // pop one either.
    //
    // **What this pins, corrected after review.** It used to claim that
    // without it "an implementation that never pushed a callee frame
    // would pass both assertions above" -- false, and shown false by
    // building exactly that mutant, under which all three blocks pass,
    // because all three compare against the same number. No frame count
    // can catch a frame that is never pushed; the isolation tests are
    // what catch it (`procedure_isolates_and_expose_aliases_the_caller_
    // entry` and four others fail on it).
    //
    // What this block does catch, verified by mutation rather than
    // asserted: popping unconditionally instead of only when
    // `owns_frame`, which tears down the *caller's* still-live frame.
    // That mutant fails here and passes
    // `procedure_isolates_and_expose_aliases_the_caller_entry`, so this
    // block is the one carrying it.
    let mut interp = Interp::new();
    say_output(
        &mut interp,
        b"zz = 1\ncall sub\nexit\nsub:\nyy = 2\nreturn\n",
    );
    assert_eq!(
        interp.roots.live_frames(),
        1,
        "a shared-pool callee must leave the caller's frame open"
    );
}

// ---- USE ARG ----

#[test]
fn use_arg_binds_positionally_and_ignores_extra_arguments() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 1,2,3\nexit\nsub2: procedure\nuse arg p\nsay '['p']'\nreturn\n",
        ),
        b"[1]\n".to_vec()
    );

    // An omitted position holds its place rather than closing the list
    // up: an implementation that skipped it would bind R to 3.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 1,,3\nexit\n\
              sub2: procedure\nuse arg p, q, r\nsay '['p']' '['q']' '['r']'\nreturn\n",
        ),
        b"[1] [Q] [3]\n".to_vec()
    );
}

/// A target with no argument and no default is **dropped**, not left
/// alone.
///
/// The callee has no `PROCEDURE` on purpose, so the caller's `PRESET` is
/// the same variable and the drop is observable after the return.
/// `preset-value` is deliberately not `PRESET`: a target whose prior
/// value equalled its own derived name would render identically whether
/// it was dropped or left, so such a probe could not fail.
#[test]
fn use_arg_drops_a_target_with_no_argument_and_no_default() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"preset = 'preset-value'\ncall sub2 1\nsay 'after:' preset\nexit\n\
              sub2:\nuse arg p, preset\nsay 'inside:' preset\nreturn\n",
        ),
        b"inside: PRESET\nafter: PRESET\n".to_vec()
    );
}

#[test]
fn use_arg_defaults_fill_an_absent_or_omitted_position() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 1\nexit\n\
              sub2: procedure\nuse arg p, q = 'dflt'\nsay '['p']['q']'\nreturn\n",
        ),
        b"[1][dflt]\n".to_vec(),
        "absent past the end of the list"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 1,,3\nexit\n\
              sub2: procedure\nuse arg p, q = 'dflt', r\nsay '['p']['q']['r']'\nreturn\n",
        ),
        b"[1][dflt][3]\n".to_vec(),
        "omitted in the middle"
    );
}

/// `STRICT`'s two arity checks, and the two things that switch them off.
/// Every number and boundary measured on the oracle.
#[test]
fn use_strict_arg_checks_arity_at_both_ends() {
    // Too many: 40.4, naming the routine and the maximum.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call sub2 1,2,3\nexit\nsub2: procedure\nuse strict arg p\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (40, 4));
    assert_eq!(raised.additional, vec![b"SUB2".to_vec(), b"1".to_vec()]);

    // Too few: 40.3, naming the minimum.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call sub2 1\nexit\nsub2: procedure\nuse strict arg p, q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (40, 3));
    assert_eq!(raised.additional, vec![b"SUB2".to_vec(), b"2".to_vec()]);

    // A trailing `...` suppresses the maximum check only.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 1,2,3,4\nexit\n\
              sub2: procedure\nuse strict arg p, q, ...\nsay '['p']['q']'\nreturn\n",
        ),
        b"[1][2]\n".to_vec()
    );

    // A default satisfies the minimum, so this must not raise 40.3.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 1\nexit\n\
              sub2: procedure\nuse strict arg p, q = 'dflt'\nsay '['p']['q']'\nreturn\n",
        ),
        b"[1][dflt]\n".to_vec()
    );
}

/// `USE ARG >name` aliases the caller's variable; the same call into a
/// plain target copies its value instead.
///
/// The pair is what makes this test discriminating: an implementation
/// that aliased unconditionally, or never, gets exactly one of the two
/// right.
#[test]
fn use_arg_alias_binds_the_callers_variable_and_a_plain_target_does_not() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'orig'\ncall sub2 >p\nsay 'after:' p\nexit\n\
              sub2: procedure\nuse arg >q\nsay 'callee:' q\nq = 'aliased'\nreturn\n",
        ),
        b"callee: orig\nafter: aliased\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'orig'\ncall sub2 >p\nsay 'after:' p\nexit\n\
              sub2: procedure\nuse arg q\nq = 'aliased'\nreturn\n",
        ),
        b"after: orig\n".to_vec(),
        "a plain target copies the value, so the caller's P is untouched"
    );

    // A stem aliases the same way, measured.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"st.1 = 'orig'\ncall sub2 >st.\nsay 'after:' st.1\nexit\n\
              sub2: procedure\nuse arg >q.\nq.1 = 'aliased'\nreturn\n",
        ),
        b"after: aliased\n".to_vec()
    );

    // An aliased but unset variable reads as the *callee's* own derived
    // name, not the caller's -- measured, and it falls out of the alias
    // pointing at an unset slot.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 >unsetvar\nexit\n\
              sub2: procedure\nuse arg >q\nsay 'callee:' q\nreturn\n",
        ),
        b"callee: Q\n".to_vec()
    );
}

/// `USE ARG >` has two distinct refusals, and they are different
/// sub-numbers rather than one shared complaint. Both measured.
#[test]
fn use_arg_alias_refuses_a_plain_value_and_an_omitted_position() {
    // A supplied argument that is not a reference: 88.928, carrying the
    // 1-based position and the argument's own *value*.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"zebra = 'orig'\ncall sub2 zebra\nexit\nsub2: procedure\nuse arg >q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 928));
    assert_eq!(
        raised.additional,
        vec![b"1".to_vec(), b"orig".to_vec()],
        "the substitution is the argument's value, not the variable's spelling -- \
         a probe naming the variable `caller` could not tell those apart"
    );

    // An omitted position: 88.931, a different complaint.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call sub2 1\nexit\nsub2: procedure\nuse arg p, >q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 931));
    assert_eq!(raised.additional, vec![b"2".to_vec()]);
}

/// `USE ARG >name` requires its target to be **currently unset**, and
/// raises 98.995 otherwise.
///
/// **These three are a set and the middle one carries the weight.** The
/// message says "it must be an uninitialized *local* variable", which
/// invites writing the check as an exposure or locality test. The pair
/// that rules that out is the second and third cases below: the same
/// `procedure expose q`, raising when the exposed `q` holds a value and
/// succeeding when it does not. Exposure is identical in both; only the
/// value differs. The raising case alone does not pin which rule is being
/// applied, and the succeeding case alone cannot fail against a wrong
/// fix -- neither is worth much without the other.
#[test]
fn use_arg_alias_requires_an_uninitialised_target() {
    // Assigned, then aliased: refused, naming the target.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'p-orig'\ncall sub >p\nexit\n\
          sub: procedure\nq = 1\nuse arg >q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 995));
    assert_eq!(raised.additional, vec![b"Q".to_vec()]);

    // Exposed AND holding a value: still refused. Exposure is not the
    // trigger, but this case on its own cannot show that.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'p-orig'\nq = 'q-in-caller'\ncall sub >p\nexit\n\
          sub: procedure expose q\nuse arg >q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 995));

    // Exposed and UNSET: succeeds. This is what makes the pair
    // discriminating -- an exposure check would wrongly refuse here.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'p-orig'\ncall sub >p\nsay 'p:' p 'q:' q\nexit\n\
              sub: procedure expose q\nuse arg >q\nq = 'via-alias'\nreturn\n",
        ),
        b"p: via-alias q: Q\n".to_vec(),
        "the alias must have been installed, and the caller's own exposed Q must \
         still read unset"
    );

    // DROP restores the uninitialised state.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'p-orig'\nq = 'local'\ndrop q\ncall sub >p\nsay 'p:' p\nexit\n\
              sub:\nuse arg >q\nq = 'via-alias'\nreturn\n",
        ),
        b"p: via-alias\n".to_vec()
    );

    // Repeating `use arg >q` onto one target is the same rule, not a case
    // of its own: the first alias makes Q read the caller's variable, so
    // it has a value by the second. `RootSet::slot` resolving through the
    // alias is what makes this fall out.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'p-orig'\nr = 'r-orig'\ncall sub >p, >r\nexit\n\
          sub:\nuse arg >q\nuse arg xx, >q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 995));
}

/// `USE ARG >name` requires the reference's **kind** to match the
/// target's: a simple reference into a stem target is 88.929, and a stem
/// reference into a simple target is 88.930.
///
/// **Each refusal is paired with its adjacent success, in this test
/// rather than elsewhere, and the pairing is the point.** A test that
/// only checks `>p` into `>q.` raises cannot distinguish "the kinds must
/// match" from "a stem target is always refused"; the passing `>p.` into
/// `>q.` case is what rules the second out. The same holds mirrored for
/// 88.930. All four cells measured against the oracle.
#[test]
fn use_arg_alias_requires_the_reference_kind_to_match_the_target() {
    // Simple reference -> STEM target: refused.
    //
    // `p` holds `value-not-name` so that the substitution discriminates:
    // 88.929 names the *caller's variable*, `P`, where 88.928 in the same
    // position names the argument's *value*. A probe whose variable held
    // its own name could not tell those apart -- the mistake this task
    // already made once, on 88.928.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'value-not-name'\ncall sub >p\nexit\nsub: procedure\nuse arg >q.\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 929));
    assert_eq!(raised.additional, vec![b"1".to_vec(), b"P".to_vec()]);

    // ...and the adjacent success: a STEM reference into the same stem
    // target. Without this, "stem targets are always refused" passes.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p.1 = 'orig'\ncall sub >p.\nsay 'after:' p.1\nexit\n\
              sub: procedure\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
        ),
        b"after: via-alias\n".to_vec()
    );

    // Stem reference -> SIMPLE target: refused, naming `P.` with its
    // period.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p.1 = 'value-not-name'\ncall sub >p.\nexit\n\
          sub: procedure\nuse arg >q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 930));
    assert_eq!(raised.additional, vec![b"1".to_vec(), b"P.".to_vec()]);

    // ...and its adjacent success: a simple reference into a simple
    // target.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'orig'\ncall sub >p\nsay 'after:' p\nexit\n\
              sub: procedure\nuse arg >q\nq = 'via-alias'\nreturn\n",
        ),
        b"after: via-alias\n".to_vec()
    );

    // An argument that is not a reference at all is still 88.928, even
    // against a stem target, and still substitutes the VALUE. So the kind
    // check sits behind the is-a-reference check rather than replacing
    // it.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'value-not-name'\ncall sub p\nexit\nsub: procedure\nuse arg >q.\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 928));
    assert_eq!(
        raised.additional,
        vec![b"1".to_vec(), b"value-not-name".to_vec()]
    );

    // The position substitution is the argument's own, not always 1.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'val'\ncall sub 1, >p\nexit\nsub: procedure\nuse arg aa, >q.\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(raised.additional, vec![b"2".to_vec(), b"P".to_vec()]);

    // STRICT does not change the kind rule.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'val'\ncall sub >p\nexit\nsub: procedure\nuse strict arg >q.\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 929));
}

/// The kind check runs **before** the uninitialised check, so a target
/// that fails both reports the kind error.
///
/// Measured both ways round. Ordering is not cosmetic here: each of the
/// two errors is a different number and rc, so getting it backwards is a
/// wrong answer rather than a differently-worded right one. A single
/// test would not pin it -- both directions are needed, because a check
/// that always reported the kind error would pass one of them alone.
#[test]
fn use_arg_alias_reports_the_kind_mismatch_before_the_uninitialised_target() {
    // Stem target, already assigned, given a simple reference: 88.929,
    // not 98.995.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'val'\ncall sub >p\nexit\n\
          sub: procedure\nq.1 = 'already-set'\nuse arg >q.\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 929));

    // Simple target, already assigned, given a stem reference: 88.930,
    // not 98.995.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p.1 = 'val'\ncall sub >p.\nexit\n\
          sub: procedure\nq = 'already-set'\nuse arg >q\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (88, 930));
}

/// The stem half of the same rule, which is where this crate's own
/// representation shows through and where the obvious one-line check gets
/// it wrong.
///
/// `read_stem` vivifies a fresh empty `Body::Stem` into the slot on a bare
/// stem read, and `stem_drop` leaves exactly the same thing, so a slot
/// being `Some` is *not* the same question as the variable being
/// initialised. All five measured against the oracle; the first three
/// must succeed and would all raise under a plain `slot(..).is_some()`
/// test.
#[test]
fn use_arg_alias_treats_an_empty_stem_as_uninitialised() {
    for (source, expected, why) in [
        (
            &b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
               sub: procedure\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n"[..],
            &b"after: via-alias\n"[..],
            "a never-touched stem target",
        ),
        (
            b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
              sub: procedure\nsay 'bare read:' q.\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
            b"bare read: Q.\nafter: via-alias\n",
            "a bare stem READ vivifies an empty stem into the slot, and must not \
             count as initialising it",
        ),
        (
            b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
              sub: procedure\nq.1 = 'x'\ndrop q.\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
            b"after: via-alias\n",
            "DROP of a written stem restores the uninitialised state",
        ),
        (
            b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
              sub: procedure\nq.1 = 'x'\ndrop q.1\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
            b"after: via-alias\n",
            "a TOMBSTONED tail is not content -- `tails.is_empty()` would refuse this, \
             and did until it was measured",
        ),
        (
            b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
              sub: procedure\nq.1 = 'x'\nq.2 = 'y'\ndrop q.1\ndrop q.2\n\
              use arg >q.\nq.1 = 'via-alias'\nreturn\n",
            b"after: via-alias\n",
            "every tail tombstoned is still no content",
        ),
    ] {
        let mut interp = Interp::new();
        assert_eq!(say_output(&mut interp, source), expected.to_vec(), "{why}");
    }

    // A written tail initialises the stem: refused, naming `Q.` with its
    // period, which is the target's own spelling.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"st.1 = 'orig'\ncall sub >st.\nexit\n\
          sub: procedure\nq.1 = 'local'\nuse arg >q.\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 995));
    assert_eq!(raised.additional, vec![b"Q.".to_vec()]);

    // An assigned default initialises it too, with no tails written.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"st.1 = 'orig'\ncall sub >st.\nexit\n\
          sub: procedure\nq. = 'dflt'\nuse arg >q.\nreturn\n",
    )
    .unwrap_err();
    assert!(matches!(failure, Failure::Raised(_)));

    // **The two that make the rule "no LIVE tail" rather than "no
    // tails".** One tombstone beside one surviving tail still has
    // content; a default survives a tombstoned tail. Without this pair
    // the three succeeding tombstone rows above are equally consistent
    // with "ignore tails entirely", which would wrongly accept both.
    for source in [
        &b"st.1 = 'orig'\ncall sub >st.\nexit\n\
           sub: procedure\nq.1 = 'x'\nq.2 = 'y'\ndrop q.1\nuse arg >q.\nreturn\n"[..],
        b"st.1 = 'orig'\ncall sub >st.\nexit\n\
          sub: procedure\nq. = 'dflt'\ndrop q.1\nuse arg >q.\nreturn\n",
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 995));
    }

    // **The exemption is keyed on the target's NAME shape, not on the
    // value's.** A simple variable holding a fresh, empty stem object is
    // an initialised simple variable -- measured, `zz = q.` then `use arg
    // >zz` raises. A check written against "the value is an empty stem"
    // passes everything above and fails here.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"p = 'p-orig'\ncall sub >p\nexit\n\
          sub: procedure\nzz = q.\nuse arg >zz\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 995));
    assert_eq!(raised.additional, vec![b"ZZ".to_vec()]);
}

/// A variable reference in an ordinary value position is worth the
/// referenced variable's value. Measured: `say >p` prints `p`'s value.
#[test]
fn a_variable_reference_decays_to_the_referenced_value() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"p = 'orig'\nsay >p"),
        b"orig\n".to_vec()
    );
}

/// The caller's own arguments survive a nested call.
///
/// **This is Task 4's Critical, in this task's own currency.** That
/// finding was a piece of per-activation state a call failed to restore,
/// invisible until two activations per clause were reachable.
/// `Interp::call_context` is the fifth such piece; without the restore in
/// `Interp::invoke_call`, the second `USE ARG` below reads the *inner*
/// call's arguments and prints `inner-arg`.
#[test]
fn a_callers_arguments_survive_a_nested_call() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call outer 'outer-arg'\nexit\n\
              outer: procedure\nuse arg first\ncall inner 'inner-arg'\n\
              use arg second\nsay '['first']['second']'\nreturn\n\
              inner: procedure\nuse arg deep\nreturn\n",
        ),
        b"[outer-arg][outer-arg]\n".to_vec()
    );
}

/// `USE LOCAL` as a program's own first instruction is 98.993 -- the one
/// shape that reaches `exec_use` at all, since `rexx-parse` rejects the
/// rest at parse time. `exec_use`'s own doc comment lists the eight
/// shapes that were tried and why the 99.910 arm carries no test.
///
/// **Driven through `run_program` rather than `run_source`**, and that is
/// not incidental: this module's helper runs a body through
/// `run_bounded`, which never grants the first-instruction permission,
/// so a `run_source` version of this test would take the *other* arm and
/// assert 99.910 -- passing against an implementation that had the two
/// swapped. Only the real entry point puts the program in the state the
/// oracle's own 98.993 describes. Verified against the oracle in a clean
/// directory: rc 158 and these two lines.
#[test]
fn use_local_as_a_programs_first_instruction_raises_98_993() {
    let outcome = crate::run_program(
        "/tmp/use-local.rex",
        b"use local outer\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 158, "256 - 98");
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert!(
        stderr.contains("Error 98.993:")
            && stderr
                .contains("The USE LOCAL instruction may only be used from method invocations."),
        "expected the oracle's own 98.993 report, got: {stderr}"
    );
}

/// `USE LOCAL` first in a `::ROUTINE` is 98.993 as well, and **not**
/// 99.910.
///
/// The two numbers ask different questions, and a routine is the entry
/// that separates them: it is entered by a call, and it is not a method
/// invocation. Measured on the oracle in a clean directory, reached by
/// `CALL` and as a function alike -- rc 158, and the same message the
/// top-level shape gets.
#[test]
fn use_local_first_in_a_routine_raises_98_993_not_99_910() {
    for (source, why) in [
        (
            &b"call sub\nsay 'main'\nexit 0\n::routine sub\nuse local\n"[..],
            "reached by CALL",
        ),
        (
            b"qq = sub()\nsay 'main' qq\nexit 0\n::routine sub\nuse local\n",
            "reached as a function",
        ),
    ] {
        let outcome = crate::run_program(
            "/tmp/use-local-routine.rex",
            source.to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 158, "256 - 98, {why}");
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert!(
            stderr.contains("Error 98.993:")
                && stderr.contains(
                    "The USE LOCAL instruction may only be used from method invocations."
                ),
            "{why}: expected the oracle's own 98.993 report, got: {stderr}"
        );
    }
}

/// `PROCEDURE EXPOSE` of a single compound tail fails loudly rather than
/// approximating.
///
/// Measured on the oracle: exposing `a.1` shares that one tail and
/// leaves `a.2` the callee's own, which is aliasing inside a stem object
/// and not something a whole-slot alias can express. Exposing the stem
/// instead would silently share `a.2` too.
#[test]
fn procedure_expose_of_a_single_compound_tail_fails_loudly() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"a.1 = 'kept'\ncall sub\nexit\nsub: procedure expose a.1\nreturn\n",
    )
    .unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    assert!(
        loud.message.contains("A.1"),
        "the message must name the tail it refused: {}",
        loud.message
    );
}

// ---- EXPOSE and the scope-keyed variable pool ----
//
// **No value below equals its own variable's derived name**, the same rule
// the `PROCEDURE EXPOSE` block above states: an unbound exposed read yields
// the upcased name, so a witness holding `V` in `v` cannot tell a pool that
// works from one that does nothing.
//
// The programs that agree with the oracle byte for byte are in
// `corpus/lang/expose_*.rex` and are run by the corpus differential on both
// engines. What is here is what a corpus program cannot carry: a loud
// refusal, and the pool's own keying, which a program cannot reach because
// two scopes on one object need a superclass and `::CLASS ... SUBCLASS` is
// still a `directive_gap`.

/// `EXPOSE` of a single compound tail fails loudly, the same refusal
/// `PROCEDURE EXPOSE` makes and for the same reason.
///
/// Measured on the oracle: with `expose a.1` in a class method that assigns
/// both `a.1` and `a.2`, and `expose a.1` in a second one reading them back,
/// the answer is `[tail-one][A.2]` -- tail 1 is the object's and tail 2 is
/// the method's own local. That is aliasing inside a stem object, and a whole
/// name is what this crate's pool holds.
#[test]
fn expose_of_a_single_compound_tail_fails_loudly() {
    let mut interp = Interp::new();
    let failure = run_source_with_directives(
        &mut interp,
        b"say .K~m\n::class K\n::method m class\nexpose a.1\nreturn 1\n",
    )
    .unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    assert!(
        loud.message.contains("A.1") && loud.message.starts_with("EXPOSE "),
        "the message must name EXPOSE and the tail it refused: {}",
        loud.message
    );
}

/// The neighbouring success, which is what pins the refusal above to the
/// *tail* rather than to a compound having been mentioned at all: the whole
/// stem binds and round-trips through the pool.
#[test]
fn expose_of_a_whole_stem_binds_rather_than_refusing() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output_with_directives(
            &mut interp,
            b"x = .K~set\nsay .K~get\n::class K\n::method set class\n\
              expose a.\na.1 = 'tail-one'\nreturn 1\n::method get class\n\
              expose a.\nreturn a.1\n",
        ),
        b"tail-one\n".to_vec()
    );
}

/// The pool is keyed on a scope, and a name bound in one scope is not in
/// another's.
///
/// **What this can and cannot see.** It reads the pool a class method's own
/// `EXPOSE` wrote and asks a *different* class identity for the same name: an
/// implementation holding one flat table per object, or ignoring the scope
/// altogether, answers the value for both and fails here. What it cannot see
/// is the scope being confused with the *receiver*, because for a class
/// method on `::class K` those are the same handle -- separating them needs a
/// method inherited from a superclass, and `::CLASS ... SUBCLASS` does not
/// install yet. `rexx-core`'s `scope_pools.rs` carries that half against the
/// storage directly, with the transcript the oracle answers for the
/// two-scope program (`S B`, against `B B` for a pool keyed on the object
/// alone).
#[test]
fn a_pool_entry_belongs_to_one_scope_and_not_to_another() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output_with_directives(
            &mut interp,
            b"say .K~m\n::class K\n::method m class\nexpose v\n\
              v = 'pool-value'\nreturn 'ran'\n",
        ),
        b"ran\n".to_vec()
    );
    // Read out of the package's own installed-class table rather than out of
    // `class_variables`, whose key is the receiver: taking the scope from the
    // map under test would make the positive half true by construction.
    let scope = *interp
        .package_classes
        .values()
        .next()
        .expect("the program installed a package class")
        .get(b"K".as_slice())
        .expect("the program declared class K");
    let owner = *interp
        .class_variables
        .get(&scope)
        .expect("the EXPOSE gave K a variable object");
    let held = interp
        .pools_of(owner)
        .expect("that object holds pools")
        .get(scope, b"V")
        .expect("K's own scope holds V");
    assert_eq!(interp.to_text(held).into_owned(), b"pool-value".to_vec());
    let other = interp
        .classes()
        .lookup("Array")
        .expect("Array is a native class");
    assert_eq!(
        interp
            .pools_of(owner)
            .expect("that object holds pools")
            .get(other, b"V"),
        None,
        "V belongs to K's pool; another scope's pool must not answer it"
    );
}

// ---- condition traps, RAISE and NOVALUE ----
//
// Every trap test below asserts a value the *handler set*, never that
// the program exited 0: a criterion of the second kind is satisfied by a
// program that never raised at all. The values are chosen so that a
// handler which did not run prints an unset variable's derived name --
// `ZWITNESS`, not something that reads like data.

/// The base case, and the one every other test here is a variation of.
///
/// `sigl` is asserted alongside the handler's own value because the two
/// fail independently: a trap that fires with the wrong `SIGL` looks
/// exactly like a correct one to any test that only checks the handler
/// ran. Measured, `SIGL` is the **raising** clause's line (3), not the
/// `SIGNAL ON` clause's (1) and not the handler's (5).
#[test]
fn a_signal_on_syntax_trap_runs_its_handler_and_sets_sigl_to_the_raising_clause() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzwitness = 'BEFORE'\nsay 1/0\nexit\nsyntax:\nzwitness = 'TRAPPED'\nsay zwitness sigl\n",
        ),
        b"TRAPPED 3\n".to_vec()
    );
}

/// The adjacent success for the test above: the identical program with
/// the `SIGNAL ON` clause removed is the ordinary fatal 42.3, so the
/// handler's output in that test is caused by the trap and not by
/// anything else in the program.
#[test]
fn the_same_program_without_the_trap_is_the_ordinary_fatal_condition() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"zwitness = 'BEFORE'\nsay 1/0\nexit\nsyntax:\nzwitness = 'TRAPPED'\nsay zwitness sigl\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
    assert!(
        interp.out.is_empty(),
        "the handler must not have run: {:?}",
        String::from_utf8_lossy(&interp.out)
    );
}

/// `SIGNAL OFF` really removes the trap, and the pair is what pins it to
/// `SIGNAL OFF` rather than to anything about where the raise sits.
#[test]
fn signal_off_removes_a_trap_that_signal_on_had_enabled() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nsignal on novalue\nsignal off syntax\nsay zunset\nexit\nsyntax:\nsay 'SYNTAX-HANDLER'\nexit\nnovalue:\nsay 'NOVALUE-HANDLER'\n",
        ),
        b"NOVALUE-HANDLER\n".to_vec(),
        "NOVALUE is still enabled and SYNTAX is not, so the read traps \
         and nothing reaches the SYNTAX handler"
    );

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax\nsignal off syntax\nsay 1/0\nexit\nsyntax:\nsay 'SYNTAX-HANDLER'\n",
    )
    .unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 42),
        "after SIGNAL OFF the same raise is fatal, got {failure:?}"
    );
}

/// The trap that fired is gone from the table by the time the handler
/// runs.
///
/// **A direct assertion on the table rather than on behaviour, and the
/// mutation record is why.** The behavioural pair below -- re-arm under
/// a new label, and a second raise with no re-arm -- was written first,
/// and deleting the removal left the first of the two *green*: its
/// handler re-arms before raising again, and `insert` over a live entry
/// looks exactly like `insert` over an absent one. The second test does
/// go red, but by looping until the harness kills it, which is a poor
/// signal to leave as the only one. This one fails in microseconds and
/// names the property.
#[test]
fn the_trap_that_fired_is_removed_from_the_table() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nsignal on novalue\nsay 1/0\nexit\nsyntax:\nsay 'TRAPPED'\n",
        ),
        b"TRAPPED\n".to_vec()
    );
    let traps = &interp.activation().traps;
    assert!(
        !traps.contains_key(b"SYNTAX".as_slice()),
        "the SYNTAX trap fired and must be gone"
    );
    assert!(
        traps.contains_key(b"NOVALUE".as_slice()),
        "the NOVALUE trap did not fire and must be untouched -- without \
         this half the assertion above is satisfied by clearing the whole \
         table, or by never filling it"
    );
}

/// `SIGNAL ON` inside the handler re-arms the condition, under a new
/// label: the first raise reaches `first` and the second reaches
/// `second`.
///
/// This one is about the re-arm and **not** about the removal -- see
/// `the_trap_that_fired_is_removed_from_the_table` above, which is the
/// test that actually pins that, and the note there for how this one was
/// measured not to.
#[test]
fn a_trap_can_be_re_armed_inside_its_own_handler() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax name first\nsay 1/0\nexit\nfirst:\nsay 'FIRST' sigl\nsignal on syntax name second\nsay 2/0\nexit\nsecond:\nsay 'SECOND' sigl\n",
        ),
        b"FIRST 2\nSECOND 7\n".to_vec()
    );
}

/// Without the re-arm the second raise is fatal -- the adjacent case
/// that pins the test above to "the trap was disabled" rather than to
/// "the second raise happened to reach a different label".
///
/// Against an implementation that leaves the fired trap armed this
/// program does not fail, it *loops*: the handler raises again, is
/// trapped again, and prints `FIRST` forever.
/// `the_trap_that_fired_is_removed_from_the_table` is the bounded
/// version of the same property.
#[test]
fn a_second_raise_inside_a_handler_is_fatal_without_a_re_arm() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name first\nsay 1/0\nexit\nfirst:\nsay 'FIRST'\nsay 2/0\n",
    )
    .unwrap_err();
    assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 42));
    assert_eq!(interp.out, b"FIRST\n".to_vec());
}

/// **Inherited item I11.** `Interp::failure_site` is first-wins, so a
/// second raise after a trapped first one reported the *first* site
/// until `offer_to_trap` began clearing it. The report must name line 6,
/// the second raise's own clause, and a version without the clearing
/// names line 2.
#[test]
fn a_second_raise_after_a_trapped_one_reports_its_own_site() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"signal on syntax\nsay 1/0\nexit\nsyntax:\nsay 'HANDLER'\nsay 2/0\n",
    )
    .unwrap_err();
    let mut sites = std::mem::take(&mut interp.failure_sites);
    sites.extend(interp.failure_site.take());
    let lines: Vec<usize> = sites
        .iter()
        .map(|site| site.line().expect("a clause site"))
        .collect();
    assert_eq!(
        lines,
        vec![6],
        "exactly one echo, naming the second raise's own clause -- the \
         first raise's site (line 2) must not have survived being trapped"
    );
    assert_eq!(sites[0].text(), b"say 2/0");
}

/// `SIGNAL ON NOVALUE` fires for a simple variable and for a compound,
/// and **not** for a bare stem. The third row is the one that cannot be
/// guessed: measured, `say zstem.` under the same trap prints the
/// derived name and the program carries on, where `say zstem.1` traps.
#[test]
fn novalue_fires_for_a_simple_variable_and_a_compound_but_not_a_bare_stem() {
    for (source, expected) in [
        (
            &b"signal on novalue\nsay zprobe\nexit\nnovalue:\nsay 'TRAPPED' sigl\n"[..],
            &b"TRAPPED 2\n"[..],
        ),
        (
            &b"signal on novalue\nsay zprobe.1\nexit\nnovalue:\nsay 'TRAPPED' sigl\n"[..],
            &b"TRAPPED 2\n"[..],
        ),
        (
            &b"signal on novalue\nsay zprobe.\nsay 'RESUMED'\nexit\nnovalue:\nsay 'TRAPPED'\n"[..],
            &b"ZPROBE.\nRESUMED\n"[..],
        ),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, source),
            expected.to_vec(),
            "{}",
            String::from_utf8_lossy(source)
        );
    }
}

/// An untrapped `NOVALUE` costs nothing and changes nothing -- the read
/// still yields the derived name. The neighbouring case that pins
/// `novalue_check`'s gate to "there is a trap" rather than to anything
/// about the variable.
#[test]
fn an_untrapped_novalue_still_reads_the_derived_name() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"say zprobe\nsay zprobe.1\n"),
        b"ZPROBE\nZPROBE.1\n".to_vec()
    );
}

/// A trap label that does not exist is `16.1`, reported against the
/// **raising** clause rather than against the `SIGNAL ON` clause or the
/// missing label -- so the site the raise had already recorded survives,
/// which is why `offer_to_trap` resolves the label before it clears
/// anything.
#[test]
fn a_trap_label_that_does_not_exist_is_16_1_at_the_raising_clause() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name nosuchlabel\nsay 'a'\nsay 1/0\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (16, 1));
    assert_eq!(raised.additional, vec![b"NOSUCHLABEL".to_vec()]);
    let site = interp.failure_site.expect("a site was resolved");
    assert_eq!((site.line(), site.text()), (Some(3), &b"say 1/0"[..]));
}

/// A trap is inherited by a callee and fires **there**, in the callee's
/// own activation -- which is observable only because `PROCEDURE`
/// isolates the pool: the handler reads the callee's `ZOWNER`, not the
/// caller's. `SIGL` is the callee's raising line for the same reason.
#[test]
fn a_trap_is_inherited_by_a_callee_and_fires_in_the_callees_own_activation() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzowner = 'CALLER-POOL'\ncall sub\nexit\nsub: procedure\nzowner = 'CALLEE-POOL'\nsay 1/0\nreturn\nsyntax:\nsay zowner sigl\n",
        ),
        b"CALLEE-POOL 7\n".to_vec()
    );
}

/// The other half: turning the trap off *in the callee* leaves the
/// caller's own enabled, so the condition propagates outward and is
/// trapped there instead -- with `SIGL` now the caller's `call` clause.
/// Together the two tests say the table is inherited **by copy**.
#[test]
fn a_callees_signal_off_leaves_the_callers_trap_enabled() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzowner = 'CALLER-POOL'\ncall sub\nexit\nsub: procedure\nsignal off syntax\nsay 1/0\nreturn\nsyntax:\nsay zowner sigl\n",
        ),
        b"CALLER-POOL 3\n".to_vec()
    );
}

/// **The mid-clause resumption shape, and the one route that could still
/// have created two activations within one clause.** `zz = one(1)
/// two(2)`: `one` raises, the inherited trap transfers to a handler that
/// `RETURN`s, so `one(1)` yields the handler's value and evaluation
/// resumes *inside the enclosing clause*, which then calls `two`.
///
/// `two` reports `SIGL`, which is the quantity Tasks 4 and 6 each shipped
/// a defect on: it must be the enclosing clause's line (2), not `one`'s
/// raise line (6), not the handler's (11). It is right because
/// `clause_state` lives in `ClauseState` and `Interp::invoke_call`
/// restores it whole -- the mechanism the brief asked this route to
/// test, verified rather than assumed.
#[test]
fn a_trap_that_resumes_mid_clause_leaves_the_enclosing_clauses_state_intact() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzz = one(1) two(2)\nsay 'zz=' zz\nexit\none:\nsay 1/0\nreturn 'ONEVAL'\ntwo:\nreturn 'SIGLIS' sigl\nsyntax:\nreturn 'FROMHANDLER'\n",
        ),
        b"zz= FROMHANDLER SIGLIS 2\n".to_vec()
    );
}

/// **A `CALL ON` handler runs at the clause boundary, not at the raise.**
/// `one` raises a trapped `USER` condition and returns its `RAISE ...
/// RETURN` value; the enclosing clause finishes -- `two(2)` and the
/// assignment included -- and only then does the handler run and
/// overwrite `zz`.
///
/// The `SIGL` in the handler's own output is what caught the first
/// implementation, which delivered at the next clause boundary reached by
/// *any* activation and so ran the handler inside `two`, reporting `two`'s
/// label line instead of the enclosing clause's.
#[test]
fn a_call_trap_waits_for_the_raising_clause_to_finish() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user marker name uh\nzz = one(1) two(2)\nsay 'zz=' zz\nexit\none:\nraise user marker return 'ONEVAL'\ntwo:\nreturn 'TWOVAL'\nuh:\nzz = 'HANDLER-RAN-AT' sigl\nreturn\n",
        ),
        b"zz= HANDLER-RAN-AT 2\n".to_vec(),
        "the assignment stored ONEVAL TWOVAL first, then the handler \
         overwrote it -- a handler that fired at the raise would leave \
         `zz` as the routine's own value"
    );
}

/// `RAISE`'s delivery table, the part that a two-level program cannot
/// tell apart. Each row is a three-level chain with the trap enabled in
/// exactly one place; the expected value names which handler ran.
#[test]
fn raise_delivery_depends_on_the_tail_and_on_the_condition() {
    // `RAISE SYNTAX ... RETURN` searches from the raising activation, so
    // the trap `lev2` inherited from `lev1` is the one that fires.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4 return\nmid:\nsay 'MID' sigl\nexit\n",
        ),
        b"MID 8\n".to_vec()
    );

    // The same raise with no tail skips every level but the outermost,
    // so `mid` never runs and the condition is fatal.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4\nmid:\nsay 'MID' sigl\nexit\n",
    )
    .unwrap_err();
    assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 40));
    assert!(interp.out.is_empty(), "`mid` must not have run");

    // ... and enabling it at the top as well is what catches it, with
    // `SIGL` the main body's own clause.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax name outer\ncall lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4\nmid:\nsay 'MID' sigl\nexit\nouter:\nsay 'OUTER' sigl\nexit\n",
        ),
        b"OUTER 2\n".to_vec()
    );

    // A non-`SYNTAX` condition with a `RETURN` tail searches from the
    // *caller*, so the raising routine's own inherited trap is skipped:
    // `SIGL` is the caller's clause, not the `raise` line.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on user marker\ncall sub\nexit\nsub:\nraise user marker return\nmarker:\nsay 'MARKER' sigl\nexit\n",
        ),
        b"MARKER 2\n".to_vec()
    );
}

/// An untrapped condition's default action: `HALT` reports, everything
/// else is silent. The pair is what makes `Raised::reportable`'s split
/// mean something rather than being a spelling.
#[test]
fn an_untrapped_raise_reports_for_halt_and_is_silent_for_the_rest() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"say 'a'\nraise halt\nsay 'b'\n").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (4, 1));

    for source in [
        &b"say 'a'\nraise user marker\nsay 'b'\n"[..],
        &b"say 'a'\nraise error 5\nsay 'b'\n"[..],
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, source),
            b"a\n".to_vec(),
            "{}: silent, and `b` is not reached because the tail-less \
             RAISE ends the program",
            String::from_utf8_lossy(source)
        );
    }
}

/// `RC` is the major for a trapped `SYNTAX` however it arose, the raise's
/// own argument for `ERROR`, and untouched for `NOVALUE` -- three rows
/// that no two of which share a rule.
#[test]
fn rc_is_set_from_the_condition_when_a_trap_fires() {
    for (source, expected) in [
        (
            &b"signal on syntax\nsay 1/0\nexit\nsyntax:\nsay rc\n"[..],
            &b"42\n"[..],
        ),
        (
            &b"signal on syntax\nraise syntax 40.4\nexit\nsyntax:\nsay rc\n"[..],
            &b"40\n"[..],
        ),
        (
            &b"signal on error\ncall sub\nexit\nsub:\nraise error 5 return\nerror:\nsay rc\n"[..],
            &b"5\n"[..],
        ),
        (
            &b"signal on novalue\nsay zprobe\nexit\nnovalue:\nsay rc\n"[..],
            &b"RC\n"[..],
        ),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, source),
            expected.to_vec(),
            "{}",
            String::from_utf8_lossy(source)
        );
    }
}

/// **Inherited item I16, re-verified against a real trap rather than
/// argued.** I16 concluded that `SIGNAL ON SYNTAX` cannot accumulate a
/// temps leak, resting entirely on `step_in_temps_frame` being the single
/// chokepoint that heals the `?`-skipped `pop_frame` sites in
/// `eval.rs`. The conclusion is measured here rather than inherited: two
/// hundred trap-and-resume cycles and
/// four hundred must leave the same number of live temps, and a leak of
/// even one root per cycle would make the second number two hundred
/// larger.
///
/// The raise fires from inside a parenthesised expression on purpose --
/// that is what puts `eval`'s own frame-opening sites on the path, which
/// is what the chokepoint claim is about; a raise from a bare clause
/// would exercise nothing.
#[test]
fn a_trap_that_resumes_does_not_accumulate_temps_frames() {
    fn live_temps_after(cycles: usize) -> usize {
        let source = format!(
            "signal on novalue name h\nzcount = 0\ntop:\nzcount = zcount + 1\nsay ((zprobe))\nh:\nsignal on novalue name h\nif zcount < {cycles} then signal top\nsay 'done' zcount\n"
        );
        let mut interp = Interp::new();
        let out = say_output(&mut interp, source.as_bytes());
        assert_eq!(
            out,
            format!("done {cycles}\n").into_bytes(),
            "the loop must actually have trapped {cycles} times"
        );
        interp.roots.temps_len()
    }
    assert_eq!(
        live_temps_after(200),
        live_temps_after(400),
        "a temps root retained per trapped-and-resumed cycle would make \
         the four-hundred-cycle run exactly two hundred larger"
    );
}

/// `RAISE PROPAGATE` re-raises the condition whose handler is running,
/// past every enclosing trap, and the report names the **original**
/// raising clause rather than the `raise propagate` clause -- which is
/// why the echo stack travels on `ActiveCondition` rather than being
/// dropped when the trap cleared it.
#[test]
fn raise_propagate_re_raises_the_original_condition_and_its_site() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name outer\ncall sub\nexit\nsub:\nsignal on syntax name inner\nsay 1/0\nreturn\ninner:\nraise propagate\nouter:\nsay 'OUTER-MUST-NOT-RUN'\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
    assert!(
        raised.delivery.positionless,
        "the major line drops its ` running <path> line <n>` span"
    );
    assert!(interp.out.is_empty(), "`outer` must not have run");
    let mut sites = std::mem::take(&mut interp.failure_sites);
    sites.extend(interp.failure_site.take());
    let lines: Vec<usize> = sites
        .iter()
        .map(|site| site.line().expect("a clause site"))
        .collect();
    assert_eq!(
        lines,
        vec![6, 2],
        "the original `say 1/0` clause and the `call sub` above it, not \
         the `raise propagate` clause on line 9"
    );
}

/// With no handler running, `RAISE PROPAGATE` is `98.918` -- the
/// adjacent case that pins the test above to "there was an active
/// condition" rather than to anything about `PROPAGATE` itself.
#[test]
fn raise_propagate_with_no_active_condition_is_98_918() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"say 'a'\nraise propagate\n").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 918));
    assert!(!raised.delivery.positionless);
}

/// A `CALL ON` trap does not catch a condition that has no resumption
/// point, even through `ANY` -- the one spelling that makes the question
/// askable at all, since `CALL ON SYNTAX` is a parse error. And `SIGNAL
/// ON ANY` does catch it, which is the pair that keeps this a statement
/// about `CALL` rather than about `ANY` not working.
#[test]
fn a_call_trap_declines_a_condition_with_nowhere_to_resume_but_a_signal_trap_takes_it() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call on any name uh\nsay 1/0\nexit\nuh:\nsay 'UH-MUST-NOT-RUN'\nreturn\n",
    )
    .unwrap_err();
    assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 42));
    assert!(interp.out.is_empty());

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on any\nsay 1/0\nexit\nany:\nsay 'ANY-TRAPPED' sigl\n",
        ),
        b"ANY-TRAPPED 2\n".to_vec()
    );
}

// ---- fix round 1: the pending-trap delivery boundary, and its identity ----

/// **Fix round 1's Critical, half (a).** The clause that finishes may
/// itself be the `RETURN`: `aa`'s whole body is `return bb()`, and `bb`
/// raises a trapped `USER` condition. The oracle runs the handler at that
/// clause's own boundary -- so `SIGL` is line 7, `aa`'s `return bb()` --
/// and then returns from `aa`.
///
/// Against the delivery check at the bottom of `run_activation`'s loop,
/// past a `match` whose `Flow::Return` arm returns, the handler never ran
/// at all and `ZMARK` kept its pre-set `NOMARK`. Chosen so that failure
/// prints `NOMARK` rather than the derived name `ZMARK`: a flag that is
/// merely unset reads as plausible data.
#[test]
fn a_pending_trap_is_delivered_when_the_trapping_clause_is_a_return() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ncall aa\nsay 'end mark=' zmark\nexit\naa:\nreturn bb()\nbb:\nraise user foo return 'BBVAL'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"end mark= HANDLER-AT 7\n".to_vec()
    );
}

/// **Fix round 1's Critical, half (b).** The same program with a second,
/// unrelated `call cc` after it. The handler still belongs to `aa`'s
/// `return bb()` clause (`SIGL` 8 here), and printed `HANDLER-AT 11` --
/// the `cc:` label's own line -- before the fix, because it ran inside
/// `cc`.
///
/// **Which mechanism this actually pins, measured rather than assumed.**
/// It dies when the delivery check is put back behind the `Return`/`Exit`
/// arms, and *survives* when the activation identity is degraded to a
/// stack depth -- because once the check runs at `aa`'s own `return
/// bb()` boundary, the condition is gone before `cc` is ever called, and
/// a depth is sufficient for that. So this test covers the placement, and
/// `a_pending_trap_whose_activation_is_gone_is_never_delivered` is the
/// one that covers the identity: the two mutations kill exactly one test
/// each, which is what makes them two mechanisms rather than one.
#[test]
fn a_pending_trap_is_not_delivered_into_a_later_activation_at_the_same_depth() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ncall aa\ncall cc\nsay 'end mark=' zmark\nexit\naa:\nreturn bb()\nbb:\nraise user foo return 'BBVAL'\ncc:\nsay 'in cc'\nreturn\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"in cc\nend mark= HANDLER-AT 8\n".to_vec()
    );
}

/// **A third shape of the same defect, which the review did not reach and
/// a depth cannot close.** The pending condition's activation is unwound
/// by an error its *caller* traps, so it dies without ever finishing
/// another clause; the caller then calls something else, which lands at
/// the very depth the dead activation had.
///
/// Measured: the oracle drops the condition outright -- `after mark=
/// NOMARK` -- and against a depth-keyed `PendingTrap` we printed
/// `after mark= HANDLER-AT 13`, having run the handler inside `cc`. The
/// identity check is what makes a dead activation's pending condition
/// undeliverable rather than merely unlikely to be delivered.
///
/// **This is the test that pins the identity, and the only one.** It
/// survives the mutation that reverts the delivery *placement* and dies
/// under the one that degrades `ActivationId` to a stack depth; its
/// sibling above does the reverse. The reason it can tell them apart is
/// that here the activation never reaches another clause boundary at all
/// -- the error unwinds it -- so no amount of moving the check helps, and
/// only "that activation is gone" answers it.
#[test]
fn a_pending_trap_whose_activation_is_gone_is_never_delivered() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax name sh\ncall on user foo name uh\nzmark = 'NOMARK'\ncall aa\nsay 'end mark=' zmark\nexit\naa:\nsignal off syntax\nzq = bb() + 1/0\nreturn\nbb:\nraise user foo return 'BBVAL'\ncc:\nsay 'in cc'\nreturn\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\nsh:\nsay 'SYNTAX at' sigl\ncall cc\nsay 'after mark=' zmark\nexit\n",
        ),
        b"SYNTAX at 4\nin cc\nafter mark= NOMARK\n".to_vec()
    );
}

/// **Fix round 1's finding 2.** Once a `CALL ON` handler has returned,
/// its condition is no longer active, so a later `RAISE PROPAGATE` has
/// nothing to re-raise and is `98.918`.
///
/// The report's Concern 1 called this unmeasured; one probe settled it.
/// Against the unclearing version the program was silent at rc 0.
#[test]
fn a_returned_call_handler_leaves_no_active_condition_to_propagate() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call on user foo name uh\ncall sub\nsay 'resumed'\nraise propagate\nsay 'not reached'\nexit\nsub:\nraise user foo return 'SVAL'\nuh:\nsay 'UH ran'\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 918));
    assert_eq!(interp.out, b"UH ran\nresumed\n".to_vec());
}

/// The adjacent case that keeps the clearing where it belongs: a `SIGNAL
/// ON` handler that runs *on* -- `SIGNAL`s to another label and only then
/// propagates -- must still find its condition. Measured, and it is why
/// `offer_to_trap` has no equivalent of the line above.
#[test]
fn a_signal_handler_that_runs_on_can_still_propagate() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name th\nsay 1/0\nsay 'x'\nth:\nsay 'TH ran'\nsignal onward\nonward:\nsay 'onward'\nraise propagate\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
    assert!(raised.delivery.positionless);
    assert_eq!(interp.out, b"TH ran\nonward\n".to_vec());
}

/// **4b's fix round 1, finding 3(a).** A `CALL ON` trap is held for its
/// handler's duration and released afterwards, unlike a `SIGNAL ON`
/// trap, which is removed and stays removed. `deliver_pending_traps`
/// documented this and nothing tested it: deleting the release left the
/// whole suite and the corpus gate green.
///
/// Two raises, and the second one's handler run is the assertion -- an
/// implementation that never releases the trap prints `UH 2 / mid / end`
/// and drops the second condition silently.
///
/// The release was a re-insertion until 4c Task 10 made it
/// `trap.delayed = false`. Re-measured against the new spelling: with
/// that line skipped this test fails exactly as before, and the corpus
/// drops to 48 of 49 on `lang/call_on_trap_rearms.rex`.
#[test]
fn a_call_trap_is_put_back_after_its_handler_returns() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\ncall raiser\nsay 'mid'\ncall raiser\nsay 'end'\nexit\nraiser:\nraise user foo return 'RV'\nuh:\nsay 'UH' sigl\nreturn\n",
        ),
        b"UH 2\nmid\nUH 4\nend\n".to_vec()
    );
}

/// **Fix round 1's finding 3(b).** `RAISE PROPAGATE` of a condition with
/// no catalogue entry ends the program silently rather than reporting.
/// The guard that makes that true had no test, and deleting it does not
/// merely go unnoticed -- it emits `Error 0:  <no message 0.0 in the
/// catalogue>`, which is the exact failure mode `novalue_check`'s own doc
/// comment says the design keeps unreachable.
#[test]
fn raise_propagate_of_an_unreportable_condition_ends_the_program_silently() {
    let mut interp = Interp::new();
    let ended = run_source(
        &mut interp,
        b"call on user foo name uh\ncall sub\nsay 'after'\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 'UH ran'\nraise propagate\n",
    );
    assert!(
        ended.is_ok(),
        "a USER condition has no report to give, so this ends the \
         program rather than raising: {ended:?}"
    );
    assert_eq!(
        interp.out,
        b"UH ran\n".to_vec(),
        "`say 'after'` must not run -- the propagate ends the program"
    );
    assert!(
        !interp.trace.windows(7).any(|w| w == b"Error 0"),
        "no `Error 0` placeholder may reach stderr: {:?}",
        String::from_utf8_lossy(&interp.trace)
    );
}

/// **Fix round 1's finding 5.** `RAISE SYNTAX`'s argument is validated
/// rather than used verbatim. Every row measured against the oracle; see
/// `raise_syntax_condition` for the rule and the boundary probes.
///
/// Before this, rows 5 and 6 rendered a `<no message N.M in the
/// catalogue>` placeholder to the user at rc 216, row 8 rendered an
/// unrelated catalogue entry at rc 25, and rows 7, 9 and 10 answered
/// `Error 0` at **rc 0** -- a report on stderr beside a successful exit
/// status, which is the global constraint's worst case rather than a
/// cosmetic one.
#[test]
fn raise_syntax_validates_its_argument() {
    for (argument, expected, substitution) in [
        (&b"40.4"[..], (40u16, 4u16), None),
        (&b"40"[..], (40, 0), None),
        (&b"40.001"[..], (40, 1), None),
        (&b"99"[..], (99, 0), None),
        (&b"40.10"[..], (98, 941), Some("40010")),
        (&b"3.5"[..], (98, 941), Some("3005")),
        // A major with no `(major, 0)` catalogue entry renders the
        // original `major.sub` instead of the composed number. There are
        // 45 such majors in 1..=99, not the two an earlier version of
        // this comment claimed -- `50.1` below is one of the others, and
        // is here so the row set does not accidentally describe a
        // two-element special case.
        (&b"1"[..], (98, 941), Some("1.0")),
        (&b"2.1"[..], (98, 941), Some("2.1")),
        (&b"50.1"[..], (98, 941), Some("50.1")),
        (&b"87"[..], (98, 941), Some("87.0")),
        // Fix round 2's NEW 2: the sub is bounded at 1000, and the
        // boundary is what pins it -- `40.999` is a well-formed unknown
        // code and `40.1000` is not a code at all.
        (&b"40.999"[..], (98, 941), Some("40999")),
        (&b"40.1000"[..], (33, 904), None),
        (&b"40.99999"[..], (33, 904), None),
        // Fix round 2's NEW 4: each half is a Rexx number, and a bare
        // decimal point is not one.
        (&b"'4E1'"[..], (40, 0), None),
        (&b"'40.1E2'"[..], (98, 941), Some("40100")),
        (&b"'40.'"[..], (33, 904), None),
        (&b"'+40'"[..], (40, 0), None),
        (&b"0"[..], (33, 904), None),
        (&b"100"[..], (33, 904), None),
        (&b"999"[..], (33, 904), None),
        (&b"'abc'"[..], (33, 904), None),
    ] {
        let mut source = b"raise syntax ".to_vec();
        source.extend_from_slice(argument);
        source.push(b'\n');
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, &source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("{}: expected Raised", String::from_utf8_lossy(argument));
        };
        assert_eq!(
            (raised.number, raised.sub),
            expected,
            "{}",
            String::from_utf8_lossy(argument)
        );
        if let Some(substitution) = substitution {
            assert_eq!(
                raised.additional,
                vec![substitution.as_bytes().to_vec()],
                "{}",
                String::from_utf8_lossy(argument)
            );
        }
        // The exit code is a consequence of the number, but the global
        // constraint is about the *band*: a raise must never exit 0, and
        // three of these did.
        assert_ne!(
            raised.exit_code(),
            0,
            "{}: a raise must not exit 0",
            String::from_utf8_lossy(argument)
        );
    }
}

// ---- fix round 2: the clause boundary inside nested bodies ----

/// **Fix round 2's NEW 5.** A clause inside a `DO` body reaches its own
/// boundary, not the `END`'s. Both iterations must see the handler's
/// value, and `SIGL` must be the `call sub` clause's line (4), not the
/// `say`'s.
///
/// While the delivery check lived only in `run_activation`'s loop this
/// printed `NOMARK` on both passes and `HANDLER-AT 5` once, after the
/// loop -- wrong in timing and in `SIGL`. `run_bounded` is where a loop
/// body's clauses are actually stepped.
#[test]
fn a_pending_trap_is_delivered_inside_a_do_body() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to 2\ncall sub\nsay 'after call' i 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"after call 1 mark= HANDLER-AT 4\nafter call 2 mark= HANDLER-AT 4\nend mark= HANDLER-AT 4\n".to_vec()
    );
}

/// The same property for the other two constructs `run_bounded` serves:
/// a `WHEN`'s body and an `INTERPRET` fragment. Three callers, one rule
/// -- and all three were wrong together, which is what made this a
/// mechanism rather than a `DO` quirk.
#[test]
fn a_pending_trap_is_delivered_inside_a_when_body_and_a_fragment() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\nselect\nwhen 1 = 1 then do\ncall sub\nsay 'in when mark=' zmark\nend\notherwise nop\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"in when mark= HANDLER-AT 5\nend mark= HANDLER-AT 5\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ninterpret \"call sub; say 'inside mark=' zmark\"\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"inside mark= HANDLER-AT 3\nend mark= HANDLER-AT 3\n".to_vec()
    );
}

/// **Fix round 2's NEW 1.** A `CALL ON` handler can be delivered while a
/// `SIGNAL ON` handler is already running -- one clause can queue the
/// first and raise the second -- and when it returns, the condition it
/// interrupted must come back.
///
/// `zq = sub() + 1/0` does both: `sub` queues a trapped `USER`
/// condition, then `1/0` raises a trapped `SYNTAX` one. The `SYNTAX`
/// handler ends in `raise propagate`, which must re-raise **42.3**.
/// Round 1 set `active_condition = None` when the call handler returned
/// and got `98.918`; before round 1 nothing was cleared and it was
/// silence at rc 0. Both interpreters agree on the delivery order (`UH`
/// then `SH`), so the assertion isolates to the propagate.
#[test]
fn a_call_handler_restores_the_condition_it_interrupted() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name sh\ncall on user foo name uh\nzq = sub() + 1/0\nsay 'not reached'\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 'UH ran' sigl\nreturn\nsh:\nsay 'SH ran' sigl\nraise propagate\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(
        (raised.number, raised.sub),
        (42, 3),
        "the SYNTAX condition the SIGNAL handler is running for, not the \
         USER one the CALL handler was delivered with, and not 98.918"
    );
    assert_eq!(interp.out, b"UH ran 3\nSH ran 3\n".to_vec());
}

// ---- fix round 3: a loop header is a clause too ----

/// **Fix round 3's NEW-A.** A `DO` header is a Rexx clause: its control
/// expressions run in it, and its boundary is its own, not the whole
/// loop's. `do i = 1 to sub()` must report `SIGL` 3 -- the `DO` clause --
/// and deliver the handler before the body's first pass.
///
/// Before this the handler ran after the `END`, so this test pins the
/// *boundary*. It does **not** pin the header's line: measured, a
/// mutation that leaves `header_line` at whatever was already current
/// keeps this test green, because the `DO` instruction's own
/// `step_in_temps_frame` has already set line 3. The sibling below is
/// what pins the line, since only a re-test can be on a different clause
/// from the header. Two tests, two properties, stated because the
/// mutation said so rather than assumed because they look related.
#[test]
fn a_loop_header_is_a_clause_with_its_own_boundary() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to sub()\nsay 'body' i 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 1\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"body 1 mark= HANDLER-AT 3\nend mark= HANDLER-AT 3\n".to_vec()
    );
}

/// `WHILE` and `UNTIL` are re-tested once per pass, and **which clause
/// that test belongs to changes**: it is the clause that transferred
/// control back to the loop header. See `HeaderClause` for the oracle
/// mechanism; this test carries the `DO`-then-`END` half, and
/// `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause` carries
/// the third member.
///
/// **Round 3's version of this comment said "the `DO` clause on the first
/// pass, the `END` clause on every one after", which was false**: it
/// fitted the two members probed and broke two previously-matching
/// programs, because an `ITERATE` never reaches `END` at all.
#[test]
fn a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\nzn = 0\ndo while zn < sub()\nzn = zn + 1\nsay 'body' zn 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 2\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"body 1 mark= HANDLER-AT 4\nbody 2 mark= HANDLER-AT 7\nend mark= HANDLER-AT 7\n".to_vec()
    );

    // `UNTIL` is tested only after a pass, so every one of its tests is
    // the `END` clause's -- the same rule with the first-pass case
    // absent, which is why it needs its own row rather than being
    // assumed to follow.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\nzn = 0\ndo until zn >= sub()\nzn = zn + 1\nsay 'body' zn 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 2\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"body 1 mark= NOMARK\nbody 2 mark= HANDLER-AT 7\nend mark= HANDLER-AT 7\n".to_vec()
    );
}

// ---- fix round 4: NEW-1, the third member of the re-test family ----

/// **Fix round 4's NEW-1, a regression round 3 introduced.** When an
/// `ITERATE` ends a pass, the re-test that follows belongs to the
/// `ITERATE`'s own clause -- the oracle re-enters the loop from inside
/// `RexxActivation::iterate`, so `END` is never reached and never owns
/// anything. Round 3 attributed it to `END` and turned two
/// byte-for-byte-matching programs into divergences.
///
/// Three rows, no trap in the first two, so those are a plain `SIGL`
/// question rather than a delivery one:
///
/// * the `ITERATE` row itself (oracle `2, 4, 4`; round 3 gave `2, 5, 5`);
/// * **the adjacent success**, the same loop with the `ITERATE` removed,
///   which must stay on `END` (oracle `2, 4, 4` with the `END` at 4) --
///   that is what pins the rule to "who transferred control" rather than
///   to "not `END`";
/// * an `UNTIL` loop, where the two attributions alternate within one
///   program (oracle `4, 6`), which no single-shape row can produce.
#[test]
fn a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"zn = 0\ndo i = 1 to 3 while zs() < 3\nzn = zn + 1\niterate\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
        ),
        b"2\n4\n4\n".to_vec(),
        "the DO clause on the first test, then the ITERATE's own line twice"
    );

    // The adjacent success: drop the `ITERATE` and the body falls through
    // to `END`, which is line 4 here, so the numbers coincide only
    // because `END` moved up a line -- the rule being tested is which
    // clause, not which number.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"zn = 0\ndo i = 1 to 3 while zs() < 3\nzn = zn + 1\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
        ),
        b"2\n4\n4\n".to_vec(),
        "a fall-through pass still hands the re-test to the END clause"
    );

    // `UNTIL`, where one program shows both: the first test follows an
    // `ITERATE` on line 4, the second follows a fall-through to `END` on
    // line 6.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"zn = 0\ndo until zs() >= 2\nzn = zn + 1\nif zn = 1 then iterate\nnop\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
        ),
        b"4\n6\n".to_vec(),
        "the ITERATE clause, then the END clause, in one program"
    );
}

/// `END` is **not executed** when an `ITERATE` ends a pass, so it does not
/// echo either -- the other half of NEW-1, found while measuring it.
///
/// The oracle's reason is the same one: `RexxInstructionEnd::execute` is
/// what calls `reExecute` on a fall-through, and `RexxActivation::iterate`
/// is what calls it for an `ITERATE`; `END` is jumped straight over. The
/// adjacent success is the same loop with the `ITERATE` removed, which
/// must still echo `end` once per pass.
#[test]
fn end_does_not_echo_for_a_pass_an_iterate_ended() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace r\nzn = 0\ndo while zn < 2\nzn = zn + 1\niterate\nend\n",
    )
    .unwrap();
    assert!(
        !String::from_utf8_lossy(&interp.trace).contains("*-* end"),
        "no END echo for an ITERATE-ended pass, got:\n{}",
        String::from_utf8_lossy(&interp.trace)
    );

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace r\nzn = 0\ndo while zn < 2\nzn = zn + 1\nend\n",
    )
    .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&interp.trace)
            .matches("*-* end")
            .count(),
        2,
        "the adjacent success: a fall-through pass does echo END, once per pass, got:\n{}",
        String::from_utf8_lossy(&interp.trace)
    );
}

// ---- fix round 4: NEW-2, the header clause of a nesting construct ----

/// **Fix round 4's NEW-2.** An `IF`'s condition, a `SELECT CASE`'s
/// expression and each listed `WHEN`'s condition are clauses in their own
/// right, so a `CALL ON` handler queued by one runs at *that* clause's
/// boundary -- before the branch, before the next `WHEN`, before
/// `OTHERWISE`.
///
/// Every row writes `then` on the line **after** the condition, which is
/// what separates the right answer from the wrong one: with `then` on the
/// same line, the clause that wrongly collected the boundary reports the
/// same number, and five rounds of probes never told them apart. The
/// one-line spellings are the adjacent successes in
/// `a_single_line_then_reports_the_same_line_either_way`.
#[test]
fn a_construct_header_is_a_clause_with_its_own_boundary() {
    let trap = b"call on user foo name uh\nzmark = 'NOMARK'\n";
    let handler =
        b"exit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n";
    let rows: [(&[u8], &[u8]); 4] = [
        // The `IF` clause is line 3; `then` is line 4.
        (
            b"if sub() = 'SV'\n  then say 'yes mark=' zmark\n",
            b"yes mark= HANDLER-AT 3\n",
        ),
        // A false `WHEN` on line 4, a winning one on line 5.
        (
            b"select\nwhen sub() = 'NO' then say 'first mark=' zmark\nwhen 1 = 1 then say 'second mark=' zmark\nend\n",
            b"second mark= HANDLER-AT 4\n",
        ),
        // `SELECT CASE`'s own expression, line 3.
        (
            b"select case sub()\nwhen 'SV' then say 'hit mark=' zmark\notherwise say 'oth mark=' zmark\nend\n",
            b"hit mark= HANDLER-AT 3\n",
        ),
        // A false `WHEN` on line 4 falling through to `OTHERWISE`, whose
        // body must already see the handler's value: this row is wrong in
        // timing as well as line without the fix (`NOMARK`, then a later
        // delivery at line 6).
        (
            b"select\nwhen sub() = 'NO' then say 'hit mark=' zmark\notherwise\nsay 'oth mark=' zmark\nend\n",
            b"oth mark= HANDLER-AT 4\n",
        ),
    ];
    for (body, expected) in rows {
        let mut program = trap.to_vec();
        program.extend_from_slice(body);
        program.extend_from_slice(handler);
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &program),
            expected.to_vec(),
            "header clause boundary for:\n{}",
            String::from_utf8_lossy(body)
        );
    }
}

/// The adjacent success for `a_construct_header_is_a_clause_with_its_own_
/// boundary`: with `then` on the *same* line as the condition, the right
/// answer and the wrong one coincide, and both must be the oracle's.
///
/// Kept as its own test rather than folded in, because it is the row that
/// shows why the passing cases were never evidence: **its output is the
/// same with and without the fix.** Measured -- with the per-`WHEN`
/// clause removed, this test does fail, but on `in_clause`'s tripwire
/// ("a clause at line 4 began while a condition queued by this
/// activation's clause at line 3 was still waiting"), never on a wrong
/// value. That is the tripwire earning its place: the coincidence that
/// hid the defect for four rounds is exactly the case a value assertion
/// cannot see.
#[test]
fn a_single_line_then_reports_the_same_line_either_way() {
    let trap = b"call on user foo name uh\nzmark = 'NOMARK'\n";
    let handler =
        b"exit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n";
    let rows: [(&[u8], &[u8]); 3] = [
        (
            b"if sub() = 'SV' then say 'yes mark=' zmark\n",
            b"yes mark= HANDLER-AT 3\n",
        ),
        (
            b"select\nwhen sub() = 'SV' then say 'hit mark=' zmark\nend\n",
            b"hit mark= HANDLER-AT 4\n",
        ),
        // The false `IF`, which was already right before the fix and must
        // stay right: nothing is nested inside the branch it takes.
        (
            b"if sub() = 'NO'\n  then say 'yes mark=' zmark\n  else say 'no mark=' zmark\n",
            b"no mark= HANDLER-AT 3\n",
        ),
    ];
    for (body, expected) in rows {
        let mut program = trap.to_vec();
        program.extend_from_slice(body);
        program.extend_from_slice(handler);
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &program),
            expected.to_vec(),
            "coinciding answers for:\n{}",
            String::from_utf8_lossy(body)
        );
    }
}

/// **An `INTERPRET` fragment is the one construct that must *not* end its
/// header clause before running what it nests**, and this is the test
/// that stops the NEW-2 fix being applied to it by analogy.
///
/// The oracle runs fragment text in an activation of its own
/// (`RexxActivation::interpret`) whose condition queue is separate and is
/// merged back only on the way out, so a condition queued by the
/// `INTERPRET` clause's own expression waits for that clause's boundary --
/// measured, the fragment's `say` reads the handler's variable **unset**.
#[test]
fn an_interpret_clause_does_not_deliver_before_its_fragment_runs() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ninterpret sub()\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return \"say 'frag mark=' zmark\"\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"frag mark= NOMARK\nend mark= HANDLER-AT 3\n".to_vec()
    );
}

/// **Fix round 3's NEW-B.** When the handler run at a clause's boundary
/// itself fails, the clause the report blames is the one whose boundary
/// ran it -- `call sub` -- not the enclosing `DO`.
///
/// The neighbouring case is what pins it to the boundary rather than to
/// anything about `DO`: the same program with no trap at all, failing
/// directly inside `sub`, already blamed `call sub` correctly.
#[test]
fn a_handler_that_fails_at_a_clause_boundary_blames_that_clause() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"call on user foo name uh\ndo i = 1 to 1\ncall sub\nsay 'after'\nend\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 1/0\nreturn\n",
    )
    .unwrap_err();
    let mut sites = std::mem::take(&mut interp.failure_sites);
    sites.extend(interp.failure_site.take());
    assert!(
        sites.iter().any(|site| site.text() == b"call sub"),
        "expected the `call sub` clause among the echoes, got {:?}",
        sites
            .iter()
            .map(|site| String::from_utf8_lossy(site.text()).into_owned())
            .collect::<Vec<_>>()
    );
}

// ---- builtin dispatch ----

/// A builtin reached through every call form the resolution serves, and
/// the one form that must **not** reach it.
///
/// Every line is the oracle's, measured in a clean directory:
///
/// ```text
/// say length('abc')      3            rc 0
/// say Length('abcd')     4            rc 0     (a symbol target upcases)
/// say "LENGTH"('abc')    3            rc 0     (a literal target matches verbatim)
/// say "length"('abc')    Error 43.1   rc 213   (and so does not match)
/// call length 'abc'      RESULT 3     rc 0
/// call "LENGTH" 'abc'    RESULT 3     rc 0
/// ```
///
/// The lowercase literal is the neighbouring failure that pins the
/// match to the bytes rather than to a case-insensitive compare, and it
/// is the oracle's own 43.1 here: the `::ROUTINE` step behind the builtin
/// table is what makes "matched no builtin" and "matched nothing at all"
/// the same answer for a name no directive defines.
#[test]
fn length_dispatches_from_every_call_form_that_reaches_the_builtin_table() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"say length('abc')\nsay Length('abcd')"),
        b"3\n4\n".to_vec(),
        "the expression form, at both source spellings of the symbol"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"say \"LENGTH\"('abc')"),
        b"3\n".to_vec(),
        "a literal target matches the table verbatim"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"call length 'abc'\nsay result"),
        b"3\n".to_vec(),
        "`CALL` settles RESULT from the builtin's own value"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"call \"LENGTH\" 'abc'\nsay result"),
        b"3\n".to_vec(),
        "and so does `CALL` with a literal target"
    );

    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"say \"length\"('abc')").unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 43 && raised.sub == 1),
        "a lowercase literal target matches no builtin, so it is the \
         oracle's own 43.1; got {failure:?}"
    );
}

// ---- `::ROUTINE` dispatch ----

/// Runs `source` through the real entry point, which is the only path
/// that installs directives -- this module's own `activate` pushes an
/// activation directly and never sees one.
fn routine_program(source: &[u8]) -> crate::Outcome {
    crate::run_program(
        "/tmp/routine.rex",
        source.to_vec(),
        crate::Invocation::none(),
    )
}

/// The whole resolution order in one program, and the order is what each
/// line is for rather than the dispatch: every name below resolves to
/// **something** on this crate, so a wrong order is a wrong answer and
/// not a failure.
///
/// Measured on the oracle in a clean directory, rc 0, exactly these four
/// lines. Each is a separate `::routine` that would win if the step in
/// front of it were removed:
///
/// * `call max 1, 9` -> `9`: the builtin beats `::routine max`.
/// * `call zorkolo` -> `LABEL`: the internal label beats
///   `::routine zorkolo`.
/// * `call 'ZORKOLO'` -> `ROUTINE`: a quoted target skips the label and
///   reaches the routine anyway.
/// * `call 'MAX' 1, 9` -> `9`: a quoted target does **not** skip the
///   builtin.
#[test]
fn the_resolution_order_is_label_then_builtin_then_routine() {
    let outcome = routine_program(
        b"call max 1, 9\n\
          say result\n\
          call zorkolo\n\
          say result\n\
          call 'ZORKOLO'\n\
          say result\n\
          call 'MAX' 1, 9\n\
          say result\n\
          exit\n\
          zorkolo:\n\
          return 'LABEL'\n\
          ::routine max\n\
          return 'ROUTINE-MAX'\n\
          ::routine zorkolo\n\
          return 'ROUTINE'\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"9\nLABEL\nROUTINE\n9\n".to_vec());
}

/// The routine lookup upcases both sides, where the builtin step in front
/// of it is case-sensitive -- and the second half is what makes the first
/// observable at all, since a `::routine 'max'` is only reachable because
/// `call 'max'` misses the builtin table.
///
/// Measured on the oracle, rc 0, these three lines.
#[test]
fn a_routine_lookup_upcases_both_sides_where_the_builtin_lookup_does_not() {
    let outcome = routine_program(
        b"call 'ZORK'\n\
          say result\n\
          call zork\n\
          say result\n\
          call 'max' 1, 9\n\
          say result\n\
          ::routine 'zork'\n\
          return 'HIT'\n\
          ::routine 'max'\n\
          return 'ROUTINE-lower-max'\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"HIT\nHIT\nROUTINE-lower-max\n".to_vec(),
        "an upcased-both-sides lookup finds all three"
    );
}

/// A `::ROUTINE` gets a pool of its own, and a `CALL`ed label does not --
/// the pair, because the isolating half alone passes just as well against
/// an implementation that isolates everything.
///
/// Measured on the oracle, rc 0: the routine reads the derived name `VV`
/// for a variable the caller set, its own write does not survive the
/// return, and the identical program with a label instead prints the
/// caller's value and keeps the callee's write.
#[test]
fn a_routine_has_its_own_pool_and_a_called_label_shares_the_callers() {
    let outcome = routine_program(
        b"vv = 'CALLER'\n\
          call rtn\n\
          say vv\n\
          call lbl\n\
          say vv\n\
          exit\n\
          lbl:\n\
          say vv\n\
          vv = 'LABEL-WROTE'\n\
          return\n\
          ::routine rtn\n\
          say vv\n\
          vv = 'ROUTINE-WROTE'\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"VV\nCALLER\nCALLER\nLABEL-WROTE\n".to_vec()
    );
}

/// None of the five things [`Inherited`] carries crosses into a
/// `::ROUTINE`, and the neighbouring `CALL`ed label shows each of them
/// crossing -- so this pins the difference rather than the defaults.
///
/// Measured on the oracle, rc 0, exactly the eight lines below. The
/// `NUMERIC` probe uses `FORM` as well as `DIGITS` because a caller
/// setting `form engineering` and a routine reporting `SCIENTIFIC` is the
/// only spelling of that field where inherited and defaulted differ.
#[test]
fn a_routine_inherits_none_of_the_five_a_called_label_inherits() {
    let outcome = routine_program(
        b"numeric digits 7\n\
          numeric form engineering\n\
          address system\n\
          call rtn\n\
          call lbl\n\
          exit\n\
          lbl:\n\
          say digits() form()\n\
          say address()\n\
          say trace()\n\
          return\n\
          ::routine rtn\n\
          say digits() form()\n\
          say address()\n\
          say trace()\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"9 SCIENTIFIC\nsh\nN\n7 ENGINEERING\nSYSTEM\nN\n".to_vec(),
        "the routine reports defaults and the label reports the caller's"
    );
}

/// A caller's condition trap does not arm inside a `::ROUTINE`, and the
/// probe separates "not inherited" from "the caller caught it after the
/// routine unwound" -- which every two-level program answers the same way
/// unless the routine has a label of the trap's own name.
///
/// Measured on the oracle, rc 0, `in routine` then `CALLER TRAP`: the
/// routine's own `mytrap:` never runs.
#[test]
fn a_routine_does_not_inherit_the_callers_condition_traps() {
    let outcome = routine_program(
        b"signal on syntax name mytrap\n\
          call rtn\n\
          say 'unreached'\n\
          exit\n\
          mytrap:\n\
          say 'CALLER TRAP'\n\
          exit 0\n\
          ::routine rtn\n\
          say 'in routine'\n\
          n1 = 1/0\n\
          return\n\
          mytrap:\n\
          say 'ROUTINE TRAP'\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"in routine\nCALLER TRAP\n".to_vec());
}

/// A `::ROUTINE` call sets no `SIGL` on either side.
///
/// Measured on the oracle, rc 0: the caller's own `SIGL` still reads the
/// `SIGNAL`'s line after the call returns, and `sigl` inside the routine
/// is the derived name. The `CALL`ed label beside it is the neighbouring
/// success -- it *does* set one, in the pool the two share.
#[test]
fn a_routine_call_sets_no_sigl_where_a_called_label_does() {
    let outcome = routine_program(
        b"signal there\n\
          there:\n\
          call rtn\n\
          say sigl\n\
          call lbl\n\
          say sigl\n\
          exit\n\
          lbl:\n\
          return\n\
          ::routine rtn\n\
          say sigl\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"SIGL\n1\n5\n".to_vec(),
        "the routine has none, the caller keeps line 1, the label sets 5"
    );
}

/// Two `::ROUTINE` directives of the same name refuse the program before
/// its first clause, with the oracle's own translation error.
///
/// Measured, rc 157, stdout EMPTY -- the `say` never runs, which is the
/// half a message-only assertion would miss.
#[test]
fn a_duplicate_routine_directive_is_99_903_before_the_first_clause() {
    let outcome = routine_program(
        b"say 'main ran'\n\
          ::routine zork\n\
          return 'A'\n\
          ::routine zork\n\
          return 'B'\n",
    );
    assert_eq!(outcome.exit_code, 157, "256 - 99");
    assert_eq!(outcome.stdout, b"", "stdout is empty: main never ran");
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert!(
        stderr.contains("Error 99.903:  Duplicate ::ROUTINE directive instruction.")
            && stderr.contains("*-* ::routine zork"),
        "expected the oracle's own report with the directive echoed, got: {stderr}"
    );
}

/// A program carrying a `::ROUTINE` it never calls runs exactly as one
/// with no directive does -- the adjacent success for the refusal above,
/// and the boundary Step 4's rule turns on: presence is not use.
#[test]
fn an_uncalled_routine_directive_changes_nothing() {
    let outcome = routine_program(
        b"say 'main ran'\n\
          ::routine zork\n\
          return 'A'\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"main ran\n".to_vec());
}

/// A value too large to copy twice runs to completion on every path
/// whose second copy exists only to be traced.
///
/// **The unit this asserts is "does not abort", which is not a thing a
/// test can assert from inside the process** -- an allocation failure
/// aborts rather than unwinding, so the run below would take the whole
/// test binary with it. What it asserts instead is the property the
/// guard actually adds: that no rendering happens at all when nothing
/// will print it. `Interp::intermediate_text` and `Interp::result_text`
/// return `None` under `TRACE N`, and a version of either that rendered
/// unconditionally and then threw the bytes away would satisfy every
/// other test in this file.
///
/// The abort itself is measured out of process, in this task's own
/// report and in `phase-4-exclusions.txt`'s memory row: `say
/// length(copies('a',400000000))` at the project's `ulimit -v 1048576`
/// was SIGABRT at rc 134 and is now `400000000` at rc 0, matching the
/// oracle, and peak RSS for the 500 MB case went from 978,460 kB to
/// 490,692 kB against the oracle's 496,364 kB.
#[test]
fn nothing_is_rendered_for_a_trace_line_that_will_not_print() {
    let mut interp = Interp::new();
    let program = parse_program(b"n1 = 1".to_vec()).expect("test program parses");
    activate(&mut interp, program);
    let value = interp.text(b"whatever");

    assert_eq!(interp.trace_mode().letter, b'N', "the default setting");
    assert!(interp.intermediate_text(value).is_none());
    assert!(interp.result_text(value).is_none());

    // The neighbouring settings, because "always None" would pass the
    // three lines above. `R` traces results and not intermediates, which
    // is the one mode in which the two functions disagree.
    interp.set_trace_mode(mode_from_setting(b"r").expect("R is a valid setting"));
    assert!(interp.intermediate_text(value).is_none());
    assert_eq!(interp.result_text(value).as_deref(), Some(&b"whatever"[..]));

    interp.set_trace_mode(mode_from_setting(b"i").expect("I is a valid setting"));
    assert_eq!(
        interp.intermediate_text(value).as_deref(),
        Some(&b"whatever"[..])
    );
    assert_eq!(interp.result_text(value).as_deref(), Some(&b"whatever"[..]));
}

/// `EXIT` inside a `::ROUTINE` ends the routine and settles `RESULT`,
/// where `EXIT` inside a `CALL`ed label ends the program.
///
/// **Four routes, because the exit reaches the routine boundary four
/// different ways** and only two of them travel on a `Flow`: a plain
/// `EXIT` in the routine's own body, one reached from a label *inside*
/// the routine, one reached through an expression call (which arrives as
/// `Failure::Exited`, an `Err`), and one inside an `INTERPRET`. Measured
/// on the oracle, rc 0 for all of them.
///
/// The last block is the neighbouring case that keeps this about routines
/// rather than about `EXIT`: the same `exit 5` in a `CALL`ed label ends
/// the program at rc 5.
#[test]
fn exit_inside_a_routine_ends_the_routine_where_a_labels_exit_ends_the_program() {
    let outcome = routine_program(
        b"call rtn_value\n\
          say result\n\
          call rtn_bare\n\
          say result\n\
          say rtn_expr()\n\
          call rtn_interpret\n\
          call rtn_falls_off\n\
          say result\n\
          say 'main ran on'\n\
          exit 0\n\
          ::routine rtn_value\n\
          exit 5\n\
          ::routine rtn_bare\n\
          exit\n\
          ::routine rtn_expr\n\
          n1 = inner()\n\
          say 'unreached'\n\
          return 1\n\
          inner:\n\
          exit 9\n\
          ::routine rtn_interpret\n\
          interpret \"exit 4\"\n\
          say 'unreached'\n\
          return\n\
          ::routine rtn_falls_off\n\
          n0 = 0\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"5\nRESULT\n9\nRESULT\nmain ran on\n".to_vec(),
        "a bare EXIT and falling off the end both leave RESULT unset"
    );

    let program_exit = routine_program(
        b"call sub\n\
          say 'never'\n\
          exit\n\
          sub:\n\
          exit 5\n",
    );
    assert_eq!(program_exit.exit_code, 5, "a label's EXIT ends the program");
    assert_eq!(program_exit.stdout, b"");
}

/// A `::ROUTINE`'s own clauses echo at indent **0**, however deeply the
/// call site is nested -- the same fact as `TRACE` not crossing into a
/// routine, seen from the other side.
///
/// **The exact stderr, because nothing else in the suite can see this
/// one.** `support::normalize_stderr` (DEVIATION 0) collapses the space
/// run between a trace line's marker and its content, so
/// `tests/corpus.rs` and `tests/trace_oracle.rs` compare a clause echoed
/// at indent 2 equal to one echoed at indent 0. Measured, and the
/// mutation is the caller's own `value_indent() + 2` applied to the
/// routine path as well: the whole workspace stays green with that
/// change and only this assertion goes red.
///
/// The oracle's own transcript for this program, `cat -A`'d, is what the
/// expectation below is: the routine is called from two nested `DO`
/// blocks, so an internal label reached the same way would echo at
/// indent 6.
#[test]
fn a_routines_own_clauses_echo_at_indent_zero_however_deep_the_call_site_is() {
    const PATH: &str = "/tmp/rtn-indent.rex";
    let outcome = crate::run_program(
        PATH,
        b"do 1\n\
          do 1\n\
          call rtn\n\
          end\n\
          end\n\
          ::routine rtn\n\
          trace r\n\
          n1 = 1\n\
          return\n"
            .to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        format!(
            "       >I> Routine \"RTN\" in package \"{PATH}\".\n\
             \x20    8 *-* n1 = 1\n\
             \x20      >>>   \"1\"\n\
             \x20    9 *-* return\n\
             \x20      <I< Routine \"RTN\" in package \"{PATH}\".\n"
        )
    );
}

/// `>I>`/`<I<` fire for exactly the four `TRACE` letters whose setting
/// traces labels, and for no other, when the routine's own `TRACE` is its
/// first instruction.
///
/// **All nine accepted letters, not the four that work.** Measured on the
/// oracle by running the same routine under each: `a`, `i`, `l` and `r`
/// announce both lines, and `n`, `c`, `e`, `f` and `o` produce zero
/// stderr. A gate written as "any trace setting at all" passes a
/// four-letter test and fails this one.
///
/// The bytes are asserted whole rather than by substring: seven leading
/// blanks, the prefix, one blank, the message, the trailing period
/// outside the closing quote, and no trailing whitespace.
#[test]
fn the_invocation_prefixes_fire_for_exactly_the_four_label_tracing_letters() {
    const PATH: &str = "/tmp/rtn-letters.rex";
    let announced = format!(
        "       >I> Routine \"RTN\" in package \"{PATH}\".\n\
         \x20      <I< Routine \"RTN\" in package \"{PATH}\".\n"
    );
    for letter in *b"ailr" {
        let source = format!(
            "call rtn\n::routine rtn\ntrace {}\nreturn\n",
            letter as char
        );
        let outcome = crate::run_program(PATH, source.into_bytes(), crate::Invocation::none());
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert!(
            stderr.starts_with(&announced[..announced.find('\n').unwrap() + 1]),
            "trace {}: expected the `>I>` line first, got {stderr:?}",
            letter as char
        );
        assert!(
            stderr.ends_with(&announced[announced.find('\n').unwrap() + 1..]),
            "trace {}: expected the `<I<` line last, got {stderr:?}",
            letter as char
        );
    }
    for letter in *b"ncefo" {
        let source = format!(
            "call rtn\n::routine rtn\ntrace {}\nreturn\n",
            letter as char
        );
        let outcome = crate::run_program(PATH, source.into_bytes(), crate::Invocation::none());
        assert_eq!(
            outcome.stderr, b"",
            "trace {}: this setting does not trace labels, so neither line \
             may be announced",
            letter as char
        );
    }
}

/// A `TRACE` that is not a **top-level** clause of the routine announces
/// nothing, however the construct around it nests -- and the two
/// `INTERPRET` rows that break that rule in both directions.
///
/// **The axes this crosses, and why crossing them is the point.** The
/// gate has a trace-letter axis and a where-does-the-TRACE-sit axis.
/// `the_invocation_prefixes_are_gated_on_more_than_the_trace_letter`
/// varies the second only with flat clauses in front, and
/// `..._four_label_tracing_letters` varies the first with the `TRACE`
/// always flat and first. Each holds the other axis on its safe value,
/// and the defect lived exactly at the crossing: the decay was spent by
/// `run_activation`'s top-level loop, so anything running through
/// `run_bounded`/`run_fragment` reached the `TRACE` before it fired.
/// Measured before the fix, `if 1=1 then trace l` as a routine's first
/// clause: 0 bytes of stderr on the oracle, 318 bytes here, both rc 0.
///
/// Every row is the oracle's own stderr, captured with `cat -A` so
/// trailing whitespace is visible. `n09` is here rather than only `n01`
/// because it crosses the axes the other way: the `TRACE R` still takes
/// effect for the clauses after it, so "announced nothing" has to be
/// distinguished from "did nothing".
#[test]
fn the_invocation_prefixes_are_not_announced_from_inside_a_nested_construct() {
    const PATH: &str = "/tmp/rtn-nested.rex";
    let both = format!(
        "       >I> Routine \"RTN\" in package \"{PATH}\".\n\
         \x20      <I< Routine \"RTN\" in package \"{PATH}\".\n"
    );
    let cases: &[(&str, &str, &str)] = &[
        ("an IF then-branch", "if 1=1 then trace l", ""),
        ("an IF else-branch", "if 1=0 then nop; else trace l", ""),
        ("a DO body", "do 1; trace l; end", ""),
        ("a controlled DO body", "do i = 1 to 1; trace l; end", ""),
        ("a WHEN body", "select; when 1=1 then trace l; end", ""),
        // The same nesting under a different letter: the setting takes
        // effect (the `return` echoes) and the pair is still not
        // announced, so this row separates "announced nothing" from
        // "the TRACE did nothing".
        (
            "an IF then-branch, TRACE R",
            "if 1=1 then trace r",
            "     5 *-* return\n",
        ),
        // A fragment counts its own clauses from zero, so a `TRACE`
        // that is the fragment's own first clause DOES announce...
        ("an INTERPRET", "interpret \"trace l\"", "BOTH"),
        // ...and everything that breaks one of the fragment rule's three
        // conditions does not.
        (
            "an INTERPRET, second fragment clause",
            "interpret \"nop; trace l\"",
            "",
        ),
        (
            "an INTERPRET that is not the routine's first clause",
            "n0 = 0\ninterpret \"trace l\"",
            "",
        ),
        (
            "an INTERPRET inside an IF",
            "if 1=1 then interpret \"trace l\"",
            "",
        ),
        (
            "an INTERPRET inside an INTERPRET",
            "interpret \"interpret 'trace l'\"",
            "",
        ),
        // The neighbouring announced case, so the eleven silences above
        // are pinned to the nesting and not to something else about
        // these programs.
        ("a flat first clause", "trace l", "BOTH"),
    ];
    for (what, body, want) in cases {
        let source = format!("call rtn\nsay 'after'\n::routine rtn\n{body}\nreturn\n");
        let outcome = crate::run_program(PATH, source.into_bytes(), crate::Invocation::none());
        let want = if *want == "BOTH" { both.as_str() } else { want };
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            want,
            "{what}: stderr"
        );
        assert_eq!(outcome.stdout, b"after\n".to_vec(), "{what}: stdout");
    }
}

/// The five things other than the letter that decide whether the pair is
/// announced, each with the neighbouring case that is announced.
///
/// Every source below was measured on the oracle, rc 0. They are one test
/// because each is the *same* program differing in one clause, and
/// splitting them would hide that: the announced case is what says the
/// difference is the clause and not something else about the program.
#[test]
fn the_invocation_prefixes_are_gated_on_more_than_the_trace_letter() {
    const PATH: &str = "/tmp/rtn-gate.rex";
    let entry = format!("       >I> Routine \"RTN\" in package \"{PATH}\".\n");
    let exit = format!("       <I< Routine \"RTN\" in package \"{PATH}\".\n");
    let both = format!("{entry}{exit}");

    let check = |what: &str, source: &str, want: &str| {
        let outcome =
            crate::run_program(PATH, source.as_bytes().to_vec(), crate::Invocation::none());
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            want,
            "{what}: stderr"
        );
    };

    // The routine's `TRACE` must be its FIRST instruction, and a LABEL
    // spends that permission where `PROCEDURE`'s own permission survives
    // one.
    check(
        "trace l first",
        "call rtn\n::routine rtn\ntrace l\nreturn\n",
        &both,
    );
    check(
        "an assignment in front of it",
        "call rtn\n::routine rtn\nn0 = 0\ntrace l\nreturn\n",
        "",
    );
    check(
        "a label in front of it",
        "call rtn\n::routine rtn\nlbl:\ntrace l\nreturn\n",
        "",
    );

    // A comment is not an instruction, so it does not spend it.
    check(
        "a comment in front of it",
        "call rtn\n::routine rtn\n/* c */\ntrace l\nreturn\n",
        &both,
    );

    // The dynamic form reaches the same `setTrace` route.
    check(
        "trace value 'l'",
        "call rtn\n::routine rtn\ntrace value 'l'\nreturn\n",
        &both,
    );

    // `<I<` re-reads the setting, so a routine that turns tracing off
    // announces its entry and not its exit.
    check(
        "trace l then trace off",
        "call rtn\n::routine rtn\ntrace l\ntrace off\nreturn\n",
        &entry,
    );

    // The caller's setting never crosses, and a main body never
    // announces one however it is traced.
    check(
        "trace l in the caller",
        "trace l\ncall rtn\n::routine rtn\nn0 = 0\nreturn\n",
        "",
    );
    check("trace l in a main body alone", "trace l\nn0 = 0\n", "");

    // Once per activation, so two calls announce two pairs.
    check(
        "two calls",
        "call rtn\ncall rtn\n::routine rtn\ntrace l\nreturn\n",
        &format!("{both}{both}"),
    );
}

/// Every directive form whose installation this crate **can** perform
/// leaves the program running byte for byte as the oracle runs it, one
/// program per form.
///
/// **This is the half that stops "any directive is a gap".** Each source
/// below was measured on the oracle in a clean directory: rc 0, stdout
/// `main ran`, stderr empty. A version of `directive_gap` that refused on
/// presence passes every refusal test in this file and fails all seven of
/// these.
#[test]
fn every_directive_this_crate_can_install_leaves_the_program_alone() {
    let sources: &[(&str, &[u8])] = &[
        ("a bare ::CLASS", b"say 'main ran'\n::class foo\n"),
        (
            "::CLASS with a ::METHOD",
            b"say 'main ran'\n::class foo\n::method bar\nreturn 1\n",
        ),
        (
            "::CLASS with a ::ATTRIBUTE",
            b"say 'main ran'\n::class foo\n::attribute baz\n",
        ),
        (
            "a loose ::METHOD with no ::CLASS",
            b"say 'main ran'\n::method loose\nreturn 1\n",
        ),
        ("::CONSTANT", b"say 'main ran'\n::constant kk 5\n"),
        (
            "::RESOURCE",
            b"say 'main ran'\n::resource foo\nsome text\n::END\n",
        ),
        (
            "::ANNOTATE PACKAGE",
            b"say 'main ran'\n::annotate package author 'me'\n",
        ),
    ];
    for (what, source) in sources {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            0,
            "{what}: stderr {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(outcome.stdout, b"main ran\n".to_vec(), "{what}: stdout");
        assert!(outcome.stderr.is_empty(), "{what}: stderr must be empty");
    }
}

/// Every directive form whose installation this crate **cannot** perform
/// refuses the program before its first clause, naming the owning phase.
///
/// Two things are asserted per form and both matter. The owner suffix is
/// what `corpus.rs` and `keyword-exempt.txt` read to attribute a failure.
/// The **empty stdout** is what says the refusal happened at install
/// rather than at the call: every source below has `say 'main ran'` as
/// its first clause, and the oracle prints nothing for the five it also
/// refuses.
///
/// The `::REQUIRES` row is not the over-refusal case:
/// no `helper.rex` sits beside these programs, so the oracle refuses that
/// source too, and the over-refusal (a helper that *is* present) is
/// measured in the exclusions file instead -- this crate never opens the
/// file, so the two sources are one case to it.
#[test]
fn every_directive_this_crate_cannot_install_refuses_before_the_first_clause() {
    let cases: &[(&[u8], &str)] = &[
        (
            b"say 'main ran'\n::class foo mixinclass zzznotaclass\n",
            "::CLASS MIXINCLASS is not implemented (Phase 5)",
        ),
        (
            b"say 'main ran'\n::class foo metaclass zzznotaclass\n",
            "::CLASS METACLASS is not implemented (Phase 5)",
        ),
        (
            b"say 'main ran'\n::class bar inherit zzznotaclass\n",
            "::CLASS INHERIT is not implemented (Phase 5)",
        ),
        (
            b"say 'main ran'\n::class foo subclass ns:other\n",
            "::CLASS SUBCLASS naming a namespace is not implemented (Phase 5)",
        ),
        (
            b"say 'main ran'\n::requires 'helper.rex'\n",
            "::REQUIRES is not implemented (Phase 5)",
        ),
        (
            b"say 'main ran'\n::options digits 12\n",
            "::OPTIONS is not implemented (Phase 5)",
        ),
        (
            b"say 'main ran'\n::annotate routine nosuchrtn\n",
            "::ANNOTATE naming a target is not implemented (Phase 5)",
        ),
        (
            b"say 'main ran'\n::routine z external \"LIBRARY nosuchlib nosuchfn\"\n",
            "::ROUTINE EXTERNAL is not implemented (Phase 7)",
        ),
        (
            b"say 'main ran'\n::class foo\n::method m external \"LIBRARY nosuchlib nosuchfn\"\n",
            "::METHOD EXTERNAL is not implemented (Phase 7)",
        ),
    ];
    for (source, message) in cases {
        let outcome = routine_program(source);
        assert_eq!(
            outcome.exit_code,
            crate::NOT_IMPLEMENTED_EXIT,
            "{message}: exit code"
        );
        assert_eq!(
            outcome.stdout, b"",
            "{message}: stdout must be empty -- main must not have run"
        );
        assert_eq!(
            outcome.stderr,
            format!("rexx-exec: {message}\n").into_bytes(),
            "{message}: stderr"
        );
    }
}

/// **A gap the oracle diagnoses before it creates any class refuses before
/// this crate creates one either**, so a `::CLASS` that cannot install does
/// not answer in its place.
///
/// Every source below pairs one gap form with `::class a subclass
/// zzznotaclass`, in each order. That `::CLASS` raises 98.909, and since
/// `518cd6de7` this crate installs the file's classes ahead of the
/// source-order pass that reads `directive_gap` -- so without `staged_gap`
/// the class error is what a reader of these programs gets. The oracle's own
/// answer, measured in a clean directory, is the same in either order:
///
/// ```text
/// ::annotate routine nosuchrtn     99.945 rc 157, echoing the ::ANNOTATE line
/// ::requires 'zzznosuchfile.rex'   43.901 rc 213, echoing the ::REQUIRES line
/// ::options digits 12              98.909 rc 158, echoing the ::CLASS line
/// ```
///
/// So the `::ANNOTATE` and `::REQUIRES` sources must refuse and the
/// `::OPTIONS` ones must reach the class error, and the halves fail in
/// opposite directions: dropping `staged_gap` reddens the refusing rows,
/// and hoisting the whole gap check to the front instead reddens the
/// `::OPTIONS` rows. One half alone does not say where the boundary is.
///
/// **A refusal here is not a match** -- the oracle answers 99.945 and 43.901
/// and this crate answers neither. It is the honest half of the trade
/// `staged_gap`'s doc states, and the corpus differential cannot see any of
/// it, because no program in the subset carries a `::REQUIRES` or an
/// `::ANNOTATE` with a target.
#[test]
fn a_gap_the_oracle_diagnoses_before_a_class_refuses_ahead_of_the_class_error() {
    let failing_class = "::class a subclass zzznotaclass\n";
    let refusing: &[(&str, &str)] = &[
        (
            "::annotate routine nosuchrtn\n",
            "::ANNOTATE naming a target is not implemented (Phase 5)",
        ),
        (
            "::requires 'zzznosuchfile.rex'\n",
            "::REQUIRES is not implemented (Phase 5)",
        ),
    ];
    for (gap, message) in refusing {
        for source in [
            format!("say 'main ran'\n{gap}{failing_class}"),
            format!("say 'main ran'\n{failing_class}{gap}"),
        ] {
            let outcome = routine_program(source.as_bytes());
            assert_eq!(
                outcome.exit_code,
                crate::NOT_IMPLEMENTED_EXIT,
                "{source}: exit code, stderr {}",
                String::from_utf8_lossy(&outcome.stderr)
            );
            assert_eq!(outcome.stdout, b"", "{source}: stdout");
            assert_eq!(
                outcome.stderr,
                format!("rexx-exec: {message}\n").into_bytes(),
                "{source}: stderr"
            );
        }
    }
    for source in [
        format!("say 'main ran'\n::options digits 12\n{failing_class}"),
        format!("say 'main ran'\n{failing_class}::options digits 12\n"),
    ] {
        let outcome = routine_program(source.as_bytes());
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert_eq!(
            outcome.exit_code, 158,
            "{source}: exit code, stderr {stderr}"
        );
        assert_eq!(outcome.stdout, b"", "{source}: stdout");
        assert!(
            stderr.contains("Error 98.909:  Class \"ZZZNOTACLASS\" not found."),
            "{source}: stderr {stderr}"
        );
    }
}

/// A builtin's result is a value whose rendering `NUMERIC DIGITS` cannot
/// reach, and D15 is still visible on it from the other side.
///
/// The whole program below is the oracle's, measured in a clean
/// directory, rc 0, printing `16`, `2E+1`, `10`. Each line is doing
/// separate work:
///
/// * `nn` is built under `DIGITS 3` and read back under `DIGITS 1`, which
///   is the change D15 says a probe needs before it can see anything at
///   all. It still prints `16`.
/// * `nn + 0` under `DIGITS 1` prints `2E+1`, because the addition is a
///   new operation creating a new number under the digits then in force.
///   Without this line the first would also pass against a value that had
///   simply captured `DIGITS 3`.
/// * `length('abcdefghij')` under `DIGITS 1` prints `10` and not `1E+1`,
///   which is what rules out creating the result through
///   `Interp::number` with the current settings.
#[test]
fn a_builtins_result_renders_independently_of_the_digits_in_force() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 3\nnn = length('abcdefghijklmnop')\nnumeric digits 1\n\
              say nn\nsay nn + 0\nsay length('abcdefghij')"
        ),
        b"16\n2E+1\n10\n".to_vec()
    );
}

/// `say length()` produces the oracle's own 40.3 report, byte for byte.
///
/// Measured in a clean directory, rc 216:
///
/// ```text
///      1 *-* say length()
/// Error 40 running /.../p09.rex line 1:  Incorrect call to routine.
/// Error 40.3:  Not enough arguments in invocation of LENGTH; minimum expected is 1.
/// ```
///
/// **Driven through `run_program`**, because the bytes under test are the
/// whole report -- the clause echo, the major line's path and line number,
/// and the secondary line's two substitutions -- and only the real entry
/// point assembles all three.
#[test]
fn a_builtin_called_with_too_few_arguments_reports_the_oracles_40_3() {
    let path = "/tmp/length-arity.rex";
    let outcome = crate::run_program(path, b"say length()\n".to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 216, "256 - 40");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        format!(
            "     1 *-* say length()\n\
             Error 40 running {path} line 1:  Incorrect call to routine.\n\
             Error 40.3:  Not enough arguments in invocation of LENGTH; \
             minimum expected is 1.\n"
        )
    );
}

// ---- ADDRESS, the environment-naming forms ----
//
// Every assertion below reads the activation's own state rather than a
// program's output. That was once forced -- the environment has exactly
// two readers a Rexx program can use, `ADDRESS()` and issuing a command
// -- and is now a division of labour: `corpus/lang/address_env.rex`
// asserts the same properties through `ADDRESS()`, against the oracle,
// and these read the pair directly so a wrong *alternate* is visible
// without a toggle to expose it.

/// The running activation's `(current, alternate)` pair as text, `None`
/// staying `None` because it is not a name -- it is "the platform's
/// default environment", which this crate does not yet have a spelling
/// for.
fn address_pair(interp: &Interp) -> (Option<String>, Option<String>) {
    let show = |name: &Option<Rc<[u8]>>| {
        name.as_ref()
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
    };
    let state = &interp.activation().address;
    (show(&state.current), show(&state.alternate))
}

/// Measured on the oracle through `ADDRESS()`, four spellings of one
/// intent: `address envC` -> `ENVC`, `address 'LiTeRaL'` -> `LiTeRaL`,
/// and both computed forms -> the value unchanged.
#[test]
fn only_the_symbol_form_upcases_the_environment_name() {
    for (source, want) in [
        (&b"address envC\n"[..], "ENVC"),
        (&b"address 'LiTeRaL'\n"[..], "LiTeRaL"),
        (&b"nm = 'mIxEd'\naddress value nm\n"[..], "mIxEd"),
        (&b"nm = 'mIxEd'\naddress (nm)\n"[..], "mIxEd"),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).expect("test program runs");
        assert_eq!(
            address_pair(&interp).0.as_deref(),
            Some(want),
            "{}",
            String::from_utf8_lossy(source)
        );
    }
}

/// Bare `ADDRESS` swaps the pair, forever.
///
/// **Four consecutive toggles, because one is not enough to tell a swap
/// from a pop.** Measured on the oracle through `ADDRESS()`: `ENVB`,
/// `ENVA`, `ENVB`, `ENVA`, `ENVB` for zero through four bare `ADDRESS`es
/// after `envA` and `envB`. A pop-stack implementation agrees on the
/// first toggle and then walks backwards out of the pair instead of
/// returning into it -- measured by mutation, replacing the swap with
/// `current = alternate.take()`: the alternate is already wrong after one
/// toggle and the current after two.
///
/// Both halves are asserted rather than only the current one, and that is
/// what makes the first row catch anything at all.
#[test]
fn bare_address_is_a_toggle_and_not_a_stack() {
    for (toggles, current, alternate) in [
        (0, "ENVB", "ENVA"),
        (1, "ENVA", "ENVB"),
        (2, "ENVB", "ENVA"),
        (3, "ENVA", "ENVB"),
        (4, "ENVB", "ENVA"),
    ] {
        let mut source = b"address envA\naddress envB\n".to_vec();
        for _ in 0..toggles {
            source.extend_from_slice(b"address\n");
        }
        let mut interp = Interp::new();
        run_source(&mut interp, &source).expect("test program runs");
        assert_eq!(
            address_pair(&interp),
            (Some(current.to_string()), Some(alternate.to_string())),
            "after {toggles} bare ADDRESS"
        );
    }
}

/// A bare `ADDRESS` before any other `ADDRESS` changes nothing, and the
/// default is a name the pair can toggle back **to**.
///
/// Measured on the oracle, the one edge every earlier probe missed by
/// setting an environment first: `say address()` reports `sh` before and
/// after each of three bare `ADDRESS`es.
///
/// **The second half is what makes this discriminate anything.** The first
/// assertion alone is satisfied by a `toggle` that does nothing at all,
/// and only a change to how the default is represented could redden it.
/// The second is the failure this edge actually invites: `None` is doing
/// two jobs in an `Option`, "the platform default" and "absent", and an
/// implementation that reads it as the second guards its toggle with `if
/// let Some(previous) = self.alternate.take()` and then refuses to leave
/// `ENVA`. Measured against the oracle, which does leave it: `address
/// envA` then bare `ADDRESS` reports `sh`.
/// One `Interp` per row, and a row's whole program in one string: each
/// `run_source` pushes a **fresh** activation, so splitting a sequence
/// across two calls would silently start the second half from the default
/// again -- which is how an earlier version of this test appeared to pass
/// its middle assertion for the wrong reason.
#[test]
fn a_bare_address_with_no_prior_environment_changes_nothing() {
    const LEAD: &str = "address\naddress\naddress\n";
    for (tail, current, alternate, what) in [
        (
            "",
            None,
            None,
            "three bare ADDRESSes before any other ADDRESS",
        ),
        (
            "address envA\n",
            Some("ENVA"),
            None,
            "the default is still what the first set displaces",
        ),
        (
            "address envA\naddress\n",
            None,
            Some("ENVA"),
            "a bare ADDRESS must toggle back to the default, not decline \
             to move because the alternate is spelled None",
        ),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, format!("{LEAD}{tail}").as_bytes()).expect("test program runs");
        assert_eq!(
            address_pair(&interp),
            (current.map(str::to_string), alternate.map(str::to_string)),
            "{what}"
        );
    }
}

/// Setting the same name twice leaves the toggle with nothing to swap:
/// `set` discards the *old* alternate rather than keeping a history.
/// Measured on the oracle, `address envA; address envA` then three bare
/// `ADDRESS`es, all five `ADDRESS()` readings `ENVA`.
#[test]
fn setting_the_same_environment_twice_leaves_the_toggle_nothing_to_do() {
    for toggles in 0..=3 {
        let mut source = b"address envA\naddress envA\n".to_vec();
        for _ in 0..toggles {
            source.extend_from_slice(b"address\n");
        }
        let mut interp = Interp::new();
        run_source(&mut interp, &source).expect("test program runs");
        assert_eq!(
            address_pair(&interp),
            (Some("ENVA".to_string()), Some("ENVA".to_string())),
            "after {toggles} bare ADDRESS"
        );
    }
}

/// The environment is **per activation**: a callee's own `ADDRESS` dies
/// with it, and so does its own toggle.
///
/// Measured on the oracle: `address outer` in the caller, `address inner`
/// in the callee, and the caller still reports `OUTER` after the `RETURN`,
/// then `sh` after a bare `ADDRESS` -- so neither half of the pair came
/// back. An implementation holding one pair on `Interp` instead of one per
/// activation reads `INNER` here.
///
/// **The other direction cannot be asserted here, and is asserted in the
/// corpus instead.** A callee does inherit the caller's pair, both
/// halves, and `Activation::address`' own doc has the oracle transcript
/// -- but a callee never writes anything back and `Interp::invoke_call`
/// pops it unconditionally on both paths, so no in-crate test can read a
/// callee's own state. `corpus/lang/address_env.rex`'s E block reads it
/// from inside the callee with `ADDRESS()`, and uses two named
/// environments rather than the default for both halves, because with an
/// unset alternate "inherited the caller's pair" and "started from the
/// default" print the same bytes.
#[test]
fn a_callees_own_environment_does_not_survive_the_return() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"address outer\ncall sub\nexit\nsub:\naddress inner\naddress\nreturn\n",
    )
    .expect("test program runs");
    assert_eq!(address_pair(&interp), (Some("OUTER".to_string()), None));
}

/// 250 bytes is accepted and 251 raises 29.1, on both setting forms.
///
/// Measured on the oracle, rc 227 for each of the two long spellings, and
/// `ADDRESS()` reporting length 250 for the accepted one. The substitution
/// carries the whole name back untruncated -- 251 bytes in, 251 quoted.
#[test]
fn an_environment_name_over_two_hundred_and_fifty_bytes_raises_29_1() {
    for length in [250usize, 251] {
        let name = "z".repeat(length);
        for source in [
            format!("address '{name}'\n"),
            format!("nm = '{name}'\naddress value nm\n"),
        ] {
            let mut interp = Interp::new();
            let outcome = run_source(&mut interp, source.as_bytes());
            if length == 250 {
                outcome.expect("a 250-byte name is accepted");
                assert_eq!(address_pair(&interp).0.as_deref(), Some(name.as_str()));
                continue;
            }
            let Err(Failure::Raised(raised)) = outcome else {
                panic!("expected 29.1 for a {length}-byte name, got {outcome:?}");
            };
            assert_eq!((raised.number, raised.sub), (29, 1));
            assert_eq!(
                raised.additional,
                vec![b"250".to_vec(), name.clone().into_bytes()],
                "the limit and the whole rejected name, untruncated"
            );
            assert_eq!(
                address_pair(&interp),
                (None, None),
                "a rejected name must not have been installed"
            );
        }
    }
}

/// The command form and the `WITH` form both still fail loudly, and both
/// name Phase 7.
///
/// `tests/loud.rs` carries one witness for the `Address::Command` tag and
/// the tag covers two shapes, so the shape that witness does not spell is
/// pinned here. The third row is the one that is easy to miss: a `WITH`
/// with **no** command still configures a command's streams, so it is the
/// same owner even though it only names an environment.
#[test]
fn the_command_and_with_forms_stay_loud_and_name_phase_7() {
    for source in [
        &b"address cmd ''\n"[..],
        &b"address cmd 'text'\n"[..],
        &b"address cmd with output stem o.\n"[..],
        &b"address with output stem o.\n"[..],
        &b"address value 'q' with input stem i.\n"[..],
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Loud(loud) = failure else {
            panic!(
                "expected Loud for {:?}, got {failure:?}",
                String::from_utf8_lossy(source)
            );
        };
        assert_eq!(
            loud.message,
            "ADDRESS is not implemented (Phase 7)",
            "{}",
            String::from_utf8_lossy(source)
        );
    }
}

/// [`Interp::chunk_node_at`]'s steps land on the node the path names, and
/// answer `None` for a step that has nowhere to go.
///
/// **The descent tested at its own steps rather than through a compiled
/// op.** A path resolves the same whether or not anything emits one, so
/// the arms below are reachable here without a program that compiles to a
/// non-[`NodePath::ROOT`] address existing to reach them.
///
/// `zz = -za + zb * f(1)` is asymmetric at both levels on purpose. The
/// **branch** is pinned by the two-step variables: `ZA` hangs off the
/// prefix and `ZB` off the multiplication, so a step down the wrong child
/// lands on the other name rather than on something that merely looks
/// alike. The **order** is pinned by `ZB` alone, whose address is `[right,
/// left]`: read innermost first that is `[left, right]`, the `right` step
/// off a prefix, which has nowhere to go -- so a descent that walked the
/// steps backwards finds nothing where this finds `ZB`.
#[test]
fn a_paths_steps_land_on_the_node_it_names() {
    let program = parse_program(b"zz = -za + zb * f(1)".to_vec()).expect("test program parses");
    let instruction = &program.main.instructions[0];
    let node = |path| Interp::chunk_node_at(instruction, 0, path);
    let variable_at = |path| match node(path).map(|node| &node.kind) {
        Some(ExprKind::Variable(id)) => program.symbols.name(*id).to_string(),
        found => panic!("expected a variable at this address, found {found:?}"),
    };

    let root = node(NodePath::ROOT).expect("slot 0 is the assignment's value");
    assert!(matches!(root.kind, ExprKind::Binary { .. }));

    let left = NodePath::ROOT.child(false).expect("one step fits");
    let right = NodePath::ROOT.child(true).expect("one step fits");
    assert!(matches!(
        node(left).expect("the left operand is there").kind,
        ExprKind::Prefix { .. }
    ));
    assert!(matches!(
        node(right).expect("the right operand is there").kind,
        ExprKind::Binary { .. }
    ));

    assert_eq!(variable_at(left.child(false).expect("two steps fit")), "ZA");
    assert_eq!(
        variable_at(right.child(false).expect("two steps fit")),
        "ZB"
    );
    assert!(matches!(
        node(right.child(true).expect("two steps fit"))
            .expect("the call is the multiplication's right operand")
            .kind,
        ExprKind::Call { .. }
    ));

    // A prefix has one child and it is the `false` step, so the `true` one
    // has nowhere to go; a call has no children at all; and slot 1 is not
    // an address this names.
    assert!(node(left.child(true).expect("two steps fit")).is_none());
    let past_the_call = right
        .child(true)
        .and_then(|path| path.child(false))
        .expect("three steps fit");
    assert!(node(past_the_call).is_none());
    assert!(Interp::chunk_node_at(instruction, 1, NodePath::ROOT).is_none());
}

/// **The two message-send indents, pinned here because nothing else in the
/// tree can pin them**, for the reason
/// [`task_9s_two_new_indents_are_the_oracles_own_and_normalisation_cannot_see_them`]
/// gives: `tests/trace_oracle.rs` and `tests/corpus.rs` both compare through
/// DEVIATION 0's `normalize_stderr`, which collapses exactly the space run
/// these two lines differ in. A unit test's own `assert_eq!` is outside
/// either comparison function.
///
/// Every byte below is the oracle's, captured with `cat -A` from the two
/// programs run verbatim.
///
/// * `>M>` sits at the **sending clause's** indent, like every other value
///   line: two further in inside one `DO`.
/// * a failing native method's `Compiled method ... with scope ...` traceback
///   line carries **no** indent at all, even for a send two `DO` levels
///   deep -- the catalogue entry supplies its own leading blanks and the
///   send's nesting does not move it.
#[test]
fn a_message_sends_two_indents_are_the_oracles_own_and_normalisation_cannot_see_them() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace i\ndo ii = 1 to 1\n  say 'abc'~length\nend\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* do ii = 1 to 1\n",
            "       >L>   \"1\"\n",
            "       >L>   \"1\"\n",
            "       >K>   \"TO\" => \"1\"\n",
            "       >=>   II <= \"1\"\n",
            "     3 *-*   say 'abc'~length\n",
            "       >L>     \"abc\"\n",
            "       >M>     \"LENGTH\" => \"3\"\n",
            "       >>>     \"3\"\n",
            "     4 *-* end\n",
            "     2 *-* do ii = 1 to 1\n",
            "       >V>     II => \"1\"\n",
            "       >>>     \"1\"\n",
            "       >>>     \"2\"\n",
            "       >=>     II <= \"2\"\n",
        )
    );

    let outcome = crate::run_program(
        "/abs/nested.rex",
        b"do ii = 1 to 1\n  do jj = 1 to 1\n    say 'abc'~length(1)\n  end\nend\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(
        String::from_utf8(outcome.stderr).expect("the report is UTF-8"),
        concat!(
            "       *-* Compiled method \"LENGTH\" with scope \"String\".\n",
            "     3 *-*     say 'abc'~length(1)\n",
            "Error 93 running /abs/nested.rex line 3:  Incorrect call to method.\n",
            "Error 93.902:  Too many arguments in invocation of method; 0 expected.\n",
        )
    );
}

/// The corpus program `rel` reads as a source string, so that a trace
/// expectation and the program it was captured from cannot drift apart:
/// editing the `.rex` file changes what this test runs, and the assertion
/// below then names the line it no longer matches.
macro_rules! corpus_source {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/", $rel))
    };
}

/// Runs `source` under both engines at `path` and asserts stderr is exactly
/// `expected` on each.
///
/// Both arms rather than one, and it is not redundant with `tests/ir_dual.rs`
/// -- that harness compares the two engines against *each other*, and both
/// format trace through one `crate::trace`, so an indent that is wrong is
/// wrong identically on both and the comparison stays green. What this
/// function adds is a third party: bytes the oracle produced.
fn assert_stderr_on_both_engines(path: &str, source: &str, expected: &str) {
    for engine in [crate::Engine::Ir, crate::Engine::TreeWalker] {
        let outcome = crate::run_program(
            path,
            source.as_bytes().to_vec(),
            crate::Invocation::none().with_engine(engine),
        );
        assert_eq!(
            String::from_utf8(outcome.stderr).expect("the trace is UTF-8"),
            expected,
            "{engine:?} arm"
        );
    }
}

/// **A `::METHOD` activation's own trace indents, pinned here because
/// nothing else in the tree can pin them.**
///
/// `tests/corpus.rs` does compare these two programs' stderr raw, which is
/// what `RAW_STDERR_COMPARISON` was built for -- but that comparison needs
/// the C++ oracle on the machine and the corpus gate switched on, and this
/// one runs in every `cargo test`. `tests/trace_oracle.rs` cannot substitute
/// for either: it compares through DEVIATION 0's `normalize_stderr`, which
/// collapses exactly the space run these lines differ in.
///
/// Every byte below is the oracle's, captured with `cat -A` from the corpus
/// program named beside it, run verbatim from a scratch directory. The **one**
/// substitution is the package path, which is the file's own absolute
/// location: the capture's scratch path is replaced by the path this test
/// hands `run_program`, and nothing else is retyped.
///
/// What the shape says, and what a plausible wrong implementation gets wrong:
///
/// * the callee's clause echoes are at indent **0** while the sending clause
///   sits at 2 -- a method activation inherits no indent, exactly as a
///   `::ROUTINE` does not;
/// * so are its own value lines: `>E>   .K` inside `OUTER` against
///   `>E>     .K` in the caller;
/// * `>I>` and `<I<` take no indent at all, at any depth;
/// * `>M>` and the enclosing `>>>` are back at the **sending** clause's
///   indent once the activation has ended, which is what shows the level
///   state was restored rather than left at the callee's.
#[test]
fn a_method_activations_trace_indents_are_the_oracles_own_and_normalisation_cannot_see_them() {
    assert_stderr_on_both_engines(
        "/abs/method_trace_invocation.rex",
        corpus_source!("lang/method_trace_invocation.rex"),
        concat!(
            "     2 *-* say .K~m(3)\n",
            "       >E>   .K => \"The K class\"\n",
            "       >L>   \"3\"\n",
            "       >A>   \"3\"\n",
            "       >I> Method \"M\" with scope \"K\" in package \"/abs/method_trace_invocation.rex\".\n",
            "     9 *-* use arg n\n",
            "       >>>   \"3\"\n",
            "       >=>   N <= \"3\"\n",
            "    10 *-* return n + 1\n",
            "       >V>   N => \"3\"\n",
            "       >L>   \"1\"\n",
            "       >O>   \"+\" => \"4\"\n",
            "       >>>   \"4\"\n",
            "       <I< Method \"M\" with scope \"K\" in package \"/abs/method_trace_invocation.rex\".\n",
            "       >M>   \"M\" => \"4\"\n",
            "       >>>   \"4\"\n",
            "     3 *-* say 'after'\n",
            "       >L>   \"after\"\n",
            "       >>>   \"after\"\n",
        ),
    );

    assert_stderr_on_both_engines(
        "/abs/method_trace_nested.rex",
        corpus_source!("lang/method_trace_nested.rex"),
        concat!(
            "     2 *-* do i = 1 to 1\n",
            "       >L>   \"1\"\n",
            "       >L>   \"1\"\n",
            "       >K>   \"TO\" => \"1\"\n",
            "       >=>   I <= \"1\"\n",
            "     3 *-*   say .K~outer\n",
            "       >E>     .K => \"The K class\"\n",
            "       >I> Method \"OUTER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "    10 *-* return .K~inner\n",
            "       >E>   .K => \"The K class\"\n",
            "       >I> Method \"INNER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "    14 *-* return 'deep'\n",
            "       >L>   \"deep\"\n",
            "       >>>   \"deep\"\n",
            "       <I< Method \"INNER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "       >M>   \"INNER\" => \"deep\"\n",
            "       >>>   \"deep\"\n",
            "       <I< Method \"OUTER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "       >M>     \"OUTER\" => \"deep\"\n",
            "       >>>     \"deep\"\n",
            "     4 *-* end\n",
            "     2 *-* do i = 1 to 1\n",
            "       >V>     I => \"1\"\n",
            "       >>>     \"1\"\n",
            "       >>>     \"2\"\n",
            "       >=>     I <= \"2\"\n",
        ),
    );
}

/// **An untrapped condition inside a `::METHOD` body echoes two clauses,
/// innermost first, and neither carries an indent from the other.**
///
/// The oracle's own bytes, captured from `corpus/lang/method_body_raises.rex`
/// with the package path substituted as above. A method activation that
/// failed to seal its level would print the send's clause only, and one that
/// contributed a `Compiled method` traceback line -- which a *native* method
/// does -- would print a third line the oracle does not.
#[test]
fn a_condition_inside_a_method_body_echoes_the_body_and_then_the_send() {
    assert_stderr_on_both_engines(
        "/abs/method_body_raises.rex",
        corpus_source!("lang/method_body_raises.rex"),
        concat!(
            "     9 *-* say 1/0\n",
            "     4 *-* say .K~boom\n",
            "Error 42 running /abs/method_body_raises.rex line 9:  \
             Arithmetic overflow/underflow.\n",
            "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
        ),
    );
}
