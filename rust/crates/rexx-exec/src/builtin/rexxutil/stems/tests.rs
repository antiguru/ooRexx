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

/// The cases [`an_untrapped_error_inside_the_sort_is_reported`] reads.
const UNTRAPPED_SORT_CASES: &str = "tests/stem_sort_untrapped_cases";

/// The cases [`a_stem_named_by_its_variable_outlives_the_variable`] reads.
const DROPPED_STEM_CASES: &str = "tests/stem_dropped_by_name_cases";

/// An outcome as the case files spell it: `rc>`, then a line per line of
/// standard output and of standard error.
fn rendered(outcome: &crate::Outcome) -> String {
    let mut rendered = format!("rc> {}\n", outcome.exit_code);
    for line in String::from_utf8_lossy(&outcome.stdout).lines() {
        rendered.push_str(&format!("out> {line}\n"));
    }
    for line in String::from_utf8_lossy(&outcome.stderr).lines() {
        rendered.push_str(&format!("err> {line}\n"));
    }
    rendered
}

/// **An error `SysStemSort` raises from inside its sort is reported when
/// nothing traps it**, where the oracle crashes (`corpus/oracle-crashes.txt`
/// entry 37, upstream SF #2068). No differential can cover these, so the case
/// file is this crate's own answer: the error the trapped oracle raises for
/// the same stem, reported as any routine's error is. The adjacent successes
/// are in the same file.
#[test]
fn an_untrapped_error_inside_the_sort_is_reported() {
    let mut cases = 0;
    datadriven::walk(UNTRAPPED_SORT_CASES, |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(
                case.directive, "program",
                "unknown directive {:?}",
                case.directive
            );
            let outcome = crate::run_program(
                "/t.rex",
                case.input.clone().into_bytes(),
                crate::Invocation::none(),
            );
            rendered(&outcome)
        });
    });
    assert!(cases > 0, "{UNTRAPPED_SORT_CASES} ran no case");
}

/// **`SysStemCopy`, `SysStemDelete`, `SysStemInsert` and `SysStemSort` hold
/// a stem they resolved by its variable's name** while later arguments and
/// elements run `STRING` methods: one that drops the variable and allocates
/// leaves the routine working on the stem it resolved. Each case runs
/// normally and collecting at every allocation, and the two must agree. The
/// oracle's answer depends on when it collects (`corpus/oracle-crashes.txt`
/// entry 47), so no differential can cover these, and the case file is this
/// crate's own answer.
#[test]
fn a_stem_named_by_its_variable_outlives_the_variable() {
    let mut cases = 0;
    datadriven::walk(DROPPED_STEM_CASES, |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(
                case.directive, "program",
                "unknown directive {:?}",
                case.directive
            );
            let source = case.input.clone().into_bytes();
            let normal = crate::run_program("/t.rex", source.clone(), crate::Invocation::none());
            let stressed =
                crate::run_program_collect_every_alloc("/t.rex", source, crate::Invocation::none());
            assert!(stressed.collections > 0, "the mode collected nothing");
            let stressed = rendered(&stressed);
            assert_eq!(rendered(&normal), stressed, "{}", case.input);
            stressed
        });
    });
    assert!(cases > 0, "{DROPPED_STEM_CASES} ran no case");
}

/// **`SysDumpVariables` holds the tail values it read before it writes any
/// of them**, as the oracle's `toDirectory` snapshot does: a `STRING` method
/// that empties the stem midway leaves the rest of the dump intact. Measured,
/// oracle rc 0, these lines in hash order. This crate writes tail 2 first, so
/// the emptying runs before tail 1 is written, and collecting at every
/// allocation sweeps an unheld value.
#[test]
fn a_dumped_stem_keeps_its_tails_when_a_string_method_empties_it() {
    let source = b"s.1 = copies('heap ', 8)\n\
        s.2 = .k~new(s.)\n\
        call SysDumpVariables\n\
        say s.1\n\
        ::class k\n\
        ::method init\n\
        expose st.\n\
        use arg st.\n\
        ::method string\n\
        expose st.\n\
        st.~empty\n\
        x = copies('filler', 50)\n\
        return 'K'\n";
    let outcome = crate::run_program_collect_every_alloc(
        "/t.rex",
        source.to_vec(),
        crate::Invocation::none(),
    );
    assert!(outcome.collections > 0, "the mode collected nothing");
    assert_eq!(
        outcome.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    let stdout = String::from_utf8_lossy(&outcome.stdout).into_owned();
    let mut lines: Vec<&str> = stdout.split_terminator('\n').collect();
    lines.sort_unstable();
    assert_eq!(
        lines,
        [
            "Name=S., Value='S.'",
            "Name=S.1, Value='heap heap heap heap heap heap heap heap '\r",
            "Name=S.2, Value='K'\r",
            "S.1",
        ]
    );
}

/// **`SysDumpVariables` holds the variables' values it read before it
/// writes any of them**: a `STRING` method that drops the caller's other
/// variables and allocates leaves their values to be written as read.
/// Collecting at every allocation, an unheld value is swept. Oracle rc 0,
/// these lines in hash order, measured.
#[test]
fn a_dump_keeps_the_values_of_variables_a_string_method_drops() {
    let source = b".k~new~run\n\
        ::class k\n\
        ::method run\n\
        expose a b c d e\n\
        a = copies('alpha ', 3); b = copies('bravo ', 3); c = copies('charlie ', 3)\n\
        d = copies('delta ', 3); e = copies('echo ', 3); z = self\n\
        call SysDumpVariables\n\
        ::method string\n\
        expose a b c d e\n\
        drop a b c d e\n\
        x = copies('filler ', 50)\n\
        return 'K'\n";
    let outcome = crate::run_program_collect_every_alloc(
        "/t.rex",
        source.to_vec(),
        crate::Invocation::none(),
    );
    assert!(outcome.collections > 0, "the mode collected nothing");
    assert_eq!(
        outcome.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    let stdout = String::from_utf8_lossy(&outcome.stdout).into_owned();
    let mut lines: Vec<&str> = stdout.split_terminator('\n').collect();
    lines.sort_unstable();
    assert_eq!(
        lines,
        [
            "Name=A, Value='alpha alpha alpha '",
            "Name=B, Value='bravo bravo bravo '",
            "Name=C, Value='charlie charlie charlie '",
            "Name=D, Value='delta delta delta '",
            "Name=E, Value='echo echo echo '",
            "Name=SELF, Value='K'",
            "Name=SUPER, Value='The Object class'",
            "Name=Z, Value='K'",
        ]
    );
}

/// **`SysDumpVariables` over a variable whose `STRING` method raises
/// reports the raise**, after the lines already written, where the oracle
/// crashes even under `SIGNAL ON SYNTAX` (`corpus/oracle-crashes.txt` entry
/// 38). No differential can run it, so this is the crate's own answer: the
/// method's 93.900 trapped as any raise from a routine is. The line for `A`
/// is written first where the variable order puts it first, as the oracle's
/// order did before it crashed.
#[test]
fn a_string_method_raising_inside_the_dump_is_trapped() {
    let source = b"signal on syntax\n\
        call p\n\
        exit\n\
        syntax: say 'trapped' condition('O')~code\n\
        exit\n\
        p: procedure\n\
        a = 'first'\n\
        b = .k~new\n\
        call SysDumpVariables\n\
        return\n\
        ::class k\n\
        ::method string\n\
        raise syntax 93.900 array('boom')\n";
    let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stderr), "");
    let stdout = String::from_utf8_lossy(&outcome.stdout).into_owned();
    let (written, trapped) = stdout
        .trim_end_matches('\n')
        .rsplit_once('\n')
        .unwrap_or(("", stdout.trim_end_matches('\n')));
    assert_eq!(trapped, "trapped 93.900", "{stdout:?}");
    assert!(
        written.is_empty() || written == "Name=A, Value='first'",
        "{stdout:?}"
    );
}
