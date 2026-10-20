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

//! The numeric model, `corpus/model/numeric.rex`, against the native
//! operation on the same inputs: each case in `tests/numeric_model/` runs
//! once untrapped and once under `SIGNAL ON LOSTDIGITS`, both ways. On this
//! crate every answer agrees; on the oracle the model answers as it does
//! here, and the native answer differs exactly on [`ORACLE_DEPARTURES`], as
//! each row records it.

#![expect(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rexx_exec::{Invocation, run_program};
use support::oracle::{did_not_finish, locate};

const CASES: &str = "tests/numeric_model";

/// The cases whose native oracle answer differs from the model: the case,
/// the numeric Deviation that licenses it, the rule it rests on, and the
/// oracle's answer, untrapped and then trapped. The rules R1-R3 and R6 are
/// Moritz's 2026-10-20 rulings ("follow rexxref") reported by Task 6.14:
/// * `R1`: under FUZZ a comparison checks LOSTDIGITS at DIGITS minus FUZZ
///   whatever the signs;
/// * `R2`: MAX and MIN apply FUZZ to whole numbers;
/// * `R3`: an omitted MAX or MIN argument is 40.5 for the function and
///   93.903 for the message, whatever the other arguments are;
/// * `R6`: a zero dividend still checks its divisor for LOSTDIGITS;
/// * `description`: the oracle names an unquoted literal's LOSTDIGITS by its
///   spelling at parse time, and this crate by its value;
/// * `wide-integer`: MAX and MIN round a whole number wider than DIGITS;
/// * `written-value`: MAX and MIN decide LOSTDIGITS by how the values are
///   written, not by whether the oracle holds them as integers;
/// * `do-count`: a DO count wider than DIGITS raises 26.2 or 26.3.
const ORACLE_DEPARTURES: &[(&str, u32, &str, &str)] = &[
    (
        "max-wide-argument",
        44,
        "wide-integer",
        "V[1111111111] | V[1111111111]",
    ),
    (
        "max-message-wide-argument",
        44,
        "wide-integer",
        "V[1111111111] | V[1111111111]",
    ),
    (
        "max-trailing-zeros",
        44,
        "wide-integer",
        "V[1000000000] | V[1000000000]",
    ),
    ("do-count-wide", 47, "do-count", "V[ran] | V[ran]"),
    ("do-for-wide", 47, "do-count", "V[ran] | V[ran]"),
    ("do-count-trailing-zeros", 47, "do-count", "V[ran] | V[ran]"),
    (
        "maxstr-string",
        44,
        "written-value",
        "V[1.11111111E+9] | L[1111111111]",
    ),
    (
        "maxstr-literal",
        44,
        "wide-integer",
        "V[1111111111] | V[1111111111]",
    ),
    (
        "maxmin-11",
        44,
        "wide-integer",
        "V[1111111111] | V[1111111111]",
    ),
    ("min-lone-wide", 44, "wide-integer", "V[12345] | V[12345]"),
    (
        "msg-min-lone-wide",
        44,
        "wide-integer",
        "V[12345] | V[12345]",
    ),
    (
        "max-wide-rounds-up",
        44,
        "wide-integer",
        "V[1111111115] | V[1111111115]",
    ),
    (
        "max-engineering-wide",
        44,
        "wide-integer",
        "V[11111111111] | V[11111111111]",
    ),
    ("fuzz-18", 53, "R1", "V[1] | V[1]"),
    ("fuzz-sign-differs", 53, "R1", "V[0] | V[0]"),
    ("fuzz-sign-zero", 53, "R1", "V[1] | V[1]"),
    ("max-fuzz-integers", 53, "R2", "V[100000002] | V[100000002]"),
    (
        "max-message-fuzz-integers",
        53,
        "R2",
        "V[100000002] | V[100000002]",
    ),
    ("max-missing-argument", 53, "R3", "S 93.903 | S 93.903"),
    (
        "max-fuzz-strings",
        44,
        "written-value",
        "V[100000001] | L[100000002]",
    ),
    (
        "min-fuzz-strings",
        44,
        "written-value",
        "V[100000002] | L[100000001]",
    ),
    (
        "max-fuzz-strings-short",
        44,
        "written-value",
        "V[11] | L[12]",
    ),
    (
        "max-message-fuzz-strings",
        44,
        "written-value",
        "V[100000001] | L[100000002]",
    ),
    ("min-missing-argument", 53, "R3", "S 93.903 | S 93.903"),
    ("div-zero-dividend", 53, "R6", "V[0] | V[0]"),
    ("div-zero-dividend-string", 53, "R6", "V[0] | V[0]"),
    ("intdiv-zero-dividend", 53, "R6", "V[0] | V[0]"),
    ("rem-zero-dividend", 53, "R6", "V[0] | V[0]"),
    (
        "describe-prefix-literal",
        53,
        "description",
        "V[-1.23456789E+9] | L[1.2345678905E+9]",
    ),
    (
        "describe-mul-literal",
        53,
        "description",
        "V[1.23456789E+9] | L[1.2345678905E+9]",
    ),
    (
        "describe-pow-literal",
        53,
        "description",
        "V[1.23456789E+9] | L[1.2345678905E+9]",
    ),
    (
        "describe-intdiv-literal",
        53,
        "description",
        "S 26.11 | L[1.2345678905E+9]",
    ),
    (
        "describe-max-literal",
        53,
        "description",
        "V[1.23E+4] | L[12345.5]",
    ),
    (
        "describe-do-literal",
        53,
        "description",
        "S 26.2 | L[12345.5]",
    ),
    (
        "describe-max-trailing",
        53,
        "description",
        "V[2E+1] | L[16.0]",
    ),
    (
        "min-negative-wide-string",
        44,
        "written-value",
        "V[-1.11111111E+9] | L[-1111111111]",
    ),
    ("max-message-missing-first", 53, "R3", "S 40.5 | S 40.5"),
    ("max-message-missing-middle", 53, "R3", "S 40.5 | S 40.5"),
    ("min-message-missing-first", 53, "R3", "S 40.5 | S 40.5"),
    ("fuzz-sign-negative", 53, "R1", "V[1] | V[1]"),
    ("fuzz-zero-left", 53, "R1", "V[1] | V[1]"),
    (
        "fuzz-engineering-integer",
        44,
        "wide-integer",
        "V[1240] | V[1240]",
    ),
];

/// The Deviation each rule's departures are licensed under.
fn deviation_of(rule: &str) -> u32 {
    match rule {
        "R1" | "R2" | "R3" | "R6" | "description" => 53,
        "wide-integer" | "written-value" => 44,
        "do-count" => 47,
        other => panic!("unknown rule {other}"),
    }
}

/// One case's operation, from its input line.
struct Operation<'a> {
    kind: &'a str,
    name: &'a str,
    fields: Vec<&'a str>,
}

fn operation(input: &str) -> Operation<'_> {
    let mut parts = input.trim_end().split('|');
    let head = parts.next().expect("a case has a head");
    let mut words = head.split_whitespace();
    let kind = words.next().expect("a case names its kind");
    let name = words.next().expect("a case names its operation");
    let fields = parts.map(str::trim).collect();
    Operation { kind, name, fields }
}

/// The Rexx program that answers one case natively and through the model,
/// as two lines, `native:` and `model:`.
fn program(digits: &str, fuzz: &str, form: &str, op: &Operation<'_>) -> String {
    let f = &op.fields;
    let call_args = |fields: &[&str]| fields.join(", ").replace(",  ", ", ").replace(", ,", ",,");
    let body = match op.kind {
        "infix" => format!("  return 'V[' || ({} {} {}) || ']'\n", f[0], op.name, f[1]),
        "prefix" => format!("  return 'V[' || ({} {}) || ']'\n", op.name, f[0]),
        "bif" => format!(
            "  return 'V[' || ({}({})) || ']'\n",
            op.name.to_lowercase(),
            call_args(f)
        ),
        "msg" if f.len() > 1 => format!(
            "  return 'V[' || ({}~{}({})) || ']'\n",
            f[0],
            op.name.to_lowercase(),
            call_args(&f[1..])
        ),
        "msg" => format!(
            "  return 'V[' || ({}~{}) || ']'\n",
            f[0],
            op.name.to_lowercase()
        ),
        "do" => {
            let header = match op.name {
                "count" => format!("do {}", f[0]),
                "for" => format!("do i = 1 for {}", f[0]),
                other => panic!("unknown do case {other}"),
            };
            format!(
                "  r = 'none'\n  {header}\n    r = 'ran'\n    leave\n  end\n  return 'V['r']'\n"
            )
        }
        other => panic!("unknown case kind {other}"),
    };
    let sets: String = f
        .iter()
        .enumerate()
        .filter(|(_, field)| !field.is_empty())
        .map(|(index, field)| format!("a[{}] = {field}\n", index + 1))
        .collect();
    let model = |trapped: u8| {
        format!(
            "numeric_eval({trapped}, {digits}, {fuzz}, '{form}', '{}', '{}', {}, a)",
            op.kind,
            op.name,
            f.len()
        )
    };
    format!(
        "numeric digits {digits}\nnumeric fuzz {fuzz}\nnumeric form {form}\n\
         a = .array~new\n{sets}\
         say 'native:' plain() '|' trapped()\n\
         say 'model:' {} '|' {}\n\
         exit\n\
         plain:\n  signal on syntax name plain_syntax\n{body}\
         plain_syntax:\n  return 'S' condition('O')~code\n\
         trapped:\n  signal on syntax name trapped_syntax\n  \
         signal on lostdigits name trapped_lost\n{body}\
         trapped_syntax:\n  return 'S' condition('O')~code\n\
         trapped_lost:\n  return 'L[' || condition('D') || ']'\n\
         ::requires 'numeric.rex'\n",
        model(0),
        model(1)
    )
}

/// A fresh directory holding the case's program beside a copy of the model,
/// and the program's canonical path.
fn materialise(name: &str, text: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "numeric-model-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    let model = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/model/numeric.rex");
    fs::copy(&model, dir.join("numeric.rex"))
        .unwrap_or_else(|e| panic!("cannot copy {}: {e}", model.display()));
    let file = dir.join("case.rex");
    fs::write(&file, text).unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    fs::canonicalize(&file).unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()))
}

/// The `native:` and `model:` answers of one run's stdout.
fn answers(name: &str, side: &str, stdout: &[u8], stderr: &[u8]) -> (String, String) {
    assert!(
        stderr.is_empty(),
        "[{name}] {side} wrote stderr: {}",
        String::from_utf8_lossy(stderr)
    );
    let text = String::from_utf8_lossy(stdout);
    let line = |prefix: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(prefix))
            .unwrap_or_else(|| panic!("[{name}] {side} printed no `{prefix}` line: {text}"))
            .to_string()
    };
    (line("native: "), line("model: "))
}

/// One case run here: its name, the program's path, and its two answers.
struct CrateRun {
    name: String,
    kind: String,
    path: PathBuf,
    native: String,
    model: String,
}

fn run_case(case: &datadriven::TestCase) -> CrateRun {
    assert_eq!(
        case.directive, "case",
        "unknown directive {}",
        case.directive
    );
    let arg = |key: &str, default: &'static str| {
        case.args
            .get(key)
            .map_or(default.to_string(), |values| values[0].clone())
    };
    let name = arg("name", "");
    assert!(!name.is_empty(), "a case without a name: {}", case.input);
    let op = operation(&case.input);
    let kind = op.kind.to_string();
    let text = program(
        &arg("digits", "9"),
        &arg("fuzz", "0"),
        &arg("form", "scientific"),
        &op,
    );
    let path = materialise(&name, &text);
    let path_str = path.to_str().expect("the case path is UTF-8");
    let outcome = run_program(
        path_str,
        text.into_bytes(),
        Invocation::none().with_deadline(support::oracle::RUN_DEADLINE),
    );
    assert_eq!(outcome.exit_code, 0, "[{name}] this crate's exit status");
    let (native, model) = answers(&name, "this crate", &outcome.stdout, &outcome.stderr);
    CrateRun {
        name,
        kind,
        path,
        native,
        model,
    }
}

/// What a case file records: the model's answer, and this crate's where it
/// differs.
fn recorded(run: &CrateRun) -> String {
    let mut out = format!("model: {}\n", run.model);
    if run.native != run.model {
        out.push_str(&format!("crate: {}\n", run.native));
    }
    out
}

#[test]
fn the_model_answers_every_case_as_this_crate_does() {
    let mut names = BTreeSet::new();
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            let run = run_case(case);
            assert!(
                names.insert(run.name.clone()),
                "two cases named {}",
                run.name
            );
            assert_eq!(
                run.native, run.model,
                "[{}] this crate departs from the model",
                run.name
            );
            recorded(&run)
        });
    });
    for (name, ..) in ORACLE_DEPARTURES {
        assert!(
            names.contains(*name),
            "ORACLE_DEPARTURES names no case {name}"
        );
    }
}

#[test]
fn on_the_oracle_the_model_parts_only_where_the_numeric_deviations_say() {
    let oracle = locate();
    let mut departures = BTreeSet::new();
    datadriven::walk(CASES, |file| {
        file.run(|case| {
            let run = run_case(case);
            let dir = run
                .path
                .parent()
                .expect("the case sits in its own directory");
            let cpp = oracle.run_in(&run.path, dir, &[]);
            assert!(
                !did_not_finish(&cpp),
                "[{}] the oracle did not finish: {:?}",
                run.name,
                cpp.termination
            );
            let (native, model) = answers(&run.name, "the oracle", &cpp.stdout, &cpp.stderr);
            assert_eq!(
                model, run.model,
                "[{}] the model answers differently on the oracle",
                run.name
            );
            let listed = ORACLE_DEPARTURES
                .iter()
                .find(|(name, ..)| *name == run.name);
            match listed {
                None => assert_eq!(
                    native, model,
                    "[{}] the oracle departs from the model unlisted",
                    run.name
                ),
                Some((_, deviation, rule, answer)) => {
                    assert_ne!(native, model, "[{}] listed, but agrees", run.name);
                    assert_eq!(&native, answer, "[{}] the oracle's answer moved", run.name);
                    assert_eq!(*deviation, deviation_of(rule), "[{}] mislabelled", run.name);
                    let kinds: &[&str] = match deviation {
                        47 => &["do"],
                        44 => &["bif", "msg"],
                        _ => &["infix", "prefix", "bif", "msg", "do"],
                    };
                    assert!(
                        kinds.contains(&run.kind.as_str()),
                        "[{}] Deviation {deviation} cannot license a `{}` case",
                        run.name,
                        run.kind
                    );
                }
            }
            if native != model {
                departures.insert(run.name.clone());
            }
            recorded(&run)
        });
    });
    let expected: BTreeSet<String> = ORACLE_DEPARTURES
        .iter()
        .map(|(name, ..)| (*name).to_string())
        .collect();
    assert_eq!(
        departures, expected,
        "the oracle's departures from the model are not the listed set"
    );
    let deviations: BTreeSet<u32> = ORACLE_DEPARTURES.iter().map(|(_, d, ..)| *d).collect();
    assert_eq!(deviations, BTreeSet::from([44, 47, 53]));
}
