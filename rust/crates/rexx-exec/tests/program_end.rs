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

//! Program end with an activity that never finishes (the Phase 6 S2-S5
//! plan's Review Focus 2): the oracle waits for ever, and so does
//! `rexx-run`, with main's output written and without spinning. Each test
//! kills the process once it has seen that.

#![allow(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How long main's output may take to appear.
const OUTPUT_WAIT: Duration = Duration::from_secs(30);

/// How long the process is watched once main's output is out.
const WATCH: Duration = Duration::from_secs(1);

/// Runs `source` under `rexx-run`, waits for `main_output` on its stdout,
/// then asserts the process is still running after [`WATCH`] and, on
/// Linux, that it spent at most a tenth of that on the CPU.
fn stays_alive_after(name: &str, source: &str, main_output: &str) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("program-end-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join(format!("{name}.rex"));
    std::fs::write(&file, source).expect("the probe");
    let mut child = Command::new(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(&file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rexx-run starts");
    let mut stdout = child.stdout.take().expect("piped stdout");
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 256];
        while let Ok(read) = stdout.read(&mut buffer) {
            if read == 0 || sender.send(buffer[..read].to_vec()).is_err() {
                break;
            }
        }
    });
    let began = Instant::now();
    let mut seen = Vec::new();
    while seen != main_output.as_bytes() && began.elapsed() < OUTPUT_WAIT {
        if let Ok(bytes) = receiver.recv_timeout(Duration::from_millis(100)) {
            seen.extend(bytes);
        }
    }
    let before = cpu_ticks(&child);
    std::thread::sleep(WATCH);
    let after = cpu_ticks(&child);
    let running = child.try_wait().expect("the child's status").is_none();
    kill(&mut child);
    std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
    assert_eq!(String::from_utf8_lossy(&seen), main_output);
    assert!(running, "{name} ended where the oracle waits for ever");
    if let (Some(before), Some(after)) = (before, after) {
        assert!(
            after - before <= 10,
            "{name} spent {} ticks on the CPU while it waited",
            after - before
        );
    }
}

/// The process's user and system time in clock ticks, where `/proc` has it.
fn cpu_ticks(child: &Child) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", child.id())).ok()?;
    let fields: Vec<&str> = stat.rsplit_once(')')?.1.split_whitespace().collect();
    Some(fields.get(11)?.parse::<u64>().ok()? + fields.get(12)?.parse::<u64>().ok()?)
}

fn kill(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn a_started_message_waiting_on_itself_keeps_the_process_alive() {
    stays_alive_after(
        "self-wait",
        "o = .w~new\nm = .message~new(o, 'selfwait')\no~m = m\nm~start\nsay 'main done'\n\
         ::class w\n::attribute m\n::method selfwait\n  expose m\n  say 'waiting on itself'\n\
         \x20 r = m~result\n  say 'not reached'\n",
        "main done\nwaiting on itself\n",
    );
}

#[test]
fn an_uncancelled_timer_keeps_the_process_alive() {
    stays_alive_after(
        "alarm",
        "a = .Alarm~new(99999, .message~new(.stdout, 'lineout', 'I', 'fired'))\n\
         say 'main done'\n",
        "main done\n",
    );
}
