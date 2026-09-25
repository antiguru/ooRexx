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

use super::super::dispatch;
use crate::plan::{BodyKey, ProgramId};
use crate::{Activation, Interp, error::Failure};
use rexx_parse::parse_program;
use std::rc::Rc;

/// An interpreter with a live top-level activation, which is where every
/// one of these builtins reads `DIGITS`, `FUZZ` and `FORM` from. The
/// program it activates is a `NOP`: nothing here executes an instruction,
/// and only the settings on the frame matter.
fn interp_with(digits: &str, form: &str, fuzz: &str) -> Interp {
    let mut interp = Interp::new();
    let program = Rc::new(parse_program(b"nop".to_vec()).expect("a NOP program parses"));
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
    let activation = interp.next_activation_id();
    interp.push_activation(Activation::new(
        activation, program, program_id, plan, frame,
    ));
    let settings = &mut interp.activation_mut().settings;
    settings.set_digits_str(digits).expect("a legal DIGITS");
    settings.set_form_str(form).expect("a legal FORM");
    settings.set_fuzz_str(fuzz).expect("a legal FUZZ");
    interp
}

/// Runs `name` over `arguments`, each `None` standing for an omitted
/// interior position, and answers the result's own bytes.
fn call_in(
    interp: &mut Interp,
    name: &[u8],
    arguments: &[Option<&[u8]>],
) -> Result<Vec<u8>, Failure> {
    let args: Vec<_> = arguments
        .iter()
        .map(|argument| argument.map(|bytes| interp.text(bytes)))
        .collect();
    let result = dispatch(interp, name, &args).expect("a builtin name")?;
    Ok(interp.to_text(result).into_owned())
}

fn call_with(
    digits: &str,
    form: &str,
    name: &[u8],
    arguments: &[Option<&[u8]>],
) -> Result<Vec<u8>, Failure> {
    call_in(&mut interp_with(digits, form, "0"), name, arguments)
}

fn call(name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
    call_with("9", "SCIENTIFIC", name, arguments)
}

/// [`call`], for the cases whose answer is the bytes and nothing else.
fn answer(name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
    let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
    call(name, &arguments).expect("this call succeeds")
}

fn answer_at(digits: &str, name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
    let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
    call_with(digits, "SCIENTIFIC", name, &arguments).expect("this call succeeds")
}

/// The `(major, sub)` and substitutions of the condition `name` raises.
fn raised(name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
    raised_at("9", name, arguments)
}

fn raised_at(digits: &str, name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
    let failure = call_with(digits, "SCIENTIFIC", name, arguments).expect_err("this raises");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    (raised.number, raised.sub, raised.additional)
}

fn subs(values: &[&[u8]]) -> Vec<Vec<u8>> {
    values.iter().map(|bytes| bytes.to_vec()).collect()
}

/// Argument 1 raises a different number from arguments 2 and up, and the
/// position those name is one lower than the call's.
#[test]
fn the_target_and_the_later_arguments_raise_different_numbers() {
    for name in [
        b"ABS".as_slice(),
        b"SIGN",
        b"TRUNC",
        b"FORMAT",
        b"MAX",
        b"MIN",
    ] {
        assert_eq!(
            raised(name, &[Some(b"abc")]),
            (93, 943, subs(&[name, b"abc"])),
            "{} did not name itself in 93.943",
            String::from_utf8_lossy(name)
        );
    }
    // The null string and a lone blank reach it too, and report exactly
    // what they are.
    assert_eq!(
        raised(b"ABS", &[Some(b"")]),
        (93, 943, subs(&[b"ABS", b""]))
    );
    assert_eq!(
        raised(b"SIGN", &[Some(b" ")]),
        (93, 943, subs(&[b"SIGN", b" "]))
    );
    // Quoted, so no arithmetic runs before the call: this is SIGN's own
    // error, not the 41.1 an unquoted `-1E1234567890` would raise from
    // the unary minus.
    assert_eq!(
        raised(b"SIGN", &[Some(b"-1E1234567890")]),
        (93, 943, subs(&[b"SIGN", b"-1E1234567890"]))
    );

    // Arguments 2 and up: a different sub-code, and the *method's* own
    // numbering.
    assert_eq!(
        raised(b"MAX", &[Some(b"1"), Some(b"a"), Some(b"3")]),
        (93, 904, subs(&[b"1", b"a"]))
    );
    assert_eq!(
        raised(b"MAX", &[Some(b"1"), Some(b"2"), Some(b"a")]),
        (93, 904, subs(&[b"2", b"a"]))
    );
    assert_eq!(
        raised(b"MIN", &[Some(b"1"), Some(b"")]),
        (93, 904, subs(&[b"1", b""]))
    );
}

/// An omitted `MAX` argument answers differently depending on which
/// representation argument 1 has, and both halves are pinned.
#[test]
fn an_omitted_argument_answers_differently_on_the_two_paths() {
    assert_eq!(
        raised(b"MAX", &[Some(b"1"), None, Some(b"3")]),
        (93, 903, subs(&[b"0"]))
    );
    assert_eq!(
        raised(b"MAX", &[Some(b"1"), Some(b"2"), None, Some(b"4")]),
        (93, 903, subs(&[b"1"]))
    );
    assert_eq!(
        raised(
            b"MIN",
            &[Some(b"1"), Some(b"2"), Some(b"3"), None, Some(b"5")]
        ),
        (93, 903, subs(&[b"2"]))
    );

    // A target that is not an integer object takes the general path.
    assert_eq!(
        raised(b"MAX", &[Some(b"1.0"), None, Some(b"3")]),
        (40, 5, subs(&[b"MAX", b"1"]))
    );
    assert_eq!(
        raised(b"MAX", &[Some(b"1"), Some(b"2.5"), None, Some(b"4")]),
        (40, 5, subs(&[b"MAX", b"2"]))
    );
    // ...and so does one too wide for the precision in force, which is
    // the same call at two settings.
    assert_eq!(
        raised_at("9", b"MAX", &[Some(b"12345"), None, Some(b"3")]),
        (93, 903, subs(&[b"0"]))
    );
    assert_eq!(
        raised_at("3", b"MAX", &[Some(b"12345"), None, Some(b"3")]),
        (40, 5, subs(&[b"MAX", b"1"]))
    );

    // One non-integer sends the whole list back to the general path, so
    // an omission *after* it is never reached.
    assert_eq!(
        raised(b"MAX", &[Some(b"1"), Some(b"a"), None, Some(b"4")]),
        (93, 904, subs(&[b"1", b"a"]))
    );
}

/// A tie keeps the incumbent at **both** ends, which needs a strict
/// operator for each.
#[test]
fn a_tie_keeps_the_earlier_value_at_both_ends() {
    assert_eq!(answer(b"MIN", &[b"1", b"1.0"]), b"1");
    assert_eq!(answer(b"MAX", &[b"1", b"1.0"]), b"1");
    assert_eq!(answer(b"MIN", &[b"1.0", b"1"]), b"1.0");
    assert_eq!(answer(b"MAX", &[b"1.0", b"1"]), b"1.0");
    // The adjacent non-tie, so this cannot pass by never swapping at all.
    assert_eq!(answer(b"MAX", &[b"1", b"2.5"]), b"2.5");
    assert_eq!(answer(b"MIN", &[b"2.5", b"1"]), b"1");
}

/// `MIN` answers a lone target before testing it against `DIGITS` and
/// `MAX` does not -- the one place the two differ for a reason other
/// than direction.
#[test]
fn min_answers_a_lone_target_before_max_would() {
    assert_eq!(answer_at("3", b"MIN", &[b"12345"]), b"12345");
    assert_eq!(answer_at("3", b"MAX", &[b"12345"]), b"1.23E+4");
    // At a precision the value fits, the two agree again.
    assert_eq!(answer_at("9", b"MIN", &[b"12345"]), b"12345");
    assert_eq!(answer_at("9", b"MAX", &[b"12345"]), b"12345");
}

/// Both are variadic, and the arity row is what makes that so.
#[test]
fn max_and_min_take_as_many_arguments_as_they_are_given() {
    assert_eq!(
        answer(b"MAX", &[b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8"]),
        b"8"
    );
    assert_eq!(
        answer(b"MIN", &[b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8"]),
        b"1"
    );
    assert_eq!(raised(b"MAX", &[]), (40, 3, subs(&[b"MAX", b"1"])));
    assert_eq!(raised(b"MIN", &[]), (40, 3, subs(&[b"MIN", b"1"])));
}

/// `FORMAT` rounds half up away from zero, not to even.
#[test]
fn format_rounds_half_up_away_from_zero() {
    let after = |value: &'static [u8], places: &'static [u8]| {
        call(b"FORMAT", &[Some(value), None, Some(places)]).expect("this call succeeds")
    };
    assert_eq!(after(b"2.5", b"0"), b"3");
    assert_eq!(after(b"3.5", b"0"), b"4");
    assert_eq!(after(b"-2.5", b"0"), b"-3");
    assert_eq!(after(b"1.245", b"2"), b"1.25");
    // The adjacent round-down, so this cannot pass by always rounding up.
    assert_eq!(after(b"2.4", b"0"), b"2");
}

/// `before == 0` never succeeds, not even for zero, while an omitted
/// `before` renders zero happily.
#[test]
fn a_before_of_zero_always_fails_and_an_omitted_one_does_not() {
    assert_eq!(
        raised(b"FORMAT", &[Some(b"1"), Some(b"0")]),
        (93, 942, subs(&[b"1", b"0"]))
    );
    assert_eq!(
        raised(b"FORMAT", &[Some(b"0"), Some(b"0")]),
        (93, 942, subs(&[b"0", b"0"]))
    );
    assert_eq!(answer(b"FORMAT", &[b"0"]), b"0");
}

/// 93.942 substitutes the number as `FORMAT` has it when it gives up --
/// rounded by `after`, and reframed by the exponential decision -- not
/// the argument it was handed.
#[test]
fn the_oversize_message_names_the_number_as_it_stands_at_the_failure() {
    assert_eq!(
        raised(b"FORMAT", &[Some(b"1.5"), Some(b"0"), Some(b"0")]),
        (93, 942, subs(&[b"2", b"0"]))
    );
    assert_eq!(
        raised(b"FORMAT", &[Some(b"99.996"), Some(b"2"), Some(b"2")]),
        (93, 942, subs(&[b"100.0", b"2"]))
    );
    let failure = call_with(
        "9",
        "ENGINEERING",
        b"FORMAT",
        &[Some(b"123456.789"), Some(b"2"), None, None, Some(b"0")],
    )
    .expect_err("two spaces cannot hold three integer digits");
    let Failure::Raised(engineering) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(
        (engineering.number, engineering.sub, engineering.additional),
        (93, 942, subs(&[b"123.456789", b"2"]))
    );
    // The same call without `after` leaves the value alone, which is
    // what makes the first case's `2` a rounding rather than a constant.
    assert_eq!(
        raised(b"FORMAT", &[Some(b"1.5"), Some(b"0")]),
        (93, 942, subs(&[b"1.5", b"0"]))
    );

    // It goes through the same rendering a `SAY` would, so it honours
    // `NUMERIC FORM` as well -- measured, `numeric form engineering ;
    // format(1e10,0,,0)` reports `"10E+9"` where the same call under
    // SCIENTIFIC reports `"1E+10"`. `expp` is `0` in both, which
    // suppresses exponential form in the *result* and not here.
    let narrow = [Some(b"1e10".as_slice()), Some(b"0"), None, Some(b"0")];
    for (form, expected) in [
        ("SCIENTIFIC", b"1E+10".as_slice()),
        ("ENGINEERING", b"10E+9"),
    ] {
        let failure = call_with("9", form, b"FORMAT", &narrow)
            .expect_err("no spaces can hold an integer part");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(
            (raised.number, raised.sub, raised.additional),
            (93, 942, subs(&[expected, b"0"])),
            "{form} named the wrong value"
        );
    }
}

/// `expp == 0` suppresses exponential form and beats `expt == 0`, which
/// otherwise forces it.
#[test]
fn expp_zero_suppresses_exponential_and_beats_expt_zero() {
    let exp = |value: &'static [u8], expp: Option<&'static [u8]>, expt: Option<&'static [u8]>| {
        call(b"FORMAT", &[Some(value), None, None, expp, expt]).expect("this call succeeds")
    };
    assert_eq!(exp(b"12345", Some(b"0"), None), b"12345");
    assert_eq!(exp(b"12345", None, Some(b"0")), b"1.2345E+4");
    assert_eq!(exp(b"12345", Some(b"0"), Some(b"0")), b"12345");
    assert_eq!(exp(b"12345", Some(b"2"), Some(b"0")), b"1.2345E+04");
    assert_eq!(exp(b"12345", Some(b"4"), Some(b"0")), b"1.2345E+0004");
    assert_eq!(exp(b"1e10", None, Some(b"20")), b"10000000000");
    // `expt` is a threshold and never occupies a byte, so a value far
    // past what a width could ever be is simply a trigger nothing
    // reaches -- measured, `format(1,,,,999999999999999999)` is `1`.
    assert_eq!(exp(b"1", None, Some(b"999999999999999999")), b"1");
    // An exponent too wide for an explicit `expp` is 93.941, and the
    // mantissa it names is the reframed one.
    assert_eq!(
        raised(
            b"FORMAT",
            &[Some(b"1e10"), None, None, Some(b"1"), Some(b"0")]
        ),
        (93, 941, subs(&[b"1", b"1"]))
    );
}

/// `FORMAT` honours `NUMERIC FORM`, which is the second half of the pair
/// `DIGITS` alone cannot show.
#[test]
fn format_honours_numeric_form() {
    let args = [Some(b"1e10".as_slice()), None, None, None, Some(b"0")];
    assert_eq!(
        call_with("9", "SCIENTIFIC", b"FORMAT", &args).expect("legal"),
        b"1E+10"
    );
    assert_eq!(
        call_with("9", "ENGINEERING", b"FORMAT", &args).expect("legal"),
        b"10E+9"
    );
}

/// The three validation layers run in the oracle's own order, which a
/// program can tell apart because each names a different number.
#[test]
fn the_three_validation_layers_run_in_the_oracles_order() {
    assert_eq!(
        raised(b"TRUNC", &[Some(b"AB.CD"), Some(b"V")]),
        (40, 12, subs(&[b"TRUNC", b"2", b"V"]))
    );
    assert_eq!(
        raised(b"TRUNC", &[Some(b"AB.CD"), Some(b"-1")]),
        (93, 943, subs(&[b"TRUNC", b"AB.CD"]))
    );
    assert_eq!(
        raised(b"TRUNC", &[Some(b"1.5"), Some(b"-1")]),
        (93, 906, subs(&[b"1", b"-1"]))
    );

    assert_eq!(
        raised(b"FORMAT", &[Some(b"1"), Some(b"x")]),
        (40, 12, subs(&[b"FORMAT", b"2", b"x"]))
    );
    assert_eq!(
        raised(b"FORMAT", &[Some(b"a"), Some(b"-1")]),
        (93, 943, subs(&[b"FORMAT", b"a"]))
    );
    assert_eq!(
        raised(b"FORMAT", &[Some(b"1"), Some(b"-1")]),
        (93, 906, subs(&[b"1", b"-1"]))
    );
    // All four optional positions are type-checked before the target,
    // in call order, and each names its own method position when it is
    // the range that is wrong.
    assert_eq!(
        raised(b"FORMAT", &[Some(b"a"), Some(b"x"), Some(b"y")]),
        (40, 12, subs(&[b"FORMAT", b"2", b"x"]))
    );
    assert_eq!(
        raised(b"FORMAT", &[Some(b"1"), Some(b"1"), Some(b"y")]),
        (40, 12, subs(&[b"FORMAT", b"3", b"y"]))
    );
    for (position, method) in [(2usize, b"1"), (3, b"2"), (4, b"3"), (5, b"4")] {
        let mut args: Vec<Option<&[u8]>> = vec![Some(b"1"), None, None, None, None];
        args[position - 1] = Some(b"-1");
        assert_eq!(
            raised(b"FORMAT", &args),
            (93, 906, subs(&[method, b"-1"])),
            "argument {position} named the wrong method position"
        );
    }
}

/// `TRUNC` rounds its input to `DIGITS` before truncating, and never
/// produces exponential form.
#[test]
fn trunc_rounds_to_digits_first_and_never_goes_exponential() {
    assert_eq!(answer_at("3", b"TRUNC", &[b"123456", b"2"]), b"123000.00");
    assert_eq!(
        answer_at("9", b"TRUNC", &[b"1e20"]),
        b"100000000000000000000"
    );
    assert_eq!(answer(b"TRUNC", &[b"12.987", b"2"]), b"12.98");
    assert_eq!(answer(b"TRUNC", &[b"1.5"]), b"1");
    assert_eq!(answer(b"TRUNC", &[b"-1.5"]), b"-1");
    assert_eq!(answer(b"TRUNC", &[b"0", b"3"]), b"0.000");
    // A value that truncates away entirely loses its sign.
    assert_eq!(answer(b"TRUNC", &[b"-0.0001234", b"2"]), b"0.00");
}

/// `ABS` and `SIGN` round to the precision in force, and the rounding is
/// not skipped for a value that is already positive.
#[test]
fn abs_rounds_whatever_the_sign_and_sign_ignores_the_precision() {
    assert_eq!(answer_at("3", b"ABS", &[b"1.23456"]), b"1.23");
    assert_eq!(answer_at("3", b"ABS", &[b"-1.23456"]), b"1.23");
    assert_eq!(answer(b"ABS", &[b"-4.5"]), b"4.5");
    assert_eq!(answer(b"ABS", &[b"  -4.5  "]), b"4.5");
    assert_eq!(answer(b"SIGN", &[b"-12"]), b"-1");
    assert_eq!(answer(b"SIGN", &[b"0"]), b"0");
    // Every spelling of zero is unsigned.
    assert_eq!(answer(b"SIGN", &[b"-0.0"]), b"0");

    // **The rounding is in the stored value, not only in the rendering,
    // and only a *later* read at a wider precision can see that.**
    // `to_text` re-rounds through the captured pair either way, so a
    // result built without `round_to` renders identically; feeding it
    // back into a builtin that rounds to the precision then in force is
    // what tells the two apart. Measured: `numeric digits 3 ; x =
    // abs(-1.23456) ; numeric digits 9 ; say max(x,-1)` is `1.23`, not
    // `1.23456`.
    let mut interp = interp_with("3", "SCIENTIFIC", "0");
    let argument = interp.text(b"-1.23456");
    let rounded = dispatch(&mut interp, b"ABS", &[Some(argument)])
        .expect("a builtin name")
        .expect("a legal call");
    assert_eq!(&*interp.to_text(rounded), b"1.23");
    interp
        .activation_mut()
        .settings
        .set_digits_str("9")
        .expect("a legal DIGITS");
    // The result object itself, not a fresh value built from its bytes:
    // re-parsing the rendering would round it a second time and the
    // mutation would survive.
    let minus_one = interp.text(b"-1");
    let widened = dispatch(&mut interp, b"MAX", &[Some(rounded), Some(minus_one)])
        .expect("a builtin name")
        .expect("a legal call");
    assert_eq!(&*interp.to_text(widened), b"1.23");
}

/// The five builtins that answer a *number* capture the `DIGITS`/`FORM`
/// pair at the call, and the two that answer *text* have nothing a later
/// setting could reshape (D15).
#[test]
fn a_result_is_rendered_under_the_settings_that_produced_it() {
    // DIGITS moved down after the value was made.
    let mut interp = interp_with("5", "SCIENTIFIC", "0");
    let max =
        call_in(&mut interp, b"MAX", &[Some(b"123456789"), Some(b"1")]).expect("a legal call");
    let abs = call_in(&mut interp, b"ABS", &[Some(b"-1.23456789")]).expect("a legal call");
    let format = call_in(
        &mut interp,
        b"FORMAT",
        &[Some(b"1.23456"), None, Some(b"4")],
    )
    .expect("a legal call");
    let trunc = call_in(&mut interp, b"TRUNC", &[Some(b"1.23456"), Some(b"5")]).expect("legal");
    interp
        .activation_mut()
        .settings
        .set_digits_str("12")
        .expect("a legal DIGITS");
    assert_eq!(max, b"1.2346E+8");
    assert_eq!(abs, b"1.2346");
    assert_eq!(format, b"1.2346");
    assert_eq!(trunc, b"1.23460");
    // Re-running the same calls at the new precision gives different
    // answers, which is what makes the four above evidence of capture
    // rather than of a fixed rendering.
    assert_eq!(
        call_in(&mut interp, b"MAX", &[Some(b"123456789"), Some(b"1")]).expect("legal"),
        b"123456789"
    );

    // FORM moved after the value was made.
    let mut interp = interp_with("9", "ENGINEERING", "0");
    let max = call_in(&mut interp, b"MAX", &[Some(b"1e10"), Some(b"1")]).expect("legal");
    interp
        .activation_mut()
        .settings
        .set_form_str("SCIENTIFIC")
        .expect("a legal FORM");
    assert_eq!(max, b"10E+9");
    assert_eq!(
        call_in(&mut interp, b"MAX", &[Some(b"1e10"), Some(b"1")]).expect("legal"),
        b"1E+10"
    );
}

/// `MAX`/`MIN` compare under `NUMERIC FUZZ`, which no other builtin in
/// this family reads.
#[test]
fn max_and_min_compare_under_the_fuzz_in_force() {
    let mut fuzzy = interp_with("9", "SCIENTIFIC", "3");
    assert_eq!(
        call_in(
            &mut fuzzy,
            b"MAX",
            &[Some(b"100000000.0"), Some(b"100000001")]
        )
        .expect("legal"),
        b"100000000"
    );
    let mut exact = interp_with("9", "SCIENTIFIC", "0");
    assert_eq!(
        call_in(
            &mut exact,
            b"MAX",
            &[Some(b"100000000.0"), Some(b"100000001")]
        )
        .expect("legal"),
        b"100000001"
    );
}

/// `RANDOM`'s argument validation, including the two errors that are
/// easy to swap.
#[test]
fn random_validates_its_range_and_its_seed_separately() {
    assert_eq!(
        raised(b"RANDOM", &[Some(b"-1")]),
        (40, 33, subs(&[b"-1", b""]))
    );
    assert_eq!(
        raised(b"RANDOM", &[Some(b"5"), Some(b"1")]),
        (40, 33, subs(&[b"5", b"1"]))
    );
    assert_eq!(
        raised(b"RANDOM", &[Some(b"1"), Some(b"2"), Some(b"-1")]),
        (40, 13, subs(&[b"RANDOM", b"3", b"-1"]))
    );
    // Every substitution is the *converted* integer, not the argument's
    // own text: measured, `random(1,2,'-1.0')` reports `found "-1"` and
    // `random('5.0','1.0')` reports `("5")` and `("1")`.
    assert_eq!(
        raised(b"RANDOM", &[Some(b"1"), Some(b"2"), Some(b"-1.0")]),
        (40, 13, subs(&[b"RANDOM", b"3", b"-1"]))
    );
    assert_eq!(
        raised(b"RANDOM", &[Some(b"5.0"), Some(b"1.0")]),
        (40, 33, subs(&[b"5", b"1"]))
    );
    assert_eq!(
        raised(b"RANDOM", &[Some(b"0"), Some(b"1000000000")]),
        (40, 32, subs(&[b"0", b"1000000000"]))
    );
    assert_eq!(
        raised(b"RANDOM", &[Some(b"1.5")]),
        (40, 12, subs(&[b"RANDOM", b"1", b"1.5"]))
    );
    // A degenerate range is legal at both ends, and a widest-legal one
    // is too -- the check is `>`, not `>=`.
    assert_eq!(answer(b"RANDOM", &[b"5", b"5"]), b"5");
    assert_eq!(answer(b"RANDOM", &[b"0", b"0"]), b"0");
    assert!(call(b"RANDOM", &[Some(b"0"), Some(b"999999999")]).is_ok());
}

/// A seeded stream continues across later *unseeded* calls, and it is
/// the oracle's own stream number for number.
#[test]
fn a_seed_starts_a_stream_that_later_calls_continue() {
    let seeded = [b"1".as_slice(), b"999999999", b"12345"];
    let unseeded = [b"1".as_slice(), b"999999999"];
    let mut interp = interp_with("9", "SCIENTIFIC", "0");
    let mut run = || {
        let first = call_in(
            &mut interp,
            b"RANDOM",
            &seeded.iter().map(|b| Some(*b)).collect::<Vec<_>>(),
        )
        .expect("legal");
        let rest: Vec<Vec<u8>> = (0..2)
            .map(|_| {
                call_in(
                    &mut interp,
                    b"RANDOM",
                    &unseeded.iter().map(|b| Some(*b)).collect::<Vec<_>>(),
                )
                .expect("legal")
            })
            .collect();
        (first, rest)
    };
    let (first, rest) = run();
    // Measured on `build/bin/rexx`, and identical on three separate runs
    // of the same program.
    assert_eq!(first, b"776163098");
    assert_eq!(rest, vec![b"950445098".to_vec(), b"552120333".to_vec()]);
    assert_eq!(run(), (first, rest));
}

/// `RANDOM`'s answer keeps its own spelling whatever `DIGITS` and `FORM`
/// say, because the oracle answers a `RexxInteger` and not a value
/// carrying the D15 pair.
#[test]
fn a_random_answer_keeps_its_own_spelling_at_every_precision() {
    let degenerate = [Some(b"12345".as_slice()), Some(b"12345")];
    for digits in ["1", "3", "9", "12"] {
        assert_eq!(
            call_with(digits, "SCIENTIFIC", b"RANDOM", &degenerate).expect("legal"),
            b"12345",
            "DIGITS {digits} reshaped the answer"
        );
    }
    assert_eq!(
        call_with("3", "ENGINEERING", b"RANDOM", &degenerate).expect("legal"),
        b"12345"
    );
    // The adjacent contrast, so this cannot pass on an implementation
    // where *nothing* is ever reshaped: `ABS` answers a real number and
    // the same precision does move it.
    assert_eq!(answer_at("3", b"ABS", &[b"-12345"]), b"1.23E+4");
}

/// A padded result the allocator refuses is Error 5 at rc 251 rather
/// than an abort, for both builtins that pad.
#[test]
fn a_padding_width_too_large_to_allocate_raises_error_5() {
    for (name, args) in [
        (
            b"TRUNC".as_slice(),
            vec![Some(b"1".as_slice()), Some(b"123456789012345678")],
        ),
        (
            b"FORMAT",
            vec![Some(b"1".as_slice()), Some(b"123456789012345678")],
        ),
        (
            b"FORMAT",
            vec![Some(b"1".as_slice()), None, Some(b"123456789012345678")],
        ),
    ] {
        let failure = call(name, &args).expect_err("that width cannot be allocated");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (5, 0));
    }
    // The adjacent success: a width that is merely large is honoured.
    assert_eq!(answer(b"TRUNC", &[b"1", b"1000"]).len(), 1002);
    assert_eq!(answer(b"FORMAT", &[b"1", b"1000"]).len(), 1000);

    // `expp` is the third width, and it is the one that is reserved
    // only when the exponent field is actually written -- the same line
    // the oracle draws. The two calls below differ *only* in `expt`,
    // which is what decides whether there is an exponent to pad, and
    // measured they answer `1` at rc 0 and `System resources exhausted.`
    // at rc 251 respectively.
    assert_eq!(
        call(
            b"FORMAT",
            &[Some(b"1"), None, None, Some(b"999999999999999999")]
        )
        .expect("a width that is never written is never allocated"),
        b"1"
    );
    let failure = call(
        b"FORMAT",
        &[
            Some(b"1"),
            None,
            None,
            Some(b"999999999999999999"),
            Some(b"0"),
        ],
    )
    .expect_err("that exponent field cannot be allocated");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (5, 0));

    // And a width Rust's own formatter cannot express is still a
    // result, not a panic: `format!("{:0width$}", ..)` takes a `u16`.
    // Measured, `length(format(1e10,,,65535))` is 65538 and
    // `length(format(1e10,,,65536))` is 65539.
    assert_eq!(
        call(b"FORMAT", &[Some(b"1e10"), None, None, Some(b"65535")])
            .expect("legal")
            .len(),
        65538
    );
    assert_eq!(
        call(b"FORMAT", &[Some(b"1e10"), None, None, Some(b"65536")])
            .expect("legal")
            .len(),
        65539
    );
}
