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

//! Running a Rexx program through the built C++ interpreter, and comparing
//! its three observable channels against this crate's own.
//!
//! **One copy, shared by every differential harness.** `tests/corpus.rs`
//! wrote this first and was its only user; `tests/builtin_status.rs` needs
//! the identical locate-and-run-under-a-memory-limit behaviour, and a second
//! copy of a subprocess wrapper is exactly the "one quantity, two
//! formatters" shape `src/trace.rs`'s module doc records the cost of. Moving
//! it here rather than making the second caller reimplement it is what keeps
//! the memory limit, the missing-binary failure and the DEVIATION 0
//! normalisation the same on both.
//!
//! # The oracle
//!
//! `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, hardcoded rather than made
//! configurable: the entire point of a differential test is "does the
//! executor agree with *this* build", and an env var that could point it at a
//! different one would let a stale binary answer for the current oracle with
//! nothing to notice. If that binary is missing, [`locate`] **fails**,
//! loudly, rather than skipping: a machine without the oracle reporting "0 of
//! 0 matching" and going green would be indistinguishable from a machine with
//! the oracle and a fully-passing corpus, which is exactly the
//! silent-vacuous-harness shape this project keeps finding in its own
//! instruments. A failure names the missing path and what to do about it;
//! nothing here can go green by accident.
//!
//! # The memory limit
//!
//! Every oracle invocation is wrapped as `sh -c 'ulimit -v <KiB> && exec "$0"
//! "$@"' <binary> <args...>`, matching the `( ulimit -v 1048576; ... )` this
//! project runs by hand everywhere else it touches the oracle. Without it the
//! interpreter requests gigabytes mid-range and is OOM-killed.
//! `std::process::Command` has no direct rlimit hook; the alternative is an
//! `unsafe` `pre_exec` closure calling `setrlimit`, which the workspace
//! forbids (`unsafe_code = "forbid"`) and which buys nothing a shell builtin
//! does not already do for free. Verified directly, outside this test: `sh -c
//! 'ulimit -v 1048576 && exec "$0" "$@"' python3 -c 'bytearray(2 * 1024 *
//! 1024 * 1024)'` raises `MemoryError` under the limit and does not without
//! it, and the same wrapper still runs an ordinary corpus program (`say
//! 1/3`-shaped `arith_digits.rex`) to rc 0. The `"$0" "$@"` form passes the
//! binary and its arguments as separate `argv` entries rather than
//! interpolating them into the shell string, so no path needs escaping.
//!
//! # Why the invocation count is a field
//!
//! [`Oracle`] counts its own runs, and a caller can assert the total. A
//! differential harness that classifies names rather than programs -- "if the
//! name is in this table it is implemented, otherwise it is not" -- satisfies
//! every set-equality and count assertion a status file can carry while
//! running no program at all. The one thing such a classifier cannot fake is
//! having started a subprocess, so the count is kept where the subprocess is
//! started rather than where the loop is written.
//!
//! # stdin is never the terminal
//!
//! Both interpreters are given an empty stdin unless a caller supplies bytes
//! for it ([`Oracle::run_with`]). A probe that read a line from the runner's
//! own descriptor would otherwise block forever under a test harness, or --
//! worse -- consume whatever that descriptor happened to hold, which is not
//! the same on two machines. Supplying bytes explicitly is the only way any
//! caller here gets a non-empty input, and the executor side of that is
//! `rexx_exec::ProgramInput`, whose own doc makes the same argument about the
//! in-process callers.

// This module is pulled in by `mod support;` in more than one integration
// test binary, and each one links only the part of it that binary uses. A
// helper used by one harness and not another is therefore "dead" from the
// other's point of view, which is a fact about Cargo's test-target model
// rather than about this code. `tests/owners.rs` carries the same attribute
// for the same reason.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use rexx_exec::Outcome;

/// Address-space ceiling imposed on every oracle invocation, in KiB. 1 GiB:
/// the figure this project has used by hand throughout Phase 4, restated
/// here as a named constant rather than a magic number in the format string.
pub const ORACLE_MEMORY_LIMIT_KIB: u64 = 1_048_576;

/// Root of the built C++ oracle. See the module doc for why this is
/// hardcoded rather than read from an env var.
pub fn oracle_root() -> PathBuf {
    PathBuf::from("/home/moritz/dev/repos/ooRexx/build")
}

/// The oracle binary, the library directory it needs on `LD_LIBRARY_PATH`,
/// and how many programs have been run through it.
pub struct Oracle {
    binary: PathBuf,
    lib_dir: PathBuf,
    invocations: AtomicUsize,
}

/// What one oracle run produced. Deliberately not [`rexx_exec::Outcome`]:
/// that type carries a `stack: StackSpan` field this process never measures,
/// and reusing it would invite comparing a field that was never filled in.
pub struct CppOutcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

/// Locates the oracle, or fails the test naming exactly what is missing.
///
/// A failure here, not a skip: see the module doc's "The oracle" section for
/// why a missing binary must never let a differential test go green having
/// compared nothing.
pub fn locate() -> Oracle {
    let root = oracle_root();
    let binary = root.join("bin/rexx");
    let lib_dir = root.join("lib");
    assert!(
        binary.is_file(),
        "the oracle binary is missing at {}. This test compares the executor \
         against a built ooRexx C++ interpreter; without it there is nothing \
         to compare against, and a machine reporting \"0 of 0 matching\" here \
         would look identical to one where every program actually passed. \
         Build ooRexx there first.",
        binary.display()
    );
    Oracle {
        binary,
        lib_dir,
        invocations: AtomicUsize::new(0),
    }
}

impl Oracle {
    /// Runs `path` through the oracle under the memory limit, from `path`'s
    /// own directory. See the module doc for the mechanism and how it was
    /// verified.
    ///
    /// The working directory matters and is not incidental: a Rexx call to
    /// an unresolved name searches the current directory for an external
    /// routine, so a program run from a directory holding unrelated `.rex`
    /// files can execute one of them instead of failing. Measured on this
    /// host, the same program reported error 44.1 rc 212 from a directory of
    /// stale probes and 43.1 rc 213 from an empty one -- a different error, a
    /// different exit status and a different meaning. Callers that synthesise
    /// a program are therefore expected to give it a directory of its own.
    pub fn run(&self, path: &Path) -> CppOutcome {
        self.run_with(path, &[], None)
    }

    /// [`Oracle::run`], with command-line words after the program path and a
    /// choice of standard input.
    ///
    /// `args` are passed as separate `argv` entries, which the `"$0" "$@"`
    /// wrapper the memory limit already needs forwards for free -- so nothing
    /// here has to be escaped and the limit and the invocation count stay in
    /// one place rather than being duplicated for a second entry point.
    ///
    /// `stdin` is `None` for the module doc's "stdin is never the terminal"
    /// default, which stays a literal `Stdio::null()` rather than an
    /// immediately-closed pipe: `/dev/null` is a seekable regular-ish
    /// descriptor and a pipe is not, and `run`'s own behaviour must not change
    /// shape because this method was added beside it.
    pub fn run_with(&self, path: &Path, args: &[&str], stdin: Option<&[u8]>) -> CppOutcome {
        self.invocations.fetch_add(1, Ordering::Relaxed);
        let cwd = path.parent().unwrap_or(Path::new("."));
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(format!(
                "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" \"$@\""
            ))
            .arg(&self.binary)
            .arg(path)
            .args(args)
            .current_dir(cwd)
            .env("LD_LIBRARY_PATH", &self.lib_dir)
            .stdin(match stdin {
                None => Stdio::null(),
                Some(_) => Stdio::piped(),
            });
        let output = match stdin {
            None => command.output().unwrap_or_else(|e| {
                panic!("failed to spawn the oracle for {}: {e}", path.display())
            }),
            Some(bytes) => write_and_wait(&mut command, bytes, path),
        };
        CppOutcome {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.status.code().unwrap_or(-1),
        }
    }

    /// How many programs this instance has actually run. See the module
    /// doc's "Why the invocation count is a field".
    pub fn invocations(&self) -> usize {
        self.invocations.load(Ordering::Relaxed)
    }
}

/// Spawns `command`, writes `bytes` to its standard input, closes it, and waits.
///
/// Split out because both interpreters need it -- the oracle here and
/// `rexx-run` in `tests/input_oracle.rs` -- and because getting it wrong has one
/// specific failure mode worth naming: writing the whole buffer before reading
/// any output deadlocks if the buffer is larger than a pipe and the program
/// writes enough to fill its own. `wait_with_output` reads both output pipes
/// concurrently, so the deadlock window is only the write below; every caller
/// here feeds a handful of lines, far inside one pipe buffer.
///
/// A write failure is ignored deliberately: a program that exits before reading
/// its input leaves this end broken (`EPIPE`), which is a legitimate outcome to
/// compare rather than a harness error.
pub fn write_and_wait(command: &mut Command, bytes: &[u8], path: &Path) -> std::process::Output {
    use std::io::Write;
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn for {}: {e}", path.display()));
    {
        let mut sink = child.stdin.take().expect("stdin was requested as a pipe");
        let _ = sink.write_all(bytes);
    }
    child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("failed to wait for {}: {e}", path.display()))
}

/// Truncates an in-process exit code to the single byte a real process's
/// status would carry.
///
/// [`rexx_exec::Outcome::exit_code`] can be wider than a byte (`EXIT`'s own
/// expression result, before `rexx-run`'s `as u8` wraps it for the OS), while
/// the oracle subprocess's exit status is already a byte by construction --
/// `std::process::ExitStatus::code` only ever returns what `WEXITSTATUS`
/// gives. Comparing the two without this would make `exit 256` (in-process
/// `256`, real process `0`) look like a divergence that `rexx-run`'s own
/// wrapping already resolves.
pub fn wrapped_exit_code(code: i32) -> i32 {
    i32::from(code as u8)
}

/// Which of the three observable channels disagree, in a fixed order.
/// Empty means the two interpreters agree.
///
/// DEVIATION 0: `stderr` is compared after collapsing each side's own
/// trace-line indent run, not byte-exact -- see [`super`]'s module doc for
/// the scope and `docs/superpowers/plans/phase-4-exclusions.txt` for why.
/// Exit status, stdout, and every other byte of stderr stay byte-exact.
pub fn descriptor_diffs(rust: &Outcome, cpp: &CppOutcome) -> Vec<&'static str> {
    let mut diffs = Vec::new();
    if rust.stdout != cpp.stdout {
        diffs.push("stdout");
    }
    if super::normalize_stderr(&rust.stderr) != super::normalize_stderr(&cpp.stderr) {
        diffs.push("stderr");
    }
    if wrapped_exit_code(rust.exit_code) != cpp.exit_code {
        diffs.push("exit code");
    }
    diffs
}
