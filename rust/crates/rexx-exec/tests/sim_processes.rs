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

/// The timed program of the mode's first test (`sim/tests.rs`, `TIMED`) with
/// a native routine call, under a policy that preempts.
const TIMED_NATIVE: &str = "call time 'R'\ncall SysSleep 5\ne = time('E')\n\
     say 'slept' (e >= 5) (e < 6)\nalarm = .Alarm~new(1, .Message~new(.ringer, 'RING'))\n\
     call SysSleep 2\nsay 'rang' .ringer~rung\nsem = .EventSemaphore~new\n\
     say 'waited' sem~wait(1)\ne = time('E')\nsay 'elapsed' (e >= 8) (e < 9)\n\
     say 'root' RxCalcSqrt(16)\n\
     ::requires 'rxmath' LIBRARY\n\
     ::class ringer\n::attribute rung class\n::method init class\n  self~rung = 0\n\
     ::method ring class\n  self~rung = 1\n";

#[test]
fn a_seed_gives_one_trace_hash_in_two_processes() {
    let lib = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib");
    let dir = std::env::temp_dir().join(format!("rexx-sim-hash-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), TIMED_NATIVE).expect("the program written");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_rexx-run"))
            .arg(dir.join("p.rex"))
            .env("REXX_SWITCH_MODE", "sim:1,uniform:0.3")
            .env("LD_LIBRARY_PATH", &lib)
            .stdin(std::process::Stdio::null())
            .output()
            .expect("rexx-run runs")
    };
    let first = run();
    let second = run();
    let _ = std::fs::remove_dir_all(&dir);
    let stderr = String::from_utf8_lossy(&first.stderr).into_owned();
    assert!(first.status.success(), "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&first.stdout),
        "slept 1 1\nrang 1\nwaited 0\nelapsed 1 1\nroot 4\n"
    );
    assert!(stderr.contains(" trace="), "{stderr}");
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

/// A trace whose header's hash does not match its decisions is refused
/// before the program runs, and the file as written replays.
#[test]
fn a_damaged_trace_is_refused_before_the_run() {
    let dir = std::env::temp_dir().join(format!("rexx-sim-damaged-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), PROGRAM).expect("the program written");
    let file = dir.join("t.txt");
    let recorded = run(&dir, &format!("sim:1,uniform:0.3,trace={}", file.display()));
    let written = std::fs::read_to_string(&file).expect("the trace written");
    let replayed = run(&dir, &format!("sim:replay={}", file.display()));
    let (header, rest) = written.split_once('\n').expect("a header");
    let (configuration, hash) = header.rsplit_once(" hash=").expect("a hash");
    std::fs::write(
        &file,
        format!("{configuration} hash=0123456789abcdef\n{rest}"),
    )
    .expect("the trace damaged");
    let damaged = run(&dir, &format!("sim:replay={}", file.display()));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(recorded.status.success());
    assert!(replayed.status.success());
    assert_eq!(recorded.stdout, replayed.stdout);
    assert_eq!(damaged.status.code(), Some(2));
    assert_eq!(damaged.stdout, b"");
    assert_eq!(
        String::from_utf8_lossy(&damaged.stderr),
        format!(
            "rexx-run: REXX_SWITCH_MODE: `{}`: the decisions hash to {hash}, the header holds \
             0123456789abcdef\n",
            file.display()
        )
    );
}
