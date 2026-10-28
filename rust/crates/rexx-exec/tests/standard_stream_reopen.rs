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

//! `.STDOUT` and `.STDERR` reopened or closed by a program: each case of
//! `tests/standard_stream_reopen/cases` runs `rexx` with standard output and
//! error pipes, and all three descriptors must be what the case records.
//!
//! **Not the oracle's answer.** `SAY` writes `.OUTPUT`, which is `.STDOUT`,
//! so a write through either keeps program order, and a stream stays open
//! until the program ends (rexxref iostrms.xml), which writes it. The oracle
//! buffers a `.STDOUT` reopened without `NOBUFFER`, `SAY` included, and drops
//! the buffer at the end.

use std::path::PathBuf;
use std::process::{Command, Stdio};

const CASE_DIR: &str = "tests/standard_stream_reopen";

fn render(label: &str, program: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "standard-stream-reopen-{label}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join("p.rex");
    std::fs::write(&file, program).expect("the probe");
    let output = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(&file)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("rexx runs");
    std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
    let mut rendered = match output.status.code() {
        Some(code) => format!("rc> {code}\n"),
        None => "rc> none\n".to_string(),
    };
    for (tag, bytes) in [("out", &output.stdout), ("err", &output.stderr)] {
        for line in String::from_utf8_lossy(bytes).lines() {
            rendered.push_str(&format!("{tag}> {line}\n"));
        }
    }
    rendered
}

#[test]
fn a_reopened_standard_stream_keeps_its_writes_in_order() {
    let mut cases = 0usize;
    datadriven::walk(CASE_DIR, |file| {
        file.run(|case| {
            assert_eq!(
                case.directive, "run",
                "unknown directive {:?}",
                case.directive
            );
            cases += 1;
            render(&format!("case{cases}"), &case.input)
        });
    });
    assert!(cases > 0, "{CASE_DIR} produced no cases");
}

/// `SAY` after a reopen, past what two of the oracle's buffers hold: every
/// line arrives, the last one included.
#[test]
fn say_after_a_reopen_writes_every_line() {
    let program = "say 'a'\n.stdout~open('write')\n\
                   do i = 1 to 1000; say copies('y', 9); end\nsay 'end'\n";
    let rendered = render("say", program);
    let lines: Vec<&str> = rendered.lines().collect();
    assert_eq!(lines.len(), 1003, "{rendered}");
    assert_eq!(lines[..2], ["rc> 0", "out> a"]);
    assert!(lines[2..1002].iter().all(|line| *line == "out> yyyyyyyyy"));
    assert_eq!(lines[1002], "out> end");
}
