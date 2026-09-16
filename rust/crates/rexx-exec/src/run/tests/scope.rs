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

// ---- PROCEDURE, PROCEDURE EXPOSE, USE and the variable reference ----

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

/// `USE STRICT ARG`'s three refusals report the method catalogue rows inside a
/// method and the call ones outside -- `UseInstruction.cpp`'s three
/// `inMethod()` switches (`:103`, `:292`, `:305`).
#[test]
fn use_strict_arg_reports_method_errors_in_a_method_and_call_errors_outside() {
    for (source, status, catalogue) in [
        (
            "o = .K~new\n::class K\n::method init\nuse strict arg a\n",
            163,
            "Error 93.901:  Not enough arguments for method; 1 expected.\n",
        ),
        (
            "o = .K~new(1, 2)\n::class K\n::method init\nuse strict arg a\n",
            163,
            "Error 93.902:  Too many arguments in invocation of method; 1 expected.\n",
        ),
        (
            "o = .K~new(, 2)\n::class K\n::method init\nuse strict arg a, b\n",
            163,
            "Error 93.903:  Missing argument in method; argument 1 is required.\n",
        ),
        (
            "call sub\n::routine sub\nuse strict arg a\n",
            216,
            "Error 40.3:  Not enough arguments in invocation of SUB; minimum expected is 1.\n",
        ),
        (
            "call sub 1, 2\n::routine sub\nuse strict arg a\n",
            216,
            "Error 40.4:  Too many arguments in invocation of SUB; maximum expected is 1.\n",
        ),
        (
            "call sub , 2\n::routine sub\nuse strict arg a, b\n",
            216,
            "Error 40.5:  Missing argument in invocation of SUB; argument 1 is required.\n",
        ),
    ] {
        let outcome = crate::run_program(
            "/t.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(
            (outcome.exit_code, outcome.stdout.len()),
            (status, 0),
            "{source:?}"
        );
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert!(stderr.ends_with(catalogue), "{source:?}: {stderr:?}");
    }
    // The neighbouring successes: an omitted position without `STRICT` drops
    // the target, and one with a default of its own is filled rather than
    // refused.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 ,2\nexit\nsub2: procedure\nuse arg p, q\nsay '['p']['q']'\nreturn\n",
        ),
        b"[P][2]\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub2 ,2\nexit\n\
              sub2: procedure\nuse strict arg p = 'dflt', q\nsay '['p']['q']'\nreturn\n",
        ),
        b"[dflt][2]\n".to_vec()
    );
}

/// `USE ARG >name` aliases the caller's variable; the same call into a
/// plain target copies its value instead.
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
#[test]
fn use_arg_alias_requires_the_reference_kind_to_match_the_target() {
    // Simple reference -> STEM target: refused.
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

/// A variable reference renders as the referenced variable's value.
/// Measured: `say >p` prints `p`'s value.
#[test]
fn a_variable_reference_renders_as_the_referenced_value() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"p = 'orig'\nsay >p"),
        b"orig\n".to_vec()
    );
}

/// `>name` answers the variable, not its value: measured, oracle rc 0,
/// `p = 'orig'; o = >p; say o~class~id o~name o~value` is
/// `VariableReference P orig`.
#[test]
fn a_variable_reference_answers_the_variable() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'orig'\no = >p\nsay o~class~id o~name o~value",
        ),
        b"VariableReference P orig\n".to_vec()
    );
}

/// The variable is read at the ask and written through, so a reference and
/// the variable it names are one piece of storage rather than two.
#[test]
fn a_variable_reference_reads_and_writes_the_variable() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'first'\no = >p\np = 'second'\nsay o~value\n\
              o~value = 'third'\nsay p",
        ),
        b"second\nthird\n".to_vec()
    );
}

/// Two references to one variable share its storage, which is what
/// `RootSet::promote` being idempotent buys: a second `>p` answers a second
/// object over the same cell.
#[test]
fn two_references_to_one_variable_share_it() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'orig'\nfirst = >p\nsecond = >p\n\
              first~value = 'through the first'\nsay second~value p",
        ),
        b"through the first through the first\n".to_vec()
    );
}

/// A reference outlives the activation whose local it names -- measured,
/// oracle rc 0: a `procedure` returning `>v` answers a reference whose
/// `~value` reads `42` and whose `~value =` still writes after the return.
#[test]
fn a_variable_reference_outlives_its_frame() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"o = ref()\nsay o~name o~value\no~value = 99\nsay o~value\nexit\n\
              ref: procedure\nv = 42\nreturn >v\n",
        ),
        b"V 42\n99\n".to_vec()
    );
}

/// A message the class does not define reaches `UNKNOWN`, which sends it to
/// the referenced value -- `VariableReference::unknownRexx`. Measured, oracle
/// rc 0: `p = 'abc'; say (>p)~length` is `3`.
#[test]
fn a_variable_reference_forwards_an_unknown_message() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"p = 'abc'\nsay (>p)~length"),
        b"3\n".to_vec()
    );
}

/// `USE ARG >` takes the argument's **value**, so a reference that reached
/// the call through a variable binds exactly as one written `>` at the call
/// site does. Measured, oracle rc 0.
#[test]
fn use_arg_alias_takes_any_variable_reference_value() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"p = 'orig'\no = >p\ncall sub o\nsay p\nexit\n\
              sub: procedure\nuse arg >q\nq = 'written'\nreturn\n",
        ),
        b"written\n".to_vec()
    );
}

/// The caller's own arguments survive a nested call.
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

/// `EXPOSE` of a single compound tail fails loudly, the same refusal
/// `PROCEDURE EXPOSE` makes and for the same reason.
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
