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
