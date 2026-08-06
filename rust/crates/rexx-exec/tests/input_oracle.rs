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

//! Command-line arguments and a non-empty console, compared against the
//! running oracle.
//!
//! # Why this is a harness of its own
//!
//! `tests/corpus.rs` runs every program with **no arguments and stdin at
//! `/dev/null`**, and cannot do otherwise: a corpus entry is a path and
//! nothing else, and the corpus's one standing rule (`corpus/README.md`, "The
//! one rule: determinism") is that a program's output is the same on every
//! machine, which an interactive console is not. So the whole argument model
//! and every console read past end of input are outside what any corpus
//! program can observe. `corpus/lang/pull_queue.rex` covers what *is* in
//! reach -- the queue, and reading an exhausted console -- and says so in its
//! own header.
//!
//! # Why it drives the binary rather than calling `run_program`
//!
//! Every other differential harness in this crate calls `run_program` in
//! process, and that would be the wrong instrument here, because **the two
//! things under test both live in `bin/rexx-run.rs`**: turning `argv` into one
//! argument string, and choosing `ProgramInput::Stdin` rather than the default
//! that reads nothing. An in-process comparison would supply the joined string
//! and the input bytes itself and so would test neither decision -- it would
//! compare this file's idea of the command line against the oracle's, which is
//! a check on this file. Running the binary compares one whole command line
//! against the other.
//!
//! `rexx_exec::join_command_line`'s own unit tests cover the joining rule row
//! by row against measured oracle bytes; what is added here is that
//! `rexx-run`'s `argv` actually reaches it, and that stdin actually reaches
//! `.input`.
//!
//! # What each case is for
//!
//! [`CASES`] carries the reason per row. The two that no other check in the
//! tree can reach are the **absent versus empty argument** pair -- `rexx p.rex`
//! against `rexx p.rex ""` -- and the **`\r\n` collapse**, and both are the
//! kind of near-miss that produces a plausible wrong answer rather than a
//! failure.
//!
//! # The gate
//!
//! Gated on `REXX_CORPUS_GATE`, the switch `tests/corpus.rs` and
//! `tests/parse_version_oracle.rs` already use, so an offline checkout is not
//! asked to produce an oracle. That protection is partial and saying otherwise
//! would be false: `tests/builtin_status.rs` invokes the oracle with no gate at
//! all, so a machine without the oracle already cannot run this crate's default
//! suite. The gate is followed because it is the convention for a check whose
//! whole subject is a comparison against the oracle.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use support::oracle::{CppOutcome, write_and_wait};

/// Env var that flips this file from a skip into the check.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match std::env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// A directory of this run's own, because an unresolved Rexx name searches the
/// current directory for an external routine -- `support::oracle::Oracle::run`'s
/// own doc has the measured consequence of running a synthesised program from a
/// directory of stale `.rex` files.
fn fresh_run_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("input-oracle-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// One program, one command line, one console, and the reason the row exists.
struct Case {
    /// Names the temporary file and appears in a failure message.
    name: &'static str,
    program: &'static str,
    /// The words after the program path, exactly as `argv` entries.
    args: &'static [&'static str],
    /// The console's bytes. `None` is the harness default of `/dev/null` on
    /// the oracle side and `Stdio::null()` on this crate's, which is a
    /// different state from `Some(b"")`: the second is a pipe closed with
    /// nothing in it.
    stdin: Option<&'static [u8]>,
    /// Why this row is here, and what wrong answer it excludes.
    why: &'static str,
}

/// The program every argument row runs. Prints the argument three ways --
/// through `PARSE ARG`, through the upcasing `ARG` short form, and through
/// `USE ARG` -- so a row's divergence names which reader disagreed.
///
/// `USE ARG` is the one of the three that distinguishes an absent argument
/// from an empty one, because an unbound target reads as its own derived name
/// (`P`) where a target bound to the null string reads as nothing. `PARSE ARG`
/// answers the null string either way.
const ARG_PROGRAM: &str = "\
parse arg n1 n2 n3
say \"parse arg [\" || n1 || \"][\" || n2 || \"][\" || n3 || \"]\"
arg n4 n5 n6
say \"arg       [\" || n4 || \"][\" || n5 || \"][\" || n6 || \"]\"
use arg p
say \"use arg   [\" || p || \"] length \" || length(p)
";

/// The program every console row runs. Interleaves `PARSE PULL` and `PARSE
/// LINEIN` so a row proves they share one position, and reports each line's
/// length and hex so a terminator or case difference cannot hide inside a
/// visually identical string.
const INPUT_PROGRAM: &str = "\
parse pull n1
parse linein n2
pull n3
parse linein n4
say \"1 \" || length(n1) || \" \" || c2x(n1)
say \"2 \" || length(n2) || \" \" || c2x(n2)
say \"3 \" || length(n3) || \" \" || c2x(n3)
say \"4 \" || length(n4) || \" \" || c2x(n4)
";

/// The `USE STRICT ARG` program: the argument model's own error path.
const STRICT_PROGRAM: &str = "\
use strict arg p
say \"strict [\" || p || \"]\"
";

/// Fills the queue, reads it back, and then keeps reading past the end of it
/// into the console.
///
/// `PUSH` inserts at the head and `QUEUE` appends at the tail, so `c`/`b`/`a`
/// written in that order come back `a`, `c`, `b`; only the bare `PULL`
/// upcases. The two reads after that fall through to the console, which is
/// what pins the queue as a store consulted *before* `.input` rather than a
/// buffer in front of it -- with lines on the console, a reader that drained
/// the console first would print them in the first three fields.
const QUEUE_PROGRAM: &str = "\
push \"c\"
queue \"b\"
push \"a\"
pull n1
parse pull n2
parse pull n3
parse pull n4
parse linein n5
say \"[\" || n1 || \"][\" || n2 || \"][\" || n3 || \"][\" || n4 || \"][\" || n5 || \"]\"
";

const CASES: &[Case] = &[
    Case {
        name: "arg-none",
        program: ARG_PROGRAM,
        args: &[],
        stdin: None,
        why: "no argument at all: `use arg p` must leave `p` unbound, so it \
              reads as its own derived name `P`. An implementation that \
              modelled absence as a present empty string prints nothing here \
              and is otherwise indistinguishable",
    },
    Case {
        name: "arg-empty",
        program: ARG_PROGRAM,
        args: &[""],
        stdin: None,
        why: "one EMPTY argument, which is a present argument: the pair with \
              `arg-none` is the whole reason the argument is an `Option`, and \
              nothing else in the tree compares the two against the oracle",
    },
    Case {
        name: "arg-three-words",
        program: ARG_PROGRAM,
        args: &["a", "b", "c"],
        stdin: None,
        why: "three words become ONE argument string `a b c`, so `parse arg \
              n1 n2 n3` splits it back into three and `use arg p` sees all of \
              it. An implementation passing three separate arguments prints \
              the same first line and a different third",
    },
    Case {
        name: "arg-one-quoted-word",
        program: ARG_PROGRAM,
        args: &["a b c"],
        stdin: None,
        why: "the adjacent success for the row above: one quoted word is \
              indistinguishable from three bare ones, which is what makes the \
              single-string model right rather than merely sufficient",
    },
    Case {
        name: "arg-internal-spacing",
        program: ARG_PROGRAM,
        args: &["a  b", "c"],
        stdin: None,
        why: "internal spacing survives the join and the separator is exactly \
              one blank, so `use arg p` reports length 6 for `a  b c` -- a \
              join that normalised whitespace reports 5",
    },
    Case {
        name: "arg-leading-empty-word",
        program: ARG_PROGRAM,
        args: &["", "x"],
        stdin: None,
        why: "the join's own edge: a leading empty word contributes neither \
              text nor separator, so the argument is `x` and not ` x`. This is \
              the row that fails for an implementation that joins the words \
              with a blank, which is what the rule looks like from the other \
              rows",
    },
    Case {
        name: "arg-trailing-empty-word",
        program: ARG_PROGRAM,
        args: &["x", ""],
        stdin: None,
        why: "and its asymmetric partner: a TRAILING empty word does \
              contribute its separator, so the argument is `x ` with length 2. \
              An implementation that skipped empty words entirely passes the \
              row above and fails this one",
    },
    Case {
        name: "arg-commas",
        program: ARG_PROGRAM,
        args: &["a,b,c"],
        stdin: None,
        why: "commas are not argument separators: the whole string lands in \
              the first template field",
    },
    Case {
        name: "arg-mixed-case",
        program: ARG_PROGRAM,
        args: &["mIxEd", "CaSe"],
        stdin: None,
        why: "`ARG template` is `PARSE UPPER ARG template` and `PARSE ARG` is \
              not: the two lines of output differ in case on the same \
              argument. No corpus program can reach this, because with no \
              argument both spellings parse the null string",
    },
    Case {
        name: "strict-none",
        program: STRICT_PROGRAM,
        args: &[],
        stdin: None,
        why: "`use strict arg p` with no argument is Error 40.3, whose \
              substitution is the PROGRAM PATH at this level and not an \
              upcased label -- so this compares the report line by line, not \
              just the exit code",
    },
    Case {
        name: "strict-empty",
        program: STRICT_PROGRAM,
        args: &[""],
        stdin: None,
        why: "the same clause with one empty argument succeeds, rc 0. Paired \
              with `strict-none` this turns the absent/empty distinction into \
              an exit-status difference, which is the strongest form it takes",
    },
    Case {
        name: "input-four-lines",
        program: INPUT_PROGRAM,
        args: &[],
        stdin: Some(b"line-A\nline-B\nline-C\nline-D\n"),
        why: "one shared position: two `PARSE PULL`s and two `PARSE LINEIN`s \
              interleaved consume four consecutive lines. An implementation \
              giving each construct its own reader repeats lines instead",
    },
    Case {
        name: "input-runs-out",
        program: INPUT_PROGRAM,
        args: &[],
        stdin: Some(b"only-one\n"),
        why: "past the last line every read is the null string, not the last \
              line again and not a condition",
    },
    Case {
        name: "input-uppercase-split",
        program: INPUT_PROGRAM,
        args: &[],
        stdin: Some(b"lower one\nlower two\nlower three\nlower four\n"),
        why: "`PULL` upcases and `PARSE PULL`/`PARSE LINEIN` do not, on lines \
              from the console rather than from the queue -- the third line's \
              hex differs from the other three's while the lengths match",
    },
    Case {
        name: "input-crlf",
        program: INPUT_PROGRAM,
        args: &[],
        stdin: Some(b"crlf\r\nplain\nx\r\r\ny\r"),
        why: "the `\\r\\n` collapse, and the two near-misses beside it: one CR \
              is removed from `x\\r\\r\\n` and not both, and a final `y\\r` \
              with no newline after it keeps its CR. Nothing else in the tree \
              can feed these bytes to either interpreter",
    },
    Case {
        name: "input-blanks-and-empty-line",
        program: INPUT_PROGRAM,
        args: &[],
        stdin: Some(b" pad \n\nlast-no-newline"),
        why: "leading and trailing blanks are data, an empty line is the null \
              string, and a final line with no terminator is still a line",
    },
    Case {
        name: "input-nul-byte",
        program: INPUT_PROGRAM,
        args: &[],
        stdin: Some(b"a\x00b\nnext\n"),
        why: "the console is bytes, not text: a NUL is data and does not end \
              the line",
    },
    Case {
        name: "queue-round-trip",
        program: QUEUE_PROGRAM,
        args: &[],
        stdin: Some(b"console-A\nconsole-B\n"),
        why: "the queue's own storage order, read back, against a console \
              holding different lines. `lang/push_queue.rex` states in its own \
              header that it cannot pin which end a line landed on, because it \
              was written before anything read one back; this is that reader. \
              Removing from the tail prints `[B][c][a]`, collapsing PUSH into \
              QUEUE prints `[C][b][a]`, and draining the console before the \
              queue prints the console's lines in the first three fields",
    },
    Case {
        name: "input-closed-pipe",
        program: INPUT_PROGRAM,
        args: &[],
        stdin: Some(b""),
        why: "a pipe closed with nothing in it, which is not the same \
              descriptor as `/dev/null` and must answer the same way. The \
              `None` rows above are the `/dev/null` half",
    },
];

/// Runs `rexx-run` the way a shell would, with the same `argv` and the same
/// console the oracle gets.
///
/// `CARGO_BIN_EXE_rexx-run` is Cargo's own path to the binary built for this
/// test target, so there is no chance of driving a stale copy from `PATH`.
/// No memory limit wrapper: the limit exists for the oracle, which requests
/// gigabytes mid-range without it.
fn run_rust(path: &Path, args: &[&str], stdin: Option<&[u8]>) -> CppOutcome {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rexx-run"));
    command
        .arg(path)
        .args(args)
        .current_dir(path.parent().unwrap_or(Path::new(".")))
        .stdin(match stdin {
            None => Stdio::null(),
            Some(_) => Stdio::piped(),
        });
    let output = match stdin {
        None => command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn rexx-run for {}: {e}", path.display())),
        Some(bytes) => write_and_wait(&mut command, bytes, path),
    };
    CppOutcome {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.status.code().unwrap_or(-1),
    }
}

/// Which of the three channels disagree. A local copy of
/// `support::oracle::descriptor_diffs`'s comparison, because that one takes a
/// `rexx_exec::Outcome` from an in-process run and both sides here are
/// subprocesses.
///
/// DEVIATION 0 applies to stderr the same way it does there: each side's own
/// trace-line indent runs are collapsed before comparing.
fn diffs(rust: &CppOutcome, cpp: &CppOutcome) -> Vec<&'static str> {
    let mut out = Vec::new();
    if rust.stdout != cpp.stdout {
        out.push("stdout");
    }
    if support::normalize_stderr(&rust.stderr) != support::normalize_stderr(&cpp.stderr) {
        out.push("stderr");
    }
    if rust.exit_code != cpp.exit_code {
        out.push("exit code");
    }
    out
}

/// Every row of [`CASES`] agrees with the oracle on all three channels.
#[test]
fn command_line_arguments_and_the_console_agree_with_the_oracle() {
    if !gate_mode() {
        eprintln!(
            "*** SKIPPED -- command-line arguments and a non-empty console are \
             not compared against the oracle unless {GATE_ENV} is set. The \
             corpus harness passes no arguments and an empty stdin, so nothing \
             else in the tree compares either. ***"
        );
        return;
    }

    let dir = fresh_run_dir();
    let oracle = support::oracle::locate();
    let mut failures = Vec::new();

    for case in CASES {
        // One directory per case: the programs differ, and a leftover file
        // from an earlier case is exactly the stale-`.rex` hazard
        // `fresh_run_dir`'s comment is about.
        let case_dir = dir.join(case.name);
        fs::create_dir_all(&case_dir)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", case_dir.display()));
        let file = case_dir.join(format!("{}.rex", case.name));
        fs::write(&file, case.program)
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
        let abs = fs::canonicalize(&file)
            .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));

        let rust = run_rust(&abs, case.args, case.stdin);
        let cpp = oracle.run_with(&abs, case.args, case.stdin);

        let channels = diffs(&rust, &cpp);
        if !channels.is_empty() {
            failures.push(format!(
                "\n  {} [{}]\n    why: {}\n    argv: {:?}  stdin: {:?}\n    \
                 rust:   stdout={:?} stderr={:?} rc={}\n    \
                 oracle: stdout={:?} stderr={:?} rc={}",
                case.name,
                channels.join(", "),
                case.why,
                case.args,
                case.stdin.map(String::from_utf8_lossy),
                String::from_utf8_lossy(&rust.stdout),
                String::from_utf8_lossy(&rust.stderr),
                rust.exit_code,
                String::from_utf8_lossy(&cpp.stdout),
                String::from_utf8_lossy(&cpp.stderr),
                cpp.exit_code,
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} command-line/console cases disagree with the oracle:{}",
        failures.len(),
        CASES.len(),
        failures.join("")
    );
    // The floor is a literal, and it has to be: `oracle.invocations() ==
    // CASES.len()` derives both sides from the same array, so an empty or
    // gutted `CASES` satisfies it having compared nothing at all -- the same
    // shape as a constant compared against itself, one level up. The equality
    // below is still worth keeping beside it: it is what catches a row that
    // silently failed to start the oracle. `>=` rather than `==` so that
    // adding a row is not an edit here.
    assert!(
        oracle.invocations() >= 15,
        "only {} oracle runs, which is fewer than this file has ever had -- \
         rows were removed rather than the harness getting faster, and a \
         shrunken CASES satisfies the equality below by comparing nothing",
        oracle.invocations()
    );
    assert_eq!(
        oracle.invocations(),
        CASES.len(),
        "every case must have actually started the oracle; a comparison \
         against nothing is what the invocation counter exists to exclude"
    );

    // Only on success: a failing run's files are the evidence.
    let _ = fs::remove_dir_all(&dir);
}

/// An unreadable console is end of input, not an error, and both interpreters
/// agree on that.
///
/// Separate from [`CASES`] because it needs a descriptor rather than bytes:
/// stdin is bound to a **directory**, so the `read` itself fails with `EISDIR`
/// rather than reporting end of file. Measured on the oracle before this crate
/// was written to match: the null string, rc 0, empty stderr. That is the whole
/// justification for `Input::read_line` answering `None` for an I/O error, and
/// without this test that answer would be an unwitnessed choice to swallow one.
#[test]
fn an_unreadable_console_is_end_of_input() {
    if !gate_mode() {
        eprintln!(
            "*** SKIPPED -- the unreadable-console comparison needs the oracle \
             and runs only with {GATE_ENV} set. ***"
        );
        return;
    }

    let dir = fresh_run_dir();
    let file = dir.join("unreadable.rex");
    fs::write(&file, INPUT_PROGRAM)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));

    // The directory itself is the descriptor. Opened read-only, which succeeds
    // on Linux for a directory; it is the `read` that fails.
    let as_stdin = || {
        Stdio::from(
            fs::File::open(&dir).unwrap_or_else(|e| panic!("cannot open {}: {e}", dir.display())),
        )
    };

    let oracle = support::oracle::locate();
    let rust = Command::new(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(&abs)
        .current_dir(&dir)
        .stdin(as_stdin())
        .output()
        .expect("rexx-run starts");
    // Through the harness, not a hand-rolled `sh -c` wrapper: the memory limit
    // and the invocation counter are exactly what `run_with_stdin` exists to
    // keep in one place, and the counter is asserted below.
    let cpp = oracle.run_with_stdin(&abs, &[], as_stdin());

    assert_eq!(
        (
            String::from_utf8_lossy(&rust.stdout).into_owned(),
            String::from_utf8_lossy(&rust.stderr).into_owned(),
            rust.status.code()
        ),
        (
            String::from_utf8_lossy(&cpp.stdout).into_owned(),
            String::from_utf8_lossy(&cpp.stderr).into_owned(),
            Some(cpp.exit_code)
        ),
        "an unreadable console must answer the null string on both sides"
    );
    assert!(
        !cpp.stdout.is_empty(),
        "the oracle printed nothing, so this compared two empty outputs and \
         proved nothing about how either side handles an unreadable console"
    );
    assert_eq!(
        oracle.invocations(),
        1,
        "the comparison above must have actually started the oracle"
    );

    let _ = fs::remove_dir_all(&dir);
}
