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

//! One truth judgment: each case's input is a Rexx expression, fed through
//! every context that takes a logical value. A context answers `true`,
//! `false`, an Error 34 code, or 97.1 where the value is sent the operator as
//! a message instead. Beside the recorded answers, every context that judges
//! must agree on true, false or error, and each context raises one
//! sub-number whatever the value.

use std::collections::BTreeMap;

use rayon::prelude::*;
use rexx_exec::Invocation;

mod watchdog;

const CASES: &str = "tests/truth";

/// The classes a case's expression may name.
const CLASSES: &str = "::class s1\n::method string; return 1\n\
                       ::class s0\n::method string; return 0\n\
                       ::class sa\n::method string; return .array~of(1)\n\
                       ::class sn\n::method string; return .directory~new\n\
                       ::class sd\n";

/// Each context's name and program, `E` standing for the expression. Every
/// program prints `T` or `F`.
const CONTEXTS: &[(&str, &str)] = &[
    ("if", "v = E\nif v then say 'T'; else say 'F'\n"),
    (
        "when",
        "v = E\nselect; when v then say 'T'; otherwise say 'F'; end\n",
    ),
    (
        "while",
        "v = E\nt = 'F'\ndo while v; t = 'T'; leave; end\nsay t\n",
    ),
    (
        "until",
        "v = E\nn = 0\ndo until v; n = n + 1; if n > 1 then leave; end\nsay word('T F', n)\n",
    ),
    ("not", "v = E\nsay translate(\\v, 'FT', '10')\n"),
    ("and", "v = E\nsay translate(1 & v, 'TF', '10')\n"),
    ("list", "v = E\nif 1, v then say 'T'; else say 'F'\n"),
    // A comparison is a condition the IR tests in its own op, whose answer
    // is a logical constant more often than not.
    (
        "if-compare",
        "o = .o~new\nif o = 1 then say 'T'; else say 'F'\n\
         exit\n::class o\n::method '='; return E\n",
    ),
    (
        "when-compare",
        "o = .o~new\nselect; when o = 1 then say 'T'; otherwise say 'F'; end\n\
         exit\n::class o\n::method '='; return E\n",
    ),
    (
        "while-compare",
        "o = .o~new\nt = 'F'\ndo while o = 1; t = 'T'; leave; end\nsay t\n\
         exit\n::class o\n::method '='; return E\n",
    ),
    (
        "case",
        "select case .c~new; when 'x' then say 'T'; otherwise say 'F'; end\n\
         exit\n::class c\n::method '=='; return E\n",
    ),
    (
        "do-to",
        "n = 0\ndo i = .c~new to 3 for 3; n = n + 1; end\nsay translate(n, 'TF', '03')\n\
         exit\n::class c\n::method '+'; return self\n::method '>'; return E\n",
    ),
    (
        "do-by",
        "n = 0\ndo i = .k~new to 5 by .b~new for 3; n = n + 1; end\n\
         say translate(n, 'TF', '03')\nexit\n\
         ::class k\n::method '+'; return self\n::method '>'; return 1 = 0\n::method '<'; return 1 = 1\n\
         ::class b\n::method '+'; return self\n::method '<'; return E\n",
    ),
    (
        "array-hasitem",
        "k = .array~of(.c~new)\nsay translate(k~hasItem(.c~new), 'TF', '10')\n\
         exit\n::class c\n::method '=='; return E\n",
    ),
    (
        "array-index",
        "k = .array~of(.c~new)\nsay translate(k~index(.c~new) \\== .nil, 'TF', '10')\n\
         exit\n::class c\n::method '=='; return E\n",
    ),
    (
        "list-hasitem",
        "k = .list~of(.c~new)\nsay translate(k~hasItem(.c~new), 'TF', '10')\n\
         exit\n::class c\n::method '=='; return E\n",
    ),
    (
        "list-index",
        "k = .list~of(.c~new)\nsay translate(k~index(.c~new) \\== .nil, 'TF', '10')\n\
         exit\n::class c\n::method '=='; return E\n",
    ),
    (
        "table-hasitem",
        "k = .table~new; k['k'] = .c~new\nsay translate(k~hasItem(.c~new), 'TF', '10')\n\
         exit\n::class c\n::method '=='; return E\n",
    ),
    (
        "table-index",
        "k = .table~new; k['k'] = .c~new\n\
         say translate(k~index(.c~new) \\== .nil, 'TF', '10')\n\
         exit\n::class c\n::method '=='; return E\n",
    ),
];

/// What one context answered: `true`, `false`, or the error code.
fn answer(context: &str, program: &str, expression: &str) -> String {
    let mut text = program.replace('E', expression);
    if !text.contains("exit\n") {
        text.push_str("exit\n");
    }
    text.push_str(CLASSES);
    let outcome = watchdog::run_bounded("truth.rex", text.into_bytes(), Invocation::none());
    let stdout = String::from_utf8_lossy(&outcome.stdout);
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    match (outcome.exit_code, stdout.trim_end()) {
        (0, "T") => "true".to_string(),
        (0, "F") => "false".to_string(),
        _ => stderr
            .lines()
            .find_map(|line| {
                let (code, _) = line.strip_prefix("Error ")?.split_once(':')?;
                (code.contains('.') && !code.contains(' ')).then(|| code.to_string())
            })
            .unwrap_or_else(|| {
                panic!(
                    "{context} over {expression}: rc {} stdout {stdout:?} stderr {stderr:?}",
                    outcome.exit_code
                )
            }),
    }
}

fn class(answer: &str) -> Option<&'static str> {
    match answer {
        "true" => Some("true"),
        "false" => Some("false"),
        code if code.starts_with("34.") => Some("error"),
        _ => None,
    }
}

#[test]
fn every_context_judges_a_value_alike() {
    // Each context's error code, across every case, for the per-context check.
    let mut raised: BTreeMap<&str, BTreeMap<String, String>> = BTreeMap::new();
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            assert_eq!(case.directive, "truth", "unknown directive");
            let expression = case.input.trim_end();
            let answers: Vec<(&str, String)> = CONTEXTS
                .par_iter()
                .map(|(context, program)| (*context, answer(context, program, expression)))
                .collect();
            let judged: Vec<(&str, &'static str)> = answers
                .iter()
                .filter_map(|(context, answer)| Some((*context, class(answer)?)))
                .collect();
            assert!(
                judged.windows(2).all(|pair| pair[0].1 == pair[1].1),
                "the contexts judge {expression} differently: {judged:?}"
            );
            for (context, answer) in &answers {
                if answer.starts_with("34.") {
                    raised
                        .entry(context)
                        .or_default()
                        .insert(answer.clone(), expression.to_string());
                }
            }
            answers
                .iter()
                .map(|(context, answer)| format!("{context}: {answer}\n"))
                .collect::<String>()
        });
    });
    for (context, codes) in &raised {
        assert_eq!(
            codes.len(),
            1,
            "{context} raises more than one sub-number: {codes:?}"
        );
    }
    assert!(!raised.is_empty(), "no case raised");
}
