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
//! `unsafe` `pre_exec` closure calling `setrlimit`. The workspace lint is
//! `unsafe_code = "deny"`, so that is an exception a site may be granted
//! rather than one it cannot have -- and this one would buy nothing a shell
//! builtin does not already do for free, which is why it was not asked for.
//! `rust/CLAUDE.md` states the bar and `rexx-core/tests/unsafe_sites.rs`
//! records the granted set. Verified directly, outside this test: `sh -c
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

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use rexx_exec::Outcome;

/// Address-space ceiling imposed on every oracle invocation, in KiB. 1 GiB:
/// the figure this project has used by hand throughout Phase 4, restated
/// here as a named constant rather than a magic number in the format string.
pub const ORACLE_MEMORY_LIMIT_KIB: u64 = 1_048_576;

/// How long [`wait_with_deadline`] waits for the direct child before killing
/// it, and separately, how long it then waits for the two output pipes to
/// finish reading -- so one oracle invocation is bounded at up to **twice**
/// this, not once: `wait_with_deadline`'s own doc has the reason the read
/// needs a budget of its own rather than sharing the wait's. 10 seconds is
/// the figure `timeout -s KILL 10` fixes by hand around every oracle
/// invocation this project runs outside the suite (`rust/CLAUDE.md`'s "Wrap
/// every oracle run" and the probe rules below it), which bounds only the
/// process, not a descriptor it hands to one of its own -- the harness's
/// doubled bound is what closes that gap, not a mismatch with the by-hand
/// figure.
pub const ORACLE_DEADLINE: Duration = Duration::from_secs(10);

/// How often [`wait_with_deadline`] polls [`Child::try_wait`] while a run is
/// still outstanding.
///
/// **Why polling rather than `wait-timeout 0.2.0`.** That crate resolves
/// offline in this machine's cargo cache (`cargo add -p rexx-exec --dry-run
/// --offline wait-timeout@0.2.0` succeeds, exit 0, against
/// `~/.cargo/registry/src/.../wait-timeout-0.2.0`), so it was a real option
/// and not ruled out by the no-network constraint. It buys nothing here: its
/// `wait_timeout` only replaces the polling loop below, and this harness
/// still needs its own threads reading `stdout`/`stderr` concurrently so a
/// chatty program cannot fill one pipe and deadlock the wait -- the same
/// problem [`write_and_wait`]'s doc names for the write side. A dependency
/// that would remove one loop and leave the harder half of the mechanism
/// exactly as it is is not what "buys something" means, so this stays a
/// plain poll and no dev-dependency is added.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

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

/// How an oracle-side run ended.
///
/// A pure function of what [`ExitStatus::code`](std::process::ExitStatus::code)
/// returned and whether [`wait_with_deadline`] is the one that ended the
/// run, kept as a named type rather than an `i32` exit code:
/// `status.code().unwrap_or(-1)` gives a signal death and a normal `exit -1`
/// the identical representation, and a verdict function comparing exit
/// codes reads the former as a divergence -- a wrong classification, not
/// merely an imprecise one -- rather than the failure it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Termination {
    /// The process ran to completion and returned this status.
    Exited(i32),
    /// The process died from a signal nobody here sent -- a crash, not a
    /// timeout.
    Signaled,
    /// [`wait_with_deadline`] killed the process because it was still
    /// running at [`ORACLE_DEADLINE`].
    TimedOut,
}

/// Classifies a wait's result into a [`Termination`], from the two plain
/// values -- `code: Option<i32>` and `deadline_exceeded: bool` -- that
/// [`wait_with_deadline`] already has in hand, rather than from a live
/// `ExitStatus`. `ExitStatus` has no portable public constructor
/// (`ExitStatus::from_raw` is Unix-only and still needs a real wait status
/// from the kernel), so a signal-death test could not build one to call this
/// on; taking the two primitives it would otherwise be inspected for keeps
/// both failing arms plain unit tests instead.
pub fn classify_termination(code: Option<i32>, deadline_exceeded: bool) -> Termination {
    match (code, deadline_exceeded) {
        (Some(status), _) => Termination::Exited(status),
        (None, true) => Termination::TimedOut,
        (None, false) => Termination::Signaled,
    }
}

/// Whether an oracle run ended some way other than a normal exit -- the
/// check a caller makes before treating a run as a structural failure rather
/// than a pair of bytes to diff: whatever `stdout`/`stderr` a killed or
/// crashed process left behind is wherever it happened to be interrupted,
/// not a completed answer to compare against. Tested on
/// [`CppOutcome::termination`] directly, never on a synthesised exit
/// code, since [`classify_termination`]'s two failing arms both carry `code:
/// None` and nothing about the exit code distinguishes them.
pub fn did_not_finish(outcome: &CppOutcome) -> bool {
    !matches!(outcome.termination, Termination::Exited(_))
}

/// What one oracle run produced. Deliberately not [`rexx_exec::Outcome`]:
/// that type carries a `stack: StackSpan` field this process never measures,
/// and reusing it would invite comparing a field that was never filled in.
pub struct CppOutcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub termination: Termination,
}

impl CppOutcome {
    /// The process's own exit status.
    ///
    /// Panics if the run did not end that way -- every existing differential
    /// caller in this tree assumes the oracle ran to completion and compares
    /// bytes on that assumption; a [`did_not_finish`] check belongs before
    /// this call wherever a timeout or a crash is a live possibility rather
    /// than a bug. A panic here is what such an assumption gets when it is
    /// wrong: a structural failure naming what actually happened, rather
    /// than an ordinary exit code standing in for it.
    pub fn expect_exit_code(&self) -> i32 {
        match self.termination {
            Termination::Exited(code) => code,
            other => panic!("oracle run did not exit normally, so it has no exit code: {other:?}"),
        }
    }
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
        let mut command = self.wrapped(path, args);
        command
            .stdin(match stdin {
                None => Stdio::null(),
                Some(_) => Stdio::piped(),
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn the oracle for {}: {e}", path.display()));
        if let Some(bytes) = stdin {
            // A write failure is ignored deliberately -- see
            // `write_and_wait`'s doc for why an `EPIPE` here is a legitimate
            // outcome to compare rather than a harness error.
            use std::io::Write;
            let mut sink = child.stdin.take().expect("stdin was requested as a pipe");
            let _ = sink.write_all(bytes);
        }
        let (stdout, stderr, termination) = wait_with_deadline(child, path);
        CppOutcome {
            stdout,
            stderr,
            termination,
        }
    }

    /// [`Oracle::run_with`], given a **descriptor** for standard input instead
    /// of bytes to feed down a pipe.
    ///
    /// For the inputs a byte buffer cannot express: a closed descriptor, or one
    /// whose `read` fails outright. `Stdio` is taken by value because it is
    /// consumed by the spawn, so a caller comparing both interpreters builds
    /// one for each side.
    ///
    /// Separate from `run_with` rather than replacing its `Option<&[u8]>`,
    /// because a caller passing bytes should not have to construct a pipe
    /// itself and because `Stdio` cannot express "feed these bytes".
    pub fn run_with_stdin(&self, path: &Path, args: &[&str], stdin: Stdio) -> CppOutcome {
        self.invocations.fetch_add(1, Ordering::Relaxed);
        let mut child = self
            .wrapped(path, args)
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn the oracle for {}: {e}", path.display()));
        // A no-op for the `File`-backed `Stdio` `input_oracle.rs`'s
        // `an_unreadable_console_is_end_of_input` passes, and for
        // `Stdio::null()`, since neither hands back a `child.stdin` to
        // take. It matters for `Stdio::piped()`, which the signature also
        // accepts: this method never writes to a piped stdin, so without
        // this a caller passing one would get a child waiting on input that
        // never arrives -- killed at `ORACLE_DEADLINE` and reported
        // `TimedOut`, a ten-second wait for what closing the pipe here
        // would have answered at once. `run_with`'s bytes path avoids the
        // same trap by dropping its own `sink` after writing.
        drop(child.stdin.take());
        let (stdout, stderr, termination) = wait_with_deadline(child, path);
        CppOutcome {
            stdout,
            stderr,
            termination,
        }
    }

    /// [`Oracle::run`] with `TZ` set, for a caller asking whether a program's
    /// answer depends on the clock and zone it is read under.
    ///
    /// The variable is set on the child alone; this process's own environment
    /// is untouched, which matters because the in-process executor a
    /// differential harness compares against reads the same one.
    pub fn run_in_zone(&self, path: &Path, zone: &str) -> CppOutcome {
        self.invocations.fetch_add(1, Ordering::Relaxed);
        let child = self
            .wrapped(path, &[])
            .env("TZ", zone)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn the oracle for {}: {e}", path.display()));
        let (stdout, stderr, termination) = wait_with_deadline(child, path);
        CppOutcome {
            stdout,
            stderr,
            termination,
        }
    }

    /// [`Oracle::run`] from a chosen working directory and with chosen
    /// environment entries, for a caller asking a question about how the
    /// interpreter finds a *second* file.
    ///
    /// The default working directory is the program's own, which is right for
    /// a corpus program and is exactly what a search-order question has to be
    /// able to vary: `::REQUIRES` looks in the program's directory and in the
    /// current one, and those are the same place unless a caller separates
    /// them. `environment` is applied to the child alone.
    pub fn run_in(&self, path: &Path, cwd: &Path, environment: &[(&str, &str)]) -> CppOutcome {
        self.invocations.fetch_add(1, Ordering::Relaxed);
        let mut command = self.wrapped(path, &[]);
        command.current_dir(cwd);
        for (name, value) in environment {
            command.env(name, value);
        }
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn the oracle for {}: {e}", path.display()));
        let (stdout, stderr, termination) = wait_with_deadline(child, path);
        CppOutcome {
            stdout,
            stderr,
            termination,
        }
    }

    /// The `sh -c 'ulimit … && exec "$0" "$@"'` invocation, the library path
    /// and the working directory, with standard input left for the caller.
    ///
    /// One builder rather than one per entry point: the memory limit is not
    /// optional on any of them, and a second hand-rolled copy of this wrapper
    /// is how one gets omitted. It deliberately does **not** touch the
    /// invocation counter, so that each public method increments exactly once.
    fn wrapped(&self, path: &Path, args: &[&str]) -> Command {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(format!(
                "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" \"$@\""
            ))
            .arg(&self.binary)
            .arg(path)
            .args(args)
            .current_dir(path.parent().unwrap_or(Path::new(".")))
            .env("LD_LIBRARY_PATH", &self.lib_dir);
        command
    }

    /// How many programs this instance has actually run. See the module
    /// doc's "Why the invocation count is a field".
    pub fn invocations(&self) -> usize {
        self.invocations.load(Ordering::Relaxed)
    }
}

/// Spawns `command`, writes `bytes` to its standard input, closes it, and waits.
///
/// **Used by `rexx-run` in `tests/input_oracle.rs`, not by the oracle side
/// here.** `Oracle::run`, `run_with` and `run_with_stdin` carry a deadline
/// and need [`wait_with_deadline`]'s poll loop for it, which this function's
/// single blocking `wait_with_output` cannot express; `rexx-run` run as a
/// subprocess carries no deadline of its own, so this stays the simpler,
/// blocking form for its caller in `tests/input_oracle.rs`. Getting it
/// wrong has one specific failure mode worth naming regardless: writing the
/// whole buffer before reading any output deadlocks if the buffer is
/// larger than a pipe and the
/// program writes enough to fill its own. `wait_with_output` reads both
/// output pipes concurrently, so the deadlock window is only the write
/// below; the caller here feeds a handful of lines, far inside one pipe
/// buffer.
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

/// Waits for `child` under [`ORACLE_DEADLINE`], reading both output pipes
/// concurrently, and returns what it produced along with how it ended.
///
/// **Why this exists.** `Command::output` (and `Child::wait_with_output`,
/// which it calls) blocks on `waitpid` with no deadline of its own -- so a
/// program that never exits, `do forever; end` among them, hangs whichever
/// test calls it, and hangs `cargo test --workspace` with it. This polls
/// [`Child::try_wait`] instead, at [`POLL_INTERVAL`], and calls `kill()` the
/// moment [`ORACLE_DEADLINE`] has passed.
///
/// **Why the reader threads report over a channel rather than being
/// joined.** `read_to_end` returns only once *every* write end of a pipe is
/// closed, not once the direct child dies -- and `kill()` above reaches only
/// the direct child. A program that hands its own descriptor to a process
/// of its own (`address system 'sleep 3600 &'`, still running when its
/// parent exits) leaves that process holding the write end, so an
/// unconditional `join()` here would block for that process's own lifetime
/// regardless of how the child above was classified: this deadline would
/// bound the wait and not the read, one gap wide enough for a single
/// committed probe to hang `cargo test --workspace` for an hour. So each
/// reader thread sends its buffer down a channel instead of being joined,
/// and this function gives that channel a bounded, independent budget of
/// its own -- [`ORACLE_DEADLINE`] again, but measured fresh from here rather
/// than as whatever the process-wait loop above left over, since a direct
/// child that exits at once (this shape's whole point) would otherwise
/// leave the read almost no time to run. **The residual, honestly**: giving
/// up on a channel does not reap what is blocking it. A grandchild that
/// still holds the descriptor keeps running, and the reader thread stays
/// parked in its `read` call for as long as that process does (or forever)
/// -- a leaked background process and a leaked background thread in this
/// test binary, not a hung suite. `Child` has no handle on a process it
/// never spawned, so there is nothing further here to kill.
///
/// **Why the reader threads exist at all.** Polling `try_wait` instead of
/// blocking on `wait` means nothing here is blocked reading `stdout`/`stderr`
/// while the poll loop runs, so those two pipes are read on their own
/// threads from the moment the child is spawned -- the same concurrent-read
/// shape `wait_with_output` already uses internally, needed for the same
/// reason [`write_and_wait`]'s doc names for the write side: a chatty
/// program can fill a pipe's kernel buffer and block until something drains
/// it, and nothing here may be that something only once a wait already
/// returned.
///
/// `child`'s `stdin` is expected to already be closed or fully written by the
/// caller -- this function only waits and reads the two output pipes.
fn wait_with_deadline(mut child: Child, path: &Path) -> (Vec<u8>, Vec<u8>, Termination) {
    let mut stdout_pipe = child.stdout.take().expect("stdout was requested as a pipe");
    let mut stderr_pipe = child.stderr.take().expect("stderr was requested as a pipe");
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        let _ = stdout_tx.send(buf);
    });
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        let _ = stderr_tx.send(buf);
    });

    let start = Instant::now();
    let (mut code, mut deadline_exceeded) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) => {
                if start.elapsed() >= ORACLE_DEADLINE {
                    // The process is still running past its deadline: kill
                    // it, then bind the status `wait()` actually returns
                    // rather than assuming `None`. A process that exited in
                    // the gap between the last `try_wait` and this check is
                    // killed as a no-op -- the signal lands on an
                    // already-dead process -- and `wait()` still reports its
                    // real status, so binding it here turns what would
                    // otherwise be a flaky `TimedOut` (and the panic
                    // `expect_exit_code` gives one) into the `Exited` this
                    // run actually earned.
                    let _ = child.kill();
                    let status = child.wait().ok();
                    break (status.and_then(|s| s.code()), true);
                }
                thread::sleep(POLL_INTERVAL);
            }
            Err(e) => panic!("failed to wait for {}: {e}", path.display()),
        }
    };

    // Bounded independently of the wait above -- see "Why the reader
    // threads report over a channel" -- so a fast-exiting direct child with
    // a slow grandchild still has this whole deadline to finish reading in,
    // and a read that has already outlasted one deadline is not handed a
    // second one on top by chaining off the first one's remaining time. The
    // two channels share one clock rather than each getting a fresh
    // `ORACLE_DEADLINE`, so the pair together are bounded by it once, not
    // twice.
    let read_deadline = Instant::now() + ORACLE_DEADLINE;
    let stdout_result =
        stdout_rx.recv_timeout(read_deadline.saturating_duration_since(Instant::now()));
    let stderr_result =
        stderr_rx.recv_timeout(read_deadline.saturating_duration_since(Instant::now()));

    // A disconnected channel means the sender end was dropped without
    // sending -- the reader thread panicked before it could report its
    // buffer -- which `recv_timeout` distinguishes from an ordinary
    // `Timeout` and this function does too, rather than reading both the
    // same way. Folding it into a `TimedOut` classification instead would
    // trade a named harness bug for a silent misclassification.
    if matches!(stdout_result, Err(mpsc::RecvTimeoutError::Disconnected)) {
        panic!("stdout reader thread panicked for {}", path.display());
    }
    if matches!(stderr_result, Err(mpsc::RecvTimeoutError::Disconnected)) {
        panic!("stderr reader thread panicked for {}", path.display());
    }

    let (stdout, stderr) = match (stdout_result, stderr_result) {
        (Ok(stdout), Ok(stderr)) => (stdout, stderr),
        _ => {
            // At least one side is still blocked, almost always behind a
            // descriptor its direct child handed to a process of its own.
            // The transcript is not trustworthy either way -- a stdout that
            // did arrive on time proves nothing about a stderr that did not,
            // and vice versa -- so this run counts as a non-finish and both
            // channels read empty. Free to do: a non-finish's bytes are a
            // structural failure, never a comparison, so nothing downstream
            // reads them.
            //
            // Only synthesised when the process-wait loop above found an
            // `Exited` -- if it already found a non-finish of its own
            // (`Signaled` or `TimedOut`), that classification is left alone
            // rather than overwritten: a child that genuinely died from a
            // signal must not be relabelled `TimedOut` merely because a
            // descendant of its also kept a pipe open, since `Signaled`'s
            // own doc distinguishes a crash from a timeout and a reader
            // sent to the wrong one learns the wrong thing.
            if matches!(
                classify_termination(code, deadline_exceeded),
                Termination::Exited(_)
            ) {
                code = None;
                deadline_exceeded = true;
            }
            (Vec::new(), Vec::new())
        }
    };

    (
        stdout,
        stderr,
        classify_termination(code, deadline_exceeded),
    )
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

/// How `stderr` is compared: [`descriptor_diffs`]'s DEVIATION 0 normalisation
/// (the default every corpus program gets unless it opts out), or raw bytes.
///
/// **Opt-in per program, not a global switch.** Phase 5a's own task brief asks
/// for a way to "claim a byte-for-byte stderr comparison" -- a stricter claim
/// than DEVIATION 0 makes -- without disturbing the normalised path everything
/// else still relies on. `docs/superpowers/plans/phase-4-exclusions.txt`'s
/// Deviation 0 stays the default and is pinned by
/// `tests/support/mod.rs`'s own committed tests; this enum only adds a second,
/// stricter mode a caller can select for one program at a time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StderrComparison {
    /// DEVIATION 0: both sides' `stderr` run through [`super::normalize_stderr`]
    /// first. The default.
    Normalized,
    /// Byte-for-byte, no normalisation at all.
    Raw,
    /// Byte-for-byte after sorting each side's lines: DEVIATION 7's licence,
    /// for a program whose trace lines are written by two threads whose
    /// interleaving is not a specified observable.
    Multiset,
}

/// How `stdout` is compared: byte-for-byte, or as a sorted multiset of lines.
///
/// **Opt-in per program, exactly as [`StderrComparison::Multiset`] is.** Raw
/// is the default and stays the default; the sorted mode exists for a program
/// whose stdout is a hash-ordered collection's contents, an order neither
/// interpreter reproduces for the other. `phase-4-exclusions.txt`'s
/// Deviation 8 is the licence, and the opt-in list there IS the licence: a
/// sorted comparison hides a genuine ordering defect in any program that
/// takes it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StdoutComparison {
    /// Byte-for-byte. The default.
    Raw,
    /// Byte-for-byte after sorting each side's lines: Deviation 8's licence.
    Multiset,
}

/// One side's `stdout` in the form [`StdoutComparison::Multiset`] compares.
///
/// The same shape as [`stderr_multiset`], and split the same way, so a
/// truncated `stdout` still differs.
pub fn stdout_multiset(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes).into_owned();
    let mut lines: Vec<&str> = text.split('\n').collect();
    lines.sort_unstable();
    lines.join("\n")
}

/// One side's `stderr` in the form [`StderrComparison::Multiset`] compares,
/// exposed so the licence has one implementation and one control.
///
/// `split` on `\n` rather than `str::lines`, so the trailing empty element a
/// final newline produces survives and a truncated `stderr` still differs.
pub fn stderr_multiset(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes).into_owned();
    let mut lines: Vec<&str> = text.split('\n').collect();
    lines.sort_unstable();
    lines.join("\n")
}

/// Which of the three observable channels disagree, in a fixed order.
/// Empty means the two interpreters agree.
///
/// DEVIATION 0: `stderr` is compared after collapsing each side's own
/// trace-line indent run, not byte-exact -- see [`super`]'s module doc for
/// the scope and `docs/superpowers/plans/phase-4-exclusions.txt` for why.
/// Exit status, stdout, and every other byte of stderr stay byte-exact.
///
/// Thin wrapper over [`descriptor_diffs_with`] at [`StderrComparison::Normalized`],
/// kept as its own function so every existing call site stays untouched.
pub fn descriptor_diffs(rust: &Outcome, cpp: &CppOutcome) -> Vec<&'static str> {
    descriptor_diffs_with(rust, cpp, StderrComparison::Normalized)
}

/// Which of the three observable channels disagree, one field each.
///
/// **The answer callers should read, and [`descriptor_diffs_with`]'s labels
/// are a rendering of it rather than a second copy.** A caller that needs to
/// know *which* channel moved -- a verdict function, say -- reads these
/// fields; a caller that needs to show a human what moved calls
/// [`Self::labels`] or the `Vec`-returning wrapper. Recovering a channel by
/// matching a label's text is the shape this type exists to remove:
/// `contains` is a positive test, so a renamed or added label is not seen
/// rather than reported, and a consumer built that way reads one input as
/// permanently "did not differ" with nothing anywhere to notice.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DescriptorDiff {
    pub stdout: bool,
    pub stderr: bool,
    pub exit_code: bool,
}

impl DescriptorDiff {
    /// Whether the two sides disagreed at all.
    pub fn any(self) -> bool {
        self.stdout || self.stderr || self.exit_code
    }

    /// The differing channels' names, in a fixed order, for a report.
    ///
    /// Derived from the fields on every call, so the rendering cannot claim a
    /// channel the comparison did not find or omit one it did.
    /// `labels_name_exactly_the_channels_that_differ` walks the whole cube of
    /// three booleans and holds the two together.
    pub fn labels(self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if self.stdout {
            labels.push("stdout");
        }
        if self.stderr {
            labels.push("stderr");
        }
        if self.exit_code {
            labels.push("exit code");
        }
        labels
    }
}

/// The three-channel comparison itself, with the caller choosing how `stderr`
/// is compared. See [`StderrComparison`] for the two modes and why raw is
/// opt-in rather than the default.
pub fn descriptor_diff_with(
    rust: &Outcome,
    cpp: &CppOutcome,
    stderr_mode: StderrComparison,
) -> DescriptorDiff {
    descriptor_diff_modes(rust, cpp, StdoutComparison::Raw, stderr_mode)
}

/// [`descriptor_diff_with`] with `stdout`'s comparison chosen as well.
///
/// Kept as the one place both channels' modes are applied, so that
/// Deviation 8's licence has a single implementation the way Deviation 7's
/// does.
pub fn descriptor_diff_modes(
    rust: &Outcome,
    cpp: &CppOutcome,
    stdout_mode: StdoutComparison,
    stderr_mode: StderrComparison,
) -> DescriptorDiff {
    DescriptorDiff {
        stdout: match stdout_mode {
            StdoutComparison::Raw => rust.stdout != cpp.stdout,
            StdoutComparison::Multiset => {
                stdout_multiset(&rust.stdout) != stdout_multiset(&cpp.stdout)
            }
        },
        stderr: match stderr_mode {
            StderrComparison::Normalized => {
                super::normalize_stderr(&rust.stderr) != super::normalize_stderr(&cpp.stderr)
            }
            StderrComparison::Raw => rust.stderr != cpp.stderr,
            StderrComparison::Multiset => {
                stderr_multiset(&rust.stderr) != stderr_multiset(&cpp.stderr)
            }
        },
        exit_code: wrapped_exit_code(rust.exit_code) != cpp.expect_exit_code(),
    }
}

/// [`descriptor_diff_with`] rendered as its labels, for the callers that print
/// them or test the list for emptiness.
pub fn descriptor_diffs_with(
    rust: &Outcome,
    cpp: &CppOutcome,
    stderr_mode: StderrComparison,
) -> Vec<&'static str> {
    descriptor_diff_with(rust, cpp, stderr_mode).labels()
}

/// [`descriptor_diff_modes`] rendered as its labels.
pub fn descriptor_diffs_modes(
    rust: &Outcome,
    cpp: &CppOutcome,
    stdout_mode: StdoutComparison,
    stderr_mode: StderrComparison,
) -> Vec<&'static str> {
    descriptor_diff_modes(rust, cpp, stdout_mode, stderr_mode).labels()
}

#[cfg(test)]
mod tests {
    use super::{
        CppOutcome, DescriptorDiff, StderrComparison, StdoutComparison, Termination,
        classify_termination, descriptor_diffs_modes, descriptor_diffs_with, did_not_finish,
        stdout_multiset,
    };
    use rexx_exec::{Outcome, StackSpan};

    /// [`DescriptorDiff::labels`] names exactly the channels whose field is
    /// set, over every combination of the three.
    ///
    /// **This is what keeps the rendering and the answer one quantity.** The
    /// labels are what a report prints and what `corpus.rs` tests for
    /// emptiness; the fields are what a verdict function reads. A label
    /// dropped from the rendering would make a real divergence look like
    /// agreement to the emptiness test, and a label added would name a
    /// channel nothing compared -- neither is visible from either side alone.
    #[test]
    fn labels_name_exactly_the_channels_that_differ() {
        for stdout in [false, true] {
            for stderr in [false, true] {
                for exit_code in [false, true] {
                    let diff = DescriptorDiff {
                        stdout,
                        stderr,
                        exit_code,
                    };
                    let labels = diff.labels();
                    assert_eq!(labels.contains(&"stdout"), stdout, "{diff:?}");
                    assert_eq!(labels.contains(&"stderr"), stderr, "{diff:?}");
                    assert_eq!(labels.contains(&"exit code"), exit_code, "{diff:?}");
                    assert_eq!(
                        labels.len(),
                        usize::from(stdout) + usize::from(stderr) + usize::from(exit_code),
                        "{diff:?} rendered a label for a channel that did not differ, or \
                         rendered one channel twice"
                    );
                    assert_eq!(diff.any(), !labels.is_empty(), "{diff:?}");
                }
            }
        }
    }

    fn outcome(stderr: &[u8]) -> Outcome {
        Outcome {
            exit_code: 0,
            stdout: Vec::new(),
            stderr: stderr.to_vec(),
            stack: StackSpan::default(),
            collections: 0,
            chunks_refused: 0,
        }
    }

    fn cpp_outcome(stderr: &[u8]) -> CppOutcome {
        CppOutcome {
            stdout: Vec::new(),
            stderr: stderr.to_vec(),
            termination: Termination::Exited(0),
        }
    }

    /// Both directions of [`StderrComparison`], on a transcript pair whose
    /// only difference is two columns of trace indent -- exactly the shape
    /// DEVIATION 0 exists to absorb (`tests/support/mod.rs`'s own
    /// `two_clause_lines_differing_only_in_indent_width_normalise_equal`
    /// pins the same pair through `normalize_stderr` directly). Task 1's own
    /// brief requires both directions or the new mode is unwitnessed: a mode
    /// that only ever fails, or only ever passes, would not distinguish
    /// "wired in" from "inert".
    #[test]
    fn raw_fails_where_normalized_passes_on_a_two_column_indent_difference() {
        let rust = outcome(b"     4 *-* say 1/0\n");
        let oracle = cpp_outcome(b"     4 *-*   say 1/0\n");

        let raw = descriptor_diffs_with(&rust, &oracle, StderrComparison::Raw);
        assert_eq!(
            raw,
            vec!["stderr"],
            "raw comparison must FAIL (report a stderr diff) on a two-column \
             indent difference -- that is the whole point of adding a mode \
             DEVIATION 0 does not apply to"
        );

        let normalized = descriptor_diffs_with(&rust, &oracle, StderrComparison::Normalized);
        assert!(
            normalized.is_empty(),
            "normalized comparison (the default) must PASS on the identical \
             pair, or this is not demonstrating two different modes at all: \
             got {normalized:?}"
        );
    }

    fn stdout_outcome(stdout: &[u8]) -> Outcome {
        Outcome {
            exit_code: 0,
            stdout: stdout.to_vec(),
            stderr: Vec::new(),
            stack: StackSpan::default(),
            collections: 0,
            chunks_refused: 0,
        }
    }

    fn cpp_stdout_outcome(stdout: &[u8]) -> CppOutcome {
        CppOutcome {
            stdout: stdout.to_vec(),
            stderr: Vec::new(),
            termination: Termination::Exited(0),
        }
    }

    /// The transcript DEVIATION 8's controls mutate, and its reordering.
    const LINES: &[u8] = b"alpha\nbravo\ncharlie\n";
    const REORDERED: &[u8] = b"charlie\nalpha\nbravo\n";

    /// DEVIATION 8's control: [`stdout_multiset`] discards ORDERING and
    /// nothing else.
    ///
    /// **The mutation this exists to catch is the licence's own comparison
    /// function going inert.** Replacing `stdout_multiset`'s body with
    /// `String::new()` makes every pair below compare equal, so the five
    /// "still differs" assertions fail; without them the whole corpus binary
    /// stays green under that mutation, gate included, which is what a
    /// licence with no control means. DEVIATION 7's
    /// `the_multiset_comparison_discards_ordering_and_nothing_else` is the
    /// same control over `stderr_multiset` and this is its counterpart.
    #[test]
    fn the_stdout_multiset_comparison_discards_ordering_and_nothing_else() {
        assert_eq!(
            stdout_multiset(LINES),
            stdout_multiset(REORDERED),
            "sorting must accept a pure reordering, or the licence covers nothing"
        );
        for (what, mutated) in [
            ("a changed line", b"alpha\nBRAVO\ncharlie\n".as_slice()),
            ("a missing line", b"alpha\ncharlie\n".as_slice()),
            (
                "an added line",
                b"alpha\nbravo\ncharlie\ndelta\n".as_slice(),
            ),
            (
                "a duplicated line",
                b"alpha\nbravo\nbravo\ncharlie\n".as_slice(),
            ),
            ("a lost final newline", b"alpha\nbravo\ncharlie".as_slice()),
        ] {
            assert_ne!(
                stdout_multiset(LINES),
                stdout_multiset(mutated),
                "sorting must still catch {what}, which Deviation 8 does not license"
            );
        }
    }

    /// DEVIATION 8's second control: only the multiset mode ignores stdout
    /// ordering, so the licence cannot leak to a program off the list.
    ///
    /// **The mutation this exists to catch is `StdoutComparison::Raw`'s arm
    /// being made to compare multisets.** That widens the licence to every
    /// corpus program at once and nothing else notices -- measured by the
    /// review, 462 of 462 still matching. Here it makes the first assertion
    /// fail.
    #[test]
    fn only_the_multiset_stdout_mode_ignores_ordering() {
        let rust = stdout_outcome(REORDERED);
        let oracle = cpp_stdout_outcome(LINES);

        let raw = descriptor_diffs_modes(
            &rust,
            &oracle,
            StdoutComparison::Raw,
            StderrComparison::Normalized,
        );
        assert_eq!(
            raw,
            vec!["stdout"],
            "the raw mode must report a stdout difference on a pure reordering, \
             or Deviation 8's licence is in force for every program"
        );

        let sorted = descriptor_diffs_modes(
            &rust,
            &oracle,
            StdoutComparison::Multiset,
            StderrComparison::Normalized,
        );
        assert!(
            sorted.is_empty(),
            "the multiset mode must accept the same reordering, or the two modes \
             are not being told apart at all: got {sorted:?}"
        );
    }

    /// A run that exited normally is `Exited`, whatever `deadline_exceeded`
    /// says -- a completed process cannot also have been killed for running
    /// too long. Pairs with the two failing-arm tests below rather than
    /// standing alone, per `rust/CLAUDE.md`'s "pair a refusal with its
    /// adjacent success": without it, the two below could be pinning
    /// `deadline_exceeded` to the classification's whole answer instead of
    /// to the tiebreak it only is when `code` is `None`.
    #[test]
    fn classify_termination_reports_a_normal_exit_regardless_of_the_deadline_flag() {
        assert_eq!(classify_termination(Some(0), false), Termination::Exited(0));
        assert_eq!(classify_termination(Some(1), true), Termination::Exited(1));
    }

    /// `(None, true)`: [`wait_with_deadline`] is the one that ended the run.
    #[test]
    fn classify_termination_reports_timed_out_when_the_harness_killed_it() {
        assert_eq!(classify_termination(None, true), Termination::TimedOut);
    }

    /// `(None, false)`: the process died from a signal nobody here sent -- a
    /// crash, distinguished from `TimedOut` only by which side of the `if`
    /// in [`wait_with_deadline`] produced the `None`.
    #[test]
    fn classify_termination_reports_signaled_when_nothing_here_killed_it() {
        assert_eq!(classify_termination(None, false), Termination::Signaled);
    }

    /// [`did_not_finish`] reads the status, not a number: both failing arms
    /// answer `true` even though neither carries an exit code to compare.
    #[test]
    fn did_not_finish_is_true_for_both_failing_arms_and_false_for_a_normal_exit() {
        let finished = cpp_outcome(b"");
        assert!(!did_not_finish(&finished));

        let timed_out = CppOutcome {
            termination: Termination::TimedOut,
            ..cpp_outcome(b"")
        };
        assert!(did_not_finish(&timed_out));

        let signaled = CppOutcome {
            termination: Termination::Signaled,
            ..cpp_outcome(b"")
        };
        assert!(did_not_finish(&signaled));
    }
}
