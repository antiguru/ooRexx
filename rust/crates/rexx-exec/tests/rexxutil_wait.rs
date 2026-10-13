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

//! `SysWait` in a process of its own: it reaps any child of the process, so
//! running it in a test binary whose other tests start commands would take
//! their children. With no child it answers `-1`, where the oracle answers a
//! status it never set (`corpus/oracle-crashes.txt` entry 45), from the main
//! activity and from two started ones, normally and under the simulation mode.

use std::process::Command;

/// `SysWait` from the main activity and from two activities started together.
const PROGRAM: &str = "a = .w~new~start('WAIT')\nb = .w~new~start('WAIT')\n\
     say SysWait()\nsay a~result\nsay b~result\n\
     ::class w\n::method wait\n  return SysWait()\n";

fn run(dir: &std::path::Path, mode: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rexx-run"));
    command
        .arg(dir.join("p.rex"))
        .stdin(std::process::Stdio::null());
    if let Some(mode) = mode {
        command.env("REXX_SWITCH_MODE", mode);
    }
    command.output().expect("rexx-run runs")
}

#[test]
fn sys_wait_with_no_child_answers_minus_one() {
    let dir = std::env::temp_dir().join(format!("rexx-sys-wait-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), PROGRAM).expect("the program written");
    let runs = [
        None,
        Some("sim:1"),
        Some("sim:2,uniform:1"),
        Some("sim:3,pct:3,k=4"),
    ]
    .map(|mode| (mode, run(&dir, mode)));
    let _ = std::fs::remove_dir_all(&dir);
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
