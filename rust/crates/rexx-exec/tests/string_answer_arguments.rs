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

//! A builtin argument whose user `STRING` method answers an object with no
//! string value (Deviation 30). Each `call` case runs `say <call>` with `o`
//! an instance whose `STRING` answers the `string=` object, and records the
//! oracle's rc, stdout and stderr. `program` runs its input as the whole
//! program. `refuses` cases are target positions, where the oracle reads
//! `.nil` through the string layout and this crate refuses.

use rexx_exec::Invocation;

mod watchdog;

const CASES: &str = "tests/string_answer_arguments";

/// The path every case runs as, which the recorded stderr spells `PROGRAM`.
const PATH: &str = "probe.rex";

/// The program a `call` case runs.
fn call_program(call: &str, answer: &str) -> String {
    format!(
        "o = .q~new\nsay {call}\nsay \"after\"\nexit\n::class q\n::method string\n  return {answer}\n"
    )
}

/// `rc`, stdout and stderr as the case file records them.
fn transcript(text: String) -> String {
    let outcome = watchdog::run_bounded(PATH, text.into_bytes(), Invocation::none());
    format!(
        "rc {}\n[stdout]\n{}[stderr]\n{}",
        outcome.exit_code,
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&outcome.stderr).replace(PATH, "PROGRAM"),
    )
}

#[test]
fn string_answers_at_builtin_argument_positions() {
    let mut cases = 0;
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            cases += 1;
            let input = case.input.trim_end();
            match case.directive.as_str() {
                "call" | "refuses" => {
                    let answer = match case.args.get("string").map(|v| v[0].as_str()) {
                        Some("object") => ".object~new",
                        Some("directory") => ".directory~new",
                        other => panic!("unknown string= {other:?}"),
                    };
                    transcript(call_program(input, answer))
                }
                "program" => transcript(format!("{input}\n")),
                other => panic!("unknown directive {other}"),
            }
        });
    });
    assert!(cases > 0, "no case ran");
}
