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

//! A prompt reaches the terminal before the read that waits for its answer.
//!
//! **No corpus program can see this.** The differential harness supplies
//! stdin as a buffer and compares the two output streams once the run is
//! over, so a prompt written at exit and a prompt written before the read
//! compare equal. What the two differ in is whether a person is looking at
//! the question while the interpreter waits for their answer, which is a
//! property of `rexx-run`'s own descriptors and is only observable by
//! spawning it.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

/// How long a line may take to arrive before the run counts as blocked. Long
/// enough that a loaded machine does not fail it, short enough that a
/// genuinely buffered prompt is not waited on for the harness's own timeout.
const PATIENCE: Duration = Duration::from_secs(20);

/// A private directory for one case's program.
fn probe_dir(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "rexx-prompt-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&base).expect("a private probe directory");
    base
}

/// `rexx-run` on `source`, with all three descriptors piped and **nothing
/// written to its standard input**.
fn waiting_run(name: &str, source: &str) -> (Child, PathBuf) {
    let dir = probe_dir(name);
    let path = dir.join("ask.rex");
    std::fs::write(&path, source).expect("the probe program is writable");
    let child = Command::new(env!("CARGO_BIN_EXE_rexx-run"))
        .arg("ask.rex")
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn rexx-run: {e}"));
    (child, dir)
}

/// Reads `stream` line by line on a thread of its own, so that a line can be
/// waited for with a deadline rather than by blocking the test.
fn lines_of(stream: impl Read + Send + 'static) -> Receiver<String> {
    let (sender, receiver) = channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines() {
            let Ok(line) = line else { return };
            if sender.send(line).is_err() {
                return;
            }
        }
    });
    receiver
}

/// The next line, or a failure naming what was being waited for. A timeout
/// here **is** the defect this file exists for: the line is sitting in the
/// interpreter's buffer while the program waits for an answer to it.
fn next_line(lines: &Receiver<String>, waiting_for: &str, child: &mut Child) -> String {
    match lines.recv_timeout(PATIENCE) {
        Ok(line) => line,
        Err(RecvTimeoutError::Timeout) => {
            let _ = child.kill();
            panic!(
                "nothing arrived within {PATIENCE:?} while waiting for {waiting_for}: the program \
                 is blocked on a read whose prompt has not been written"
            );
        }
        Err(RecvTimeoutError::Disconnected) => {
            let _ = child.kill();
            panic!("the stream ended before {waiting_for} arrived");
        }
    }
}

/// Reads on until a line contains `needle`, which is what a stream carrying
/// more than one line before the interesting one needs. A prompt that is
/// never written times out inside [`next_line`].
fn line_containing(
    lines: &Receiver<String>,
    needle: &str,
    waiting_for: &str,
    child: &mut Child,
) -> String {
    loop {
        let line = next_line(lines, waiting_for, child);
        if line.contains(needle) {
            return line;
        }
    }
}

/// `SAY` then `PARSE PULL`: the question is on the terminal while the
/// interpreter waits for the answer.
#[test]
fn a_say_before_a_pull_arrives_before_the_answer_is_typed() {
    let (mut child, dir) = waiting_run(
        "pull",
        "say 'question'\nparse pull answer\nsay 'answer' answer\n",
    );
    let stdout = lines_of(child.stdout.take().expect("stdout is piped"));

    // Nothing has been written to the child's standard input at this point,
    // and nothing will be until this line has arrived.
    assert_eq!(next_line(&stdout, "the question", &mut child), "question");

    let mut stdin = child.stdin.take().expect("stdin is piped");
    stdin.write_all(b"hello\n").expect("the child is reading");
    drop(stdin);

    assert_eq!(next_line(&stdout, "the answer", &mut child), "answer hello");
    let status = child.wait().expect("the child ends");
    assert_eq!(status.code(), Some(0), "the run ended other than normally");
    let _ = std::fs::remove_dir_all(dir);
}

/// The interactive `TRACE` pause, which is the case the sinks were built for:
/// its banner and prompt go to the trace descriptor and the read that waits
/// for a command is in the same clause.
#[test]
fn an_interactive_trace_prompt_arrives_before_its_command_is_typed() {
    let (mut child, dir) = waiting_run("trace", "trace ?r\nsay 'stepped'\n");
    let stderr = lines_of(child.stderr.take().expect("stderr is piped"));

    // The source banner and the clause's own echo come first; what this
    // waits for is the prompt, which is the line the read is behind.
    line_containing(
        &stderr,
        "Interactive trace",
        "the interactive trace prompt",
        &mut child,
    );

    let mut stdin = child.stdin.take().expect("stdin is piped");
    stdin.write_all(b"\n").expect("the child is reading");
    drop(stdin);

    let status = child.wait().expect("the child ends");
    assert_eq!(status.code(), Some(0), "the run ended other than normally");
    let _ = std::fs::remove_dir_all(dir);
}
