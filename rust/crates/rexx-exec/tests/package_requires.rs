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

//! The `::REQUIRES` search, the cycle report and `::OPTIONS NOPROLOG`, each
//! against the oracle.
//!
//! **Kept beside `corpus/phase-5d.txt`, which names
//! `lang/package_requires.rex`.** That program is the witness for what
//! `::REQUIRES` imports, and the differential runs it. What this binary has
//! that no corpus program can express: every row here sets a **working
//! directory and two environment variables** of its own and writes the
//! required files into directories it creates, which is the only way to
//! separate the four search routes from each other.
//!
//! **The required files are `.cls`** here and in the corpus, because every
//! corpus scan selects `*.rex`; the spelling also exercises the extension step
//! the search tries first. `corpus/README.md`'s "Two shapes work" is the rule.
//!
//! Three descriptors compared separately and raw.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use support::oracle::locate;

/// One route the search covers, as a directory holding the required file and
/// the answer that directory's copy gives.
struct Route {
    /// The subdirectory the file is written into, `None` for the row where it
    /// is written nowhere.
    holder: Option<&'static str>,
    /// What the crate and the oracle must both print, or the empty string for
    /// the row where neither finds a file.
    answer: &'static str,
}

/// The four routes, the precedence between them, and the refusal when none
/// holds the file.
///
/// **The order is the oracle's own, measured one route at a time**
/// (`SysSearchPath`, `platform/unix/SysInterpreterInstance.cpp:123`): the
/// requiring program's own directory, then the current directory, then
/// `REXX_PATH`, then `PATH`. The `all` row is what says the first of them
/// wins; without it four rows that each find their own copy would pass under
/// a search that consulted only one of the four, since no row would ever have
/// two candidates.
const ROUTES: &[(&str, Route)] = &[
    (
        "progdir",
        Route {
            holder: Some("prog"),
            answer: "program directory",
        },
    ),
    (
        "cwd",
        Route {
            holder: Some("cwd"),
            answer: "current directory",
        },
    ),
    (
        "rexxpath",
        Route {
            holder: Some("rexxpath"),
            answer: "REXX_PATH",
        },
    ),
    (
        "syspath",
        Route {
            holder: Some("syspath"),
            answer: "PATH",
        },
    ),
    (
        "none",
        Route {
            holder: None,
            answer: "",
        },
    ),
];

/// What stays on `PATH` behind the directory each row plants its file in.
///
/// The oracle is spawned through `sh`, which `Command::new` looks up on the
/// child's own `PATH` -- so a row that replaced the variable outright would
/// fail to start the interpreter rather than measure its search. Neither
/// directory holds a `reqlib.cls`, and both sides are given the identical
/// string, so what the row varies is still one directory.
const SHELL_PATH: &str = "/bin:/usr/bin";

/// A directory of this run's own, so that a stale copy of the required file
/// from an earlier run cannot answer a route this one meant to leave empty.
fn fresh_root(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("requires-{name}-{}-{nanos}", std::process::id()));
    for sub in ["prog", "cwd", "rexxpath", "syspath"] {
        let dir = root.join(sub);
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    }
    root
}

/// Runs `rexx-run` on `path` from `cwd` with `environment` set, the same three
/// descriptors the oracle side answers with.
///
/// A subprocess rather than `run_program` in this process, and the reason is
/// the subject: the routes are a working directory and two environment
/// variables, and setting either of those in-process is `unsafe` in this
/// edition and would reach every other test in the binary besides.
fn run_rust_in(path: &Path, cwd: &Path, environment: &[(&str, &str)]) -> (Vec<u8>, Vec<u8>, i32) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rexx-run"));
    command.arg(path).current_dir(cwd);
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn rexx-run for {}: {e}", path.display()));
    (
        output.stdout,
        output.stderr,
        output.status.code().unwrap_or(-1),
    )
}

/// Each of the four routes finds the required file on its own, the first of
/// them wins when all four hold a copy, and a name no route holds is the
/// oracle's own 43.901 rather than anything quieter.
///
/// **The two sides are compared on all three descriptors and the answer is
/// asserted as well**, because a search this crate had narrowed to one route
/// and an oracle that found the file elsewhere would still agree on a row
/// where only that one route holds it -- and a row where neither finds
/// anything agrees on two refusals. The `answer` column is what separates
/// "both found the same file" from "both found nothing".
#[test]
fn each_of_the_four_search_routes_finds_the_required_file() {
    let oracle = locate();
    for (name, route) in ROUTES {
        let root = fresh_root(name);
        let program = root.join("prog/main.rex");
        fs::write(&program, b"say route()\n::requires 'reqlib'\n")
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", program.display()));
        if let Some(holder) = route.holder {
            let file = root.join(holder).join("reqlib.cls");
            fs::write(
                &file,
                format!("::routine route public\n  return '{}'\n", route.answer),
            )
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
        }
        let program = fs::canonicalize(&program)
            .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", program.display()));
        let cwd = root.join("cwd");
        let environment = [
            ("REXX_PATH", root.join("rexxpath").display().to_string()),
            (
                "PATH",
                format!("{}:{SHELL_PATH}", root.join("syspath").display()),
            ),
        ];
        let environment: Vec<(&str, &str)> = environment
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();

        let cpp = oracle.run_in(&program, &cwd, &environment);
        let (stdout, stderr, exit_code) = run_rust_in(&program, &cwd, &environment);
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&cpp.stdout),
            "route {name}: stdout differs from the oracle"
        );
        assert_eq!(
            String::from_utf8_lossy(&stderr),
            String::from_utf8_lossy(&cpp.stderr),
            "route {name}: stderr differs from the oracle"
        );
        assert_eq!(
            exit_code,
            cpp.expect_exit_code(),
            "route {name}: exit status differs from the oracle"
        );
        match route.holder {
            Some(_) => assert_eq!(
                String::from_utf8_lossy(&stdout),
                format!("{}\n", route.answer),
                "route {name}: the file the search found is not the one this row planted"
            ),
            None => {
                assert_eq!(exit_code, 213, "route {name}: exit status");
                assert!(
                    String::from_utf8_lossy(&stderr)
                        .contains("Error 43.901:  Could not find file \"reqlib\" for ::REQUIRES."),
                    "route {name}: stderr {:?}",
                    String::from_utf8_lossy(&stderr)
                );
            }
        }
    }
}

/// With every route holding a copy, the requiring program's own directory is
/// the one that answers.
#[test]
fn the_earliest_route_wins_when_every_one_of_them_holds_a_copy() {
    let oracle = locate();
    let root = fresh_root("all");
    let program = root.join("prog/main.rex");
    fs::write(&program, b"say route()\n::requires 'reqlib'\n")
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", program.display()));
    for (_, route) in ROUTES {
        let Some(holder) = route.holder else { continue };
        let file = root.join(holder).join("reqlib.cls");
        fs::write(
            &file,
            format!("::routine route public\n  return '{}'\n", route.answer),
        )
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    }
    let program = fs::canonicalize(&program)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", program.display()));
    let cwd = root.join("cwd");
    let environment = [
        ("REXX_PATH", root.join("rexxpath").display().to_string()),
        (
            "PATH",
            format!("{}:{SHELL_PATH}", root.join("syspath").display()),
        ),
    ];
    let environment: Vec<(&str, &str)> = environment
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();

    let cpp = oracle.run_in(&program, &cwd, &environment);
    let (stdout, stderr, exit_code) = run_rust_in(&program, &cwd, &environment);
    assert_eq!(
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&cpp.stdout),
        "stdout differs from the oracle"
    );
    assert_eq!(
        String::from_utf8_lossy(&stderr),
        String::from_utf8_lossy(&cpp.stderr),
        "stderr differs from the oracle"
    );
    assert_eq!(
        exit_code,
        cpp.expect_exit_code(),
        "exit status differs from the oracle"
    );
    assert_eq!(
        String::from_utf8_lossy(&stdout),
        "program directory\n",
        "the requiring program's own directory did not win"
    );
}

/// A `::REQUIRES` chain that returns to a package whose own directives are
/// still installing is the oracle's 98.952, with one clause echo per level.
///
/// **The whole chain, not only the message**: the report names the file the
/// second reference resolved to and the level it was reached from, and a
/// crate that detected the cycle at the wrong level would give the same
/// message under a different `running <name> line <n>` span.
#[test]
fn a_requires_cycle_is_the_oracles_own_report() {
    let oracle = locate();
    let root = fresh_root("cycle");
    let dir = root.join("prog");
    for (name, body) in [
        ("main.rex", "say 'main'\n::requires 'a.rex'\n"),
        ("a.rex", "say 'a'\n::requires 'b.rex'\n"),
        ("b.rex", "say 'b'\n::requires 'a.rex'\n"),
    ] {
        let file = dir.join(name);
        fs::write(&file, body).unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    }
    let program = fs::canonicalize(dir.join("main.rex"))
        .unwrap_or_else(|e| panic!("cannot resolve the cycle's entry program: {e}"));

    let cpp = oracle.run_in(&program, &dir, &[]);
    let (stdout, stderr, exit_code) = run_rust_in(&program, &dir, &[]);
    assert_eq!(
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&cpp.stdout),
        "stdout differs from the oracle"
    );
    assert_eq!(
        String::from_utf8_lossy(&stderr),
        String::from_utf8_lossy(&cpp.stderr),
        "stderr differs from the oracle"
    );
    assert_eq!(
        exit_code,
        cpp.expect_exit_code(),
        "exit status differs from the oracle"
    );
    let stderr = String::from_utf8_lossy(&stderr).into_owned();
    assert_eq!(
        exit_code,
        158,
        "the cycle did not raise: stdout {:?} stderr {stderr:?}",
        String::from_utf8_lossy(&stdout)
    );
    assert_eq!(
        stderr.matches("*-* ::requires").count(),
        3,
        "one clause echo per level of the chain: {stderr}"
    );
    assert!(
        stderr.contains("Error 98.952:  Circular ::REQUIRES references detected with ")
            && stderr.contains(&format!("{}/a.rex.", dir.display())),
        "the report does not name the file the cycle closed on: {stderr}"
    );
}

/// `::OPTIONS NOPROLOG` suppresses the leading code section of a required
/// file and leaves its directives installed, and does nothing at all to a
/// file run as the program.
///
/// **The pair is the point.** A crate that ignored the keyword agrees with
/// the oracle on the second row and prints one line too many on the first; a
/// crate that suppressed the section everywhere agrees on the first and
/// prints one too few on the second.
#[test]
fn noprolog_suppresses_a_required_files_prologue_and_not_a_programs_own() {
    let oracle = locate();
    let root = fresh_root("noprolog");
    let dir = root.join("prog");
    let required = dir.join("quiet.rex");
    fs::write(
        &required,
        "say 'the prologue ran'\n\
         ::options noprolog\n\
         ::routine reachable public\n  return 'the directives installed'\n",
    )
    .unwrap_or_else(|e| panic!("cannot write {}: {e}", required.display()));
    let program = dir.join("main.rex");
    fs::write(&program, "say reachable()\n::requires 'quiet.rex'\n")
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", program.display()));

    for (label, path) in [("required", &required), ("program", &program)] {
        let path = fs::canonicalize(path)
            .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", path.display()));
        let cpp = oracle.run_in(&path, &dir, &[]);
        let (stdout, stderr, exit_code) = run_rust_in(&path, &dir, &[]);
        assert_eq!(
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&cpp.stdout),
            "{label}: stdout differs from the oracle"
        );
        assert_eq!(
            String::from_utf8_lossy(&stderr),
            String::from_utf8_lossy(&cpp.stderr),
            "{label}: stderr differs from the oracle"
        );
        assert_eq!(
            exit_code,
            cpp.expect_exit_code(),
            "{label}: exit status differs from the oracle"
        );
    }

    let (stdout, _, _) = run_rust_in(
        &fs::canonicalize(&program).expect("the requiring program resolves"),
        &dir,
        &[],
    );
    assert_eq!(
        String::from_utf8_lossy(&stdout),
        "the directives installed\n",
        "the required file's prologue ran, or its directives did not install"
    );
    let (stdout, _, _) = run_rust_in(
        &fs::canonicalize(&required).expect("the required file resolves"),
        &dir,
        &[],
    );
    assert_eq!(
        String::from_utf8_lossy(&stdout),
        "the prologue ran\n",
        "the keyword suppressed the leading section of the program itself"
    );
}
