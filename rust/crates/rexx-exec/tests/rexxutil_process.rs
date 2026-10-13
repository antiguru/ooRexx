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

//! `SysWait` and `SysQueryProcess('PGID')` in a process of their own.

use std::os::unix::process::CommandExt as _;
use std::process::Command;

/// `SysWait` from the main activity and from two activities started together.
const PROGRAM: &str = "a = .w~new~start('WAIT')\nb = .w~new~start('WAIT')\n\
     say SysWait()\nsay a~result\nsay b~result\n\
     ::class w\n::method wait\n  return SysWait()\n";

/// A command that runs `program` in a directory of its own named for `name`,
/// answering the directory too.
fn rexx_run(name: &str, program: &str) -> (Command, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("rexx-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), program).expect("the program written");
    let mut command = Command::new(env!("CARGO_BIN_EXE_rexx-run"));
    command
        .arg(dir.join("p.rex"))
        .stdin(std::process::Stdio::null());
    (command, dir)
}

/// `SysWait` reaps any child of the process, so running it in a test binary
/// whose other tests start commands would take their children. With no child
/// it answers `-1`, where the oracle answers a status it never set
/// (`corpus/oracle-crashes.txt` entry 45), from the main activity and from
/// two started ones, normally and under the simulation mode.
#[test]
fn sys_wait_with_no_child_answers_minus_one() {
    let run = |mode: Option<&str>| {
        let (mut command, dir) = rexx_run("sys-wait", PROGRAM);
        if let Some(mode) = mode {
            command.env("REXX_SWITCH_MODE", mode);
        }
        let output = command.output().expect("rexx-run runs");
        let _ = std::fs::remove_dir_all(&dir);
        output
    };
    let runs = [
        None,
        Some("sim:1"),
        Some("sim:2,uniform:1"),
        Some("sim:3,pct:3,k=4"),
    ]
    .map(|mode| (mode, run(mode)));
    for (mode, output) in runs {
        assert!(
            output.status.success(),
            "{mode:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "-1\n-1\n-1\n",
            "{mode:?}"
        );
    }
}

/// `PGID` is the parent's process group, `getpgid(getppid())`: a child in a
/// group of its own answers this process's group and not its own.
#[test]
fn pgid_is_the_parents_group() {
    let (mut command, dir) = rexx_run(
        "sys-query-pgid",
        "say SysQueryProcess('PGID') SysQueryProcess('PID')\n",
    );
    let output = command.process_group(0).output().expect("rexx-run runs");
    let _ = std::fs::remove_dir_all(&dir);
    let text = String::from_utf8_lossy(&output.stdout);
    let (group, pid) = text.trim().split_once(' ').expect("two words");
    let ours = rustix::process::getpgrp().as_raw_nonzero().get();
    assert_eq!(group, ours.to_string(), "{text}");
    assert_ne!(group, pid, "{text}");
}

/// A command whose child a `SysWait` on another activity reaped answers `127`
/// with a `FAILURE`, where the oracle reads the status of its own failed
/// `waitpid` (`corpus/oracle-crashes.txt` entry 45). The wait itself answers
/// the child's status, `exit 5` as `1280`. The command marks that it has
/// started, and the main activity waits for the mark before it calls
/// `SysWait`, a bounded number of times. `timeout` kills a run that outlives
/// its deadline.
#[test]
fn a_command_whose_child_was_reaped_elsewhere_fails_with_127() {
    let program = "a = .w~new~start('RUN')\n\
        do 400 until SysFileExists('started'); call SysSleep 0.05; end\n\
        say 'wait' SysWait()\n\
        say 'cmd' a~result\n\
        ::class w\n\
        ::method run\n\
        call on failure name failed\n\
        'touch started; sleep 0.6; exit 5'\n\
        return 'rc' rc\n\
        failed:\n\
        say 'failure' rc\n\
        return\n";
    let (_, dir) = rexx_run("sys-wait-reaped", program);
    let output = Command::new("timeout")
        .args(["-k", "5", "60"])
        .arg(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(dir.join("p.rex"))
        .current_dir(&dir)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("timeout runs rexx-run");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        output.status.success(),
        "{:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.sort_unstable();
    assert_eq!(
        lines,
        ["cmd rc 127", "failure 127", "wait 1280"],
        "{stdout}"
    );
}
