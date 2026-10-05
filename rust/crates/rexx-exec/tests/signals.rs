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

//! A signal halts running Rexx (spec 2026-09-29 section 4): each witness runs
//! `rexx-run` in a subprocess, waits for its readiness line, signals that
//! process by its PID, and asserts all three descriptors. The expected bytes
//! are the oracle's, measured under `timeout -s SIGNAL 1`, 30 runs each,
//! except where a test says otherwise.
//!
//! Every test here depends on a wall-clock boundary (ruling P48): the signal
//! must land inside a sleep, and the halt within [`PROMPT`]. A mismatch is
//! run again once, and only a second mismatch fails.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How long the readiness line may take to appear.
const READY_WAIT: Duration = Duration::from_secs(30);

/// How long a program that writes nothing before it is signalled runs
/// first: what `timeout -s SIGNAL 1` gave the oracle.
const UNREADY_WAIT: Duration = Duration::from_secs(1);

/// How soon after the signal the process must end: well inside the
/// witnesses' five-second sleeps.
const PROMPT: Duration = Duration::from_secs(3);

/// How long a process is given to end before it is killed.
const END_WAIT: Duration = Duration::from_secs(20);

#[derive(Debug, PartialEq)]
struct Ended {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Runs `source` under `rexx-run`, through `nohup` where `nohup`, sends
/// `signal` once `ready` is on its stdout, or after [`UNREADY_WAIT`] where
/// `ready` is empty, and answers how it ended and how long after the signal.
fn run_signalled(
    name: &str,
    source: &str,
    ready: &str,
    signal: &str,
    nohup: bool,
) -> (Ended, Duration) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("signals-{name}-{signal}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join(format!("{name}.rex"));
    std::fs::write(&file, source).expect("the probe");
    let mut command = if nohup {
        let mut command = Command::new("nohup");
        command.arg(env!("CARGO_BIN_EXE_rexx-run"));
        command
    } else {
        Command::new(env!("CARGO_BIN_EXE_rexx-run"))
    };
    let mut child = command
        .arg(&file)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rexx-run starts");
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut buffer = [0u8; 256];
        while let Ok(read) = stdout.read(&mut buffer) {
            if read == 0 || sender.send(buffer[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    let errors = std::thread::spawn(move || {
        let mut all = Vec::new();
        let _ = stderr.read_to_end(&mut all);
        all
    });
    let began = Instant::now();
    let mut seen = Vec::new();
    if ready.is_empty() {
        std::thread::sleep(UNREADY_WAIT);
    }
    while !seen.ends_with(ready.as_bytes()) && began.elapsed() < READY_WAIT {
        if let Ok(bytes) = receiver.recv_timeout(Duration::from_millis(20)) {
            seen.extend(bytes);
        }
    }
    let signalled = Instant::now();
    let sent = Command::new("kill")
        .args(["-s", signal, &child.id().to_string()])
        .status()
        .is_ok_and(|status| status.success());
    let mut status = None;
    while sent && signalled.elapsed() < END_WAIT {
        if let Ok(Some(ended)) = child.try_wait() {
            status = Some(ended);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let took = signalled.elapsed();
    if status.is_none() {
        let _ = child.kill();
    }
    let _ = child.wait();
    assert!(sent, "kill -s {signal} failed");
    reader.join().expect("the stdout reader");
    seen.extend(receiver.try_iter().flatten());
    let stderr = errors.join().expect("the stderr reader");
    let path = file.to_string_lossy().into_owned();
    std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
    let ended = Ended {
        code: status.and_then(|status| status.code()),
        stdout: String::from_utf8_lossy(&seen).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).replace(&path, "PROGRAM"),
    };
    (ended, took)
}

/// [`run_signalled`], asserting it ended as `expected` within [`PROMPT`],
/// with one rerun on a mismatch (ruling P48).
fn halts(name: &str, source: &str, ready: &str, signal: &str, nohup: bool, expected: &Ended) {
    let mut last = None;
    for _ in 0..2 {
        let (ended, took) = run_signalled(name, source, ready, signal, nohup);
        if ended == *expected && took < PROMPT {
            return;
        }
        last = Some((ended, took));
    }
    let (ended, took) = last.expect("a run");
    assert_eq!(ended, *expected, "{name} under SIG{signal}, twice");
    panic!("{name} under SIG{signal} took {took:?} to end, twice");
}

const SLEEP: &str = "say 'before'\nrc = SysSleep(5)\nsay 'after' rc\n";

fn sleep_halted() -> Ended {
    Ended {
        code: Some(252),
        stdout: "before\n".to_string(),
        stderr: "     2 *-* rc = SysSleep(5)\n\
                 Error 4 running PROGRAM line 2:  Program interrupted.\n\
                 Error 4.1:  Program interrupted with HALT condition.\n"
            .to_string(),
    }
}

#[test]
fn sigint_halts_a_sleep() {
    halts("sleep", SLEEP, "before\n", "INT", false, &sleep_halted());
}

#[test]
fn sigterm_halts_a_sleep() {
    halts("sleep", SLEEP, "before\n", "TERM", false, &sleep_halted());
}

#[test]
fn sighup_halts_a_sleep() {
    halts("sleep", SLEEP, "before\n", "HUP", false, &sleep_halted());
}

/// The sleep a halt ends answers `nanosleep`'s EINTR, which `SIGNAL ON
/// HALT` sees assigned at the end of that clause.
#[test]
fn signal_on_halt_takes_a_signal() {
    halts(
        "signal_on",
        "signal on halt\nsay 'ready'\nrc = SysSleep(5)\nsay 'after' rc\nexit\n\
         halt: say 'halted' condition('D') sigl rc\n",
        "ready\n",
        "INT",
        false,
        &Ended {
            code: Some(0),
            stdout: "ready\nhalted  3 4\n".to_string(),
            stderr: String::new(),
        },
    );
}

/// A `CALL ON HALT` handler runs at the boundary after the interrupted
/// clause's, as a condition queued during delivery does.
#[test]
fn call_on_halt_takes_a_signal() {
    halts(
        "call_on",
        "call on halt\nsay 'ready'\nrc = SysSleep(5)\nsay 'after' rc\nexit\n\
         halt: say 'halted' condition('D') sigl; return\n",
        "ready\n",
        "INT",
        false,
        &Ended {
            code: Some(0),
            stdout: "ready\nafter 4\nhalted  4\n".to_string(),
            stderr: String::new(),
        },
    );
}

/// This crate hands `ready` to stdout only once the loop ends, so the
/// signal goes after a second.
#[test]
fn sigint_halts_a_busy_loop() {
    halts(
        "busy",
        "say 'ready'\ndo forever\nend\n",
        "",
        "INT",
        false,
        &Ended {
            code: Some(252),
            stdout: "ready\n".to_string(),
            stderr: "     3 *-*   end\n\
                     Error 4 running PROGRAM line 3:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// The signal wakes the activity parked in `GUARD WHEN`, which takes the
/// halt too. The oracle halts main and leaves that activity parked, so it
/// never ends: killed after 3 s, 30 of 30 runs, with main's three lines on
/// stderr. DEVIATIONS entry 10 in `phase-4-exclusions.txt`.
#[test]
fn sigint_reaches_a_parked_guard_when() {
    halts(
        "guard",
        "o = .t~new\no~start('w')\ncall SysSleep 0.2\nsay 'main sleeping'\n\
         rc = SysSleep(5)\nsay 'main after' rc\nexit\n\
         ::class t\n::method w\n  expose flag\n  flag = 0\n  say 'w parks'\n\
         \x20 guard on when flag\n  say 'w resumed'\n",
        "main sleeping\n",
        "INT",
        false,
        &Ended {
            code: Some(252),
            stdout: "w parks\nmain sleeping\n".to_string(),
            stderr: "     5 *-* rc = SysSleep(5)\n\
                     Error 4 running PROGRAM line 5:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n\
                     \x20   13 *-* guard on when flag\n\
                     Error 4 running PROGRAM line 13:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// Under `nohup` SIGHUP is ignored, so the oracle installs no handler and
/// SIGINT kills it (status 130, 30 of 30 runs); this crate checks each
/// signal and halts. DEVIATIONS entry 10 in `phase-4-exclusions.txt`.
#[test]
fn under_nohup_sigint_still_halts() {
    halts("nohup", SLEEP, "before\n", "INT", true, &sleep_halted());
}

/// With main ended and an activity parked in `GUARD WHEN`, the program waits
/// for it; the signal wakes it and the program ends. The oracle waits on:
/// killed after 3 s, 30 of 30 runs. DEVIATIONS entry 10.
#[test]
fn sigint_ends_a_program_waiting_on_a_guard_when() {
    halts(
        "endguard",
        "o = .t~new\no~start('w')\ncall SysSleep 0.2\nsay 'main done'\nexit\n\
         ::class t\n::method w\n  expose flag\n  flag = 0\n  guard on when flag\n",
        "main done\n",
        "INT",
        false,
        &Ended {
            code: Some(0),
            stdout: "main done\n".to_string(),
            stderr: "    10 *-* guard on when flag\n\
                     Error 4 running PROGRAM line 10:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}
