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
            let mut rendered = format!("rc> {}\n", outcome.exit_code);
            for line in String::from_utf8_lossy(&outcome.stdout).lines() {
                rendered.push_str(&format!("out> {line}\n"));
            }
            for line in String::from_utf8_lossy(&outcome.stderr).lines() {
                rendered.push_str(&format!("err> {line}\n"));
            }
            rendered
        });
    });
    assert!(cases > 0, "{UNTRAPPED_SORT_CASES} ran no case");
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
