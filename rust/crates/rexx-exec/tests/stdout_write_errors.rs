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
//! is a hand-over.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const PROGRAM: &str = "r1 = lineout('STDOUT', 'a'); s1 = stream('STDOUT', 'd')\n\
'true'\n\
r2 = lineout('STDOUT', 'b'); s2 = stream('STDOUT', 'd')\n\
r3 = .stdout~charout('c'); s3 = .stdout~description\n\
o = .stream~new('STDOUT'); r4 = o~lineout('d'); s4 = o~description\n\
call lineout 'STDERR', r1 s1 '|' r2 s2 '|' r3 s3 '|' r4 s4\n";

/// A private directory holding the program, for the case `name`.
fn program(name: &str) -> (PathBuf, PathBuf) {
    let dir =
        std::env::temp_dir().join(format!("rexx-stdout-errors-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a private probe directory");
    let path = dir.join("errors.rex");
    std::fs::write(&path, PROGRAM).expect("the probe program is writable");
    (dir, path)
}

/// `rexx` on case `name`'s program with `stdout`, answering its standard error and
/// status.
fn run_with_stdout(name: &str, stdout: Stdio) -> (String, Option<i32>) {
    let (dir, path) = program(name);
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
    let (stderr, status) = run_with_stdout("pipe", Stdio::from(writer));
    assert_eq!(status, Some(0), "{stderr}");
    assert_eq!(
        stderr,
        "0 READY: | 1 ERROR:32 Broken pipe | 1 ERROR:32 Broken pipe | 0 READY:\n"
    );
}

/// `/dev/full`: `ENOSPC`, `ERROR:28`.
#[test]
fn a_full_device_is_error_28() {
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full opens");
    let (stderr, status) = run_with_stdout("full", Stdio::from(full));
    assert_eq!(status, Some(0), "{stderr}");
    assert_eq!(
        stderr,
        "0 READY: | 1 ERROR:28 No space left on device | 1 ERROR:28 No space left on device | 0 READY:\n"
    );
}
