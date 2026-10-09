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

//! A seed of the simulation mode replays across processes.

use std::process::Command;

/// Sleeps, reads `TIME` and `DATE`, draws unseeded `RANDOM` in main and in
/// started activities that wait on each other.
const PROGRAM: &str = "call SysSleep 3\nsay random() time('L') date('S') time('E')\n\
     m = .w~new~start('DRAW', 0.5)\nn = .w~new~start('DRAW', 0.25)\n\
     say n~result m~result\nsay random() time('E')\n\
     ::class w\n::method draw\n  use arg pause\n  call SysSleep pause\n  \
     return random() time('L')\n";

fn run(dir: &std::path::Path, mode: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(dir.join("p.rex"))
        .env("REXX_SWITCH_MODE", mode)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("rexx-run runs")
}

#[test]
fn a_seed_gives_one_output_in_two_processes() {
    let dir = std::env::temp_dir().join(format!("rexx-sim-processes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), PROGRAM).expect("the program written");
    let first = run(&dir, "sim:1");
    let second = run(&dir, "sim:1");
    let other = run(&dir, "sim:2");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout.split(|&b| b == b'\n').count(), 4);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    assert!(
        String::from_utf8_lossy(&first.stderr).starts_with("rexx-sim: seed=1 policy=fifo "),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_ne!(first.stdout, other.stdout);
}
