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
//! `rexx-run` in a subprocess with SIGINT, SIGTERM and SIGHUP at their
//! defaults whatever this process inherited (ruling P62), waits for its
//! readiness line, signals that process by its PID, or its process group as
//! a terminal's Ctrl-C does, and asserts all three descriptors. The expected
//! bytes are the oracle's, measured under `timeout -s SIGNAL 1` or by the
//! same PID or group signal, 30 runs each, except where a test says
//! otherwise.
//!
//! Every test here depends on a wall-clock boundary (ruling P48): the signal
//! must land inside a sleep, and the halt within [`PROMPT`]. A mismatch is
//! run again once, and only a second mismatch fails.

use std::io::Read;
use std::os::unix::process::CommandExt;
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

/// How `rexx-run` is started.
#[derive(Clone, Copy, PartialEq)]
enum Launch {
    Plain,
    /// Through `nohup`, which ignores SIGHUP.
    Nohup,
    /// With SIGINT ignored, as a non-interactive shell starts a background
    /// job.
    IgnoringInt,
    /// In a process group of its own, signalled as a group, with standard
    /// input a pipe held open and never written.
    Group,
    /// With standard input a pipe held open and never written.
    Reading,
}

#[derive(Debug, PartialEq)]
struct Ended {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Runs `source` under `rexx-run` started as `launch`, sends `signal` once
/// `ready` is on its stdout, or after [`UNREADY_WAIT`] where `ready` is
/// empty, and answers how it ended and how long after the signal.
fn run_signalled(
    name: &str,
    source: &str,
    ready: &str,
    signal: &str,
    launch: Launch,
) -> (Ended, Duration) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("signals-{name}-{signal}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join(format!("{name}.rex"));
    std::fs::write(&file, source).expect("the probe");
    let mut command = Command::new("env");
    command.arg("--default-signal=INT,TERM,HUP");
    match launch {
        Launch::Nohup => {
            command.arg("nohup");
        }
        Launch::IgnoringInt => {
            command.args(["sh", "-c", "trap '' INT; exec \"$0\" \"$1\""]);
        }
        Launch::Group => {
            command.process_group(0);
        }
        Launch::Plain | Launch::Reading => {}
    }
    let reads = matches!(launch, Launch::Group | Launch::Reading);
    let mut child = command
        .arg(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(&file)
        .current_dir(&dir)
        .stdin(if reads { Stdio::piped() } else { Stdio::null() })
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
    let target = match launch {
        Launch::Group => format!("-{}", child.id()),
        _ => child.id().to_string(),
    };
    let sent = Command::new("kill")
        .args([&format!("-{signal}"), "--", &target])
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
    drop(child.stdin.take());
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
fn halts(name: &str, source: &str, ready: &str, signal: &str, launch: Launch, expected: &Ended) {
    let mut last = None;
    for _ in 0..2 {
        let (ended, took) = run_signalled(name, source, ready, signal, launch);
        if ended == *expected && took < PROMPT {
            return;
        }
        eprintln!("{name} under SIG{signal}: a mismatch in {took:?}, run again: {ended:?}");
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
    halts(
        "sleep",
        SLEEP,
        "before\n",
        "INT",
        Launch::Plain,
        &sleep_halted(),
    );
}

#[test]
fn sigterm_halts_a_sleep() {
    halts(
        "sleep",
        SLEEP,
        "before\n",
        "TERM",
        Launch::Plain,
        &sleep_halted(),
    );
}

#[test]
fn sighup_halts_a_sleep() {
    halts(
        "sleep",
        SLEEP,
        "before\n",
        "HUP",
        Launch::Plain,
        &sleep_halted(),
    );
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
        Launch::Plain,
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
        Launch::Plain,
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
        Launch::Plain,
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
        Launch::Plain,
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
    halts(
        "nohup",
        SLEEP,
        "before\n",
        "INT",
        Launch::Nohup,
        &sleep_halted(),
    );
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
        Launch::Plain,
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

/// A signal ignored before the interpreter starts stays ignored: under
/// `nohup` SIGHUP does not end the sleep, on the oracle either (rc 0, 30 of
/// 30 runs).
#[test]
fn under_nohup_sighup_stays_ignored() {
    halts(
        "nohup_hup",
        "say 'before'\nrc = SysSleep(2)\nsay 'after' rc\n",
        "before\n",
        "HUP",
        Launch::Nohup,
        &Ended {
            code: Some(0),
            stdout: "before\nafter 0\n".to_string(),
            stderr: String::new(),
        },
    );
}

/// A non-interactive shell starts a background job with SIGINT ignored; the
/// handler is installed over that (ruling P61), as the oracle's is.
#[test]
fn sigint_ignored_at_start_still_halts() {
    halts(
        "ignored",
        SLEEP,
        "before\n",
        "INT",
        Launch::IgnoringInt,
        &sleep_halted(),
    );
}

/// The command's wait is abandoned and the halt raised at once, where the
/// oracle's `waitpid` returns `EINTR`; the child, signalled by PID alone,
/// runs on.
#[test]
fn sigint_ends_a_command_wait() {
    halts(
        "command",
        "say 'a'\naddress system 'sleep 5'\nsay 'rc' rc\n",
        "a\n",
        "INT",
        Launch::Plain,
        &Ended {
            code: Some(252),
            stdout: "a\n".to_string(),
            stderr: "     2 *-* address system 'sleep 5'\n\
                     Error 4 running PROGRAM line 2:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// Ctrl-C at a terminal signals the process group: the child dies of it, and
/// the script halts rather than running its next command.
#[test]
fn ctrl_c_halts_a_script_running_commands() {
    halts(
        "group",
        "say 'a'\naddress system 'sleep 2'\nsay 'b' rc\naddress system 'sleep 2'\nsay 'c' rc\n",
        "a\n",
        "INT",
        Launch::Group,
        &Ended {
            code: Some(252),
            stdout: "a\n".to_string(),
            stderr: "     2 *-* address system 'sleep 2'\n\
                     Error 4 running PROGRAM line 2:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// A command that is the program's last clause still takes the halt.
#[test]
fn sigint_halts_a_last_clause_command() {
    halts(
        "lastcommand",
        "say 'a'\naddress system 'sleep 5'\n",
        "a\n",
        "INT",
        Launch::Plain,
        &Ended {
            code: Some(252),
            stdout: "a\n".to_string(),
            stderr: "     2 *-* address system 'sleep 5'\n\
                     Error 4 running PROGRAM line 2:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// An abandoned command answers `RC` -4 and its clause ends; the oracle's
/// reads `waitpid`'s unset status, which varies from run to run, and runs
/// the same clauses. DEVIATIONS entry 10.
#[test]
fn call_on_halt_sees_an_abandoned_command() {
    halts(
        "commandtrap",
        "call on halt name h\nsay 'a'\naddress system 'sleep 5'\nsay 'b' rc\nexit\n\
         h: say 'halted' sigl; return\n",
        "a\n",
        "INT",
        Launch::Plain,
        &Ended {
            code: Some(0),
            stdout: "a\nb -4\nhalted 4\n".to_string(),
            stderr: String::new(),
        },
    );
}

/// A long builtin in a short loop: the halt is served when the builtin
/// returns, not at the clause count's next check.
#[test]
fn sigint_halts_after_a_long_builtin() {
    halts(
        "builtin",
        "do i = 1 to 30\n  v = copies('ab', 30000000)\nend\nsay 'done'\n",
        "",
        "INT",
        Launch::Plain,
        &Ended {
            code: Some(252),
            stdout: String::new(),
            stderr: "     2 *-*   v = copies('ab', 30000000)\n\
                     Error 4 running PROGRAM line 2:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// The traceback of a halt that interrupted a read through the `.INPUT`
/// monitor.
fn monitor_halted(clause: &str) -> Ended {
    Ended {
        code: Some(252),
        stdout: "ready\n".to_string(),
        stderr: format!(
            "  1457 *-* Method UNKNOWN with scope \"Monitor\" in package \"REXX\" \
             (no source available).\n\
             \x20    2 *-* {clause}\n\
             Error 4 running REXX line 1457:  Program interrupted.\n\
             Error 4.1:  Program interrupted with HALT condition.\n"
        ),
    }
}

#[test]
fn sigint_ends_a_parse_pull() {
    halts(
        "pull",
        "say 'ready'\nparse pull v\nsay 'v=['v']'\n",
        "ready\n",
        "INT",
        Launch::Reading,
        &monitor_halted("parse pull v"),
    );
}

#[test]
fn sigint_ends_a_linein() {
    halts(
        "linein",
        "say 'ready'\nv = linein()\nsay 'v=['v']'\n",
        "ready\n",
        "INT",
        Launch::Reading,
        &monitor_halted("v = linein()"),
    );
}

#[test]
fn sigint_ends_a_charin() {
    halts(
        "charin",
        "say 'ready'\nv = charin()\nsay 'v=['v']'\n",
        "ready\n",
        "INT",
        Launch::Reading,
        &monitor_halted("v = charin()"),
    );
}

/// A read straight from `.STDIN` runs in no monitor frame.
#[test]
fn sigint_ends_a_stdin_linein() {
    halts(
        "stdin",
        "say 'ready'\nv = .stdin~linein\nsay 'v=['v']'\n",
        "ready\n",
        "INT",
        Launch::Reading,
        &Ended {
            code: Some(252),
            stdout: "ready\n".to_string(),
            stderr: "     2 *-* v = .stdin~linein\n\
                     Error 4 running PROGRAM line 2:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// The interrupted read answers the null string, the caller's `CALL ON
/// HALT` handler runs before the next clause, and the counts answer `0`.
#[test]
fn call_on_halt_sees_an_interrupted_read() {
    halts(
        "pulltrap",
        "call on halt name h\nsay 'ready'\nv = linein()\nsay 'v=['v']' chars() lines()\nexit\n\
         h: say 'halted' sigl; return\n",
        "ready\n",
        "INT",
        Launch::Reading,
        &Ended {
            code: Some(0),
            stdout: "ready\nhalted 3\nv=[] 0 0\n".to_string(),
            stderr: String::new(),
        },
    );
}

/// The halt withdraws main's untimed `SysWaitEventSem`, which answers `0` as
/// the oracle's interrupted `sem_wait` does, and main takes 4.1 there; the
/// started activity is woken from its sleep and takes its own (ruling P59).
#[test]
fn sigint_ends_a_semaphore_wait() {
    halts(
        "semaphore",
        "o = .w~new\no~start('w')\nsay 'ready'\nh = SysCreateEventSem()\n\
         rc = SysWaitEventSem(h)\nsay 'after' rc\nexit\n\
         ::class w\n::method w\n  call SysSleep 10\n  say 'w after'\n",
        "ready\n",
        "INT",
        Launch::Plain,
        &Ended {
            code: Some(252),
            stdout: "ready\n".to_string(),
            stderr: "     5 *-* rc = SysWaitEventSem(h)\n\
                     Error 4 running PROGRAM line 5:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n\
                     \x20   10 *-* call SysSleep 10\n\
                     Error 4 running PROGRAM line 10:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// A redirected command's child is waited for on the baton, which hands
/// `a` over only at the end, so the signal goes after a second; Ctrl-C kills
/// the child and the halt is served once the wait returns. The oracle's pipe read
/// returns `EINTR` and it raises 98.923 "Interrupted system call" instead.
/// DEVIATIONS entry 12.
#[test]
fn ctrl_c_halts_a_redirected_command() {
    halts(
        "redirected",
        "say 'a'\naddress system 'sleep 2' with output stem o.\nsay 'b' rc\n\
         address system 'sleep 2'\nsay 'c' rc\n",
        "",
        "INT",
        Launch::Group,
        &Ended {
            code: Some(252),
            stdout: "a\n".to_string(),
            stderr: "     2 *-* address system 'sleep 2' with output stem o.\n\
                     Error 4 running PROGRAM line 2:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}

/// The withdrawn wait answers `0` to a `CALL ON HALT` program, which runs
/// on; the started activity's own halt ends it.
#[test]
fn call_on_halt_sees_a_withdrawn_semaphore_wait() {
    halts(
        "semtrap",
        "call on halt name hh\nh = SysCreateEventSem()\no = .w~new\no~start('w')\n\
         say 'ready'\nr = SysWaitEventSem(h)\nsay 'after' r\nexit\n\
         hh: say 'halted' sigl; return\n\
         ::class w\n::method w\n  call SysSleep 3\n  say 'w after'\n",
        "ready\n",
        "INT",
        Launch::Plain,
        &Ended {
            code: Some(0),
            stdout: "ready\nafter 0\nhalted 7\n".to_string(),
            stderr: "    12 *-* call SysSleep 3\n\
                     Error 4 running PROGRAM line 12:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                .to_string(),
        },
    );
}
