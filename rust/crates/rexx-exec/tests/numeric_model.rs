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
//! crate every answer agrees except the [`REPORTED`] rows the crate takes
//! part in; on the oracle the model answers as it does here, and the native
//! answer differs exactly on [`ORACLE_DEPARTURES`] and the [`REPORTED`] rows
//! the oracle takes part in.

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

/// The cases whose native oracle answer differs from the model, each with the
/// numeric Deviation (`docs/superpowers/plans/phase-4-exclusions.txt`) that
/// licenses it.
const ORACLE_DEPARTURES: &[(&str, u32)] = &[
    ("max-wide-argument", 44),
    ("max-message-wide-argument", 44),
    ("max-trailing-zeros", 44),
    ("maxstr-string", 44),
    ("maxstr-literal", 44),
    ("maxmin-11", 44),
    ("min-lone-wide", 44),
    ("msg-min-lone-wide", 44),
    ("max-wide-rounds-up", 44),
    ("max-engineering-wide", 44),
    ("do-count-wide", 47),
    ("do-for-wide", 47),
    ("do-count-trailing-zeros", 47),
];

/// Which interpreter answers a [`REPORTED`] case differently from the model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sides {
    Both,
    Oracle,
}

/// The cases where the model follows the rule derived from the documentation
/// and an interpreter departs from it with no Deviation yet ruled: reported to
/// Moritz by Task 6.14 (`.superpowers/sdd/2026-10-18-phase-9d/task-6.14-report.md`).
const REPORTED: &[(&str, Sides)] = &[
    // Under FUZZ a comparison subtracts at DIGITS minus FUZZ, but both
    // interpreters check LOSTDIGITS at that precision only for same-signed
    // operands.
    ("fuzz-18", Sides::Both),
    ("fuzz-sign-differs", Sides::Both),
    ("fuzz-sign-zero", Sides::Both),
    // MAX's whole-number path compares without FUZZ: the function form here
    // and the oracle, and the oracle's message form.
    ("max-fuzz-integers", Sides::Both),
    ("max-message-fuzz-integers", Sides::Oracle),
    // An omitted MAX argument is 93.903 naming argument 0 on the
    // whole-number path, 40.5 otherwise.
    ("max-missing-argument", Sides::Both),
    ("max-message-missing-argument", Sides::Oracle),
    // FORMAT's `expt` trigger for the decimal part of a number with an
    // integer part.
    ("format-decimal-trigger", Sides::Both),
];

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
    let text = program(
        &arg("digits", "9"),
        &arg("fuzz", "0"),
        &arg("form", "scientific"),
        &operation(&case.input),
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

fn reported(name: &str) -> Option<Sides> {
    REPORTED
        .iter()
        .find(|(case, _)| *case == name)
        .map(|(_, sides)| *sides)
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
            let expected_to_differ = reported(&run.name) == Some(Sides::Both);
            assert_eq!(
                run.native != run.model,
                expected_to_differ,
                "[{}] this crate {} the model: native {}, model {}",
                run.name,
                if expected_to_differ {
                    "was listed as departing from"
                } else {
                    "departs from"
                },
                run.native,
                run.model
            );
            recorded(&run)
        });
    });
    for (name, _) in REPORTED {
        assert!(names.contains(*name), "REPORTED names no case {name}");
    }
    for (name, _) in ORACLE_DEPARTURES {
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
            if native != model {
                departures.insert(run.name.clone());
            }
            recorded(&run)
        });
    });
    let expected: BTreeSet<String> = ORACLE_DEPARTURES
        .iter()
        .map(|(name, _)| (*name).to_string())
        .chain(REPORTED.iter().map(|(name, _)| (*name).to_string()))
        .collect();
    assert_eq!(
        departures, expected,
        "the oracle's departures from the model are not the listed set"
    );
    let deviations: BTreeSet<u32> = ORACLE_DEPARTURES.iter().map(|(_, d)| *d).collect();
    assert_eq!(deviations, BTreeSet::from([44, 47]));
}
