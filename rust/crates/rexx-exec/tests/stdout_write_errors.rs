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

//! A stream write to standard output answers the error its descriptor
//! refused: the residual count, `ERROR:n` and `NOTREADY`.
//!
//! **No corpus program can see this.** The differential harness reads both
//! output streams through pipes it holds open, so only spawning `rexx` with a
//! standard output that refuses writes reaches the error. The buffered
//! standard output (Deviation 40) learns the error when it is handed over, so
//! the write before the first hand-over still answers `0`; the command here
//! is a hand-over. A program's own `STDOUT` stream holds its writes, and its
//! `FLUSH` is a hand-over that answers the error, as the oracle's does.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const PROGRAM: &str = "r1 = lineout('STDOUT', 'a'); s1 = stream('STDOUT', 'd')\n\
'true'\n\
r2 = lineout('STDOUT', 'b'); s2 = stream('STDOUT', 'd')\n\
r3 = .stdout~charout('c'); s3 = .stdout~description\n\
o = .stream~new('STDOUT'); r4 = o~lineout('d'); s4 = o~description\n\
call lineout 'STDERR', r1 s1 '|' r2 s2 '|' r3 s3 '|' r4 s4\n";

const FLUSH_PROGRAM: &str = "o = .stream~new('STDOUT')\n\
f0 = o~flush; d0 = o~description\n\
r1 = o~lineout('f'); f1 = o~flush; d1 = o~description\n\
f2 = o~flush; d2 = o~description\n\
r3 = o~lineout('g'); d3 = o~description\n\
f4 = .stdout~flush; d4 = .stdout~description\n\
e = .stream~new('STDERR'); r5 = e~lineout('h'); f5 = e~flush\n\
call lineout 'STDERR', f0 d0 '|' r1 f1 d1 '|' f2 d2 '|' r3 d3 '|' f4 d4 '|' r5 f5\n";

/// A private directory holding `text`, for the case `name`.
fn program(name: &str, text: &str) -> (PathBuf, PathBuf) {
    let dir =
        std::env::temp_dir().join(format!("rexx-stdout-errors-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a private probe directory");
    let path = dir.join("errors.rex");
    std::fs::write(&path, text).expect("the probe program is writable");
    (dir, path)
}

/// `rexx` on `text` with `stdout`, for the case `name`, answering its standard
/// error and status.
fn run_with_stdout(name: &str, text: &str, stdout: Stdio) -> (String, Option<i32>) {
    let (dir, path) = program(name, text);
    let mut child = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(&path)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn rexx: {e}"));
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr is piped")
        .read_to_string(&mut stderr)
        .expect("stderr is readable");
    let status = child.wait().expect("rexx ends");
    let _ = std::fs::remove_dir_all(&dir);
    (stderr, status.code())
}

/// A pipe nobody reads: `EPIPE`, which the oracle answers as `ERROR:32` (it
/// ignores `SIGPIPE`, as `rexx` does).
#[test]
fn a_closed_pipe_is_error_32() {
    let (reader, writer) = std::io::pipe().expect("a pipe");
    drop(reader);
    let (stderr, status) = run_with_stdout("pipe", PROGRAM, Stdio::from(writer));
    assert_eq!(status, Some(0), "{stderr}");
    assert_eq!(
        stderr,
        "0 READY: | 1 ERROR:32 Broken pipe | 1 ERROR:32 Broken pipe | 0 READY:\n"
    );
}

/// A full device: `ENOSPC`, `ERROR:28`.
fn full_device() -> Stdio {
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full opens");
    Stdio::from(full)
}

/// `/dev/full`: `ENOSPC`, `ERROR:28`.
#[test]
fn a_full_device_is_error_28() {
    let (stderr, status) = run_with_stdout("full", PROGRAM, full_device());
    assert_eq!(status, Some(0), "{stderr}");
    assert_eq!(
        stderr,
        "0 READY: | 1 ERROR:28 No space left on device | 1 ERROR:28 No space left on device | 0 READY:\n"
    );
}

/// A program's own `STDOUT` stream answers its `FLUSH` with the closed pipe's
/// error, and so does the flush after it, until it writes again; an unopened
/// one and `.STDOUT`, which holds nothing, answer `READY:`.
#[test]
fn a_flush_into_a_closed_pipe_is_error_32() {
    let (reader, writer) = std::io::pipe().expect("a pipe");
    drop(reader);
    let (stderr, status) = run_with_stdout("flush-pipe", FLUSH_PROGRAM, Stdio::from(writer));
    assert_eq!(status, Some(0), "{stderr}");
    assert_eq!(
        stderr,
        "h\nREADY: UNKNOWN: | 0 ERROR:32 ERROR:32 Broken pipe | ERROR:32 ERROR:32 Broken pipe | \
         0 READY: | READY: READY: | 0 READY:\n"
    );
}

/// [`a_flush_into_a_closed_pipe_is_error_32`] on `/dev/full`.
#[test]
fn a_flush_into_a_full_device_is_error_28() {
    let (stderr, status) = run_with_stdout("flush-full", FLUSH_PROGRAM, full_device());
    assert_eq!(status, Some(0), "{stderr}");
    assert_eq!(
        stderr,
        "h\nREADY: UNKNOWN: | 0 ERROR:28 ERROR:28 No space left on device | \
         ERROR:28 ERROR:28 No space left on device | 0 READY: | READY: READY: | 0 READY:\n"
    );
}

/// A standard output closed when `rexx` starts is `/dev/null` here, so a write
/// answers `0` and the stream `READY:`, where the oracle answers
/// `1 ERROR:9 Bad file descriptor`.
#[test]
fn a_standard_output_closed_at_start_is_dev_null() {
    let (dir, path) = program(
        "closed-at-start",
        "r = lineout('STDOUT', 'a'); s = stream('STDOUT', 'd')\n\
         call lineout 'STDERR', r s\n",
    );
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg("exec \"$REXX\" \"$PROG\" >&-")
        .env("REXX", env!("CARGO_BIN_EXE_rexx"))
        .env("PROG", &path)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("sh runs");
    let _ = std::fs::remove_dir_all(&dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert_eq!(stderr, "0 READY:\n");
}
