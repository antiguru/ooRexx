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

//! Activities that touch standard input while another reads it: each case
//! runs `rexx-run` with standard input a pipe, writes lines to it (and
//! signals it) at the times its `run` line names, and compares all three
//! descriptors with the oracle's, 30 runs each. Every case depends on a
//! wall-clock boundary (ruling P48), so a mismatch is run again once.

#![allow(
    clippy::disallowed_methods,
    reason = "this harness times or bounds real runs"
)]

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const CASE_DIR: &str = "tests/stdin_contention";

/// How long a case may run before it is killed.
const END_WAIT: Duration = Duration::from_secs(20);

enum Action {
    Write(String),
    Interrupt,
}

/// The case's actions in time order: `writeMS=(LINE,...)` writes the lines,
/// each with a newline, in one write at `MS` milliseconds, and `sigint=MS`
/// sends SIGINT.
fn actions(case: &datadriven::TestCase) -> Vec<(Duration, Action)> {
    let mut actions = Vec::new();
    for (key, values) in &case.args {
        if key == "sigint" {
            let at = values
                .first()
                .expect("a time")
                .parse()
                .expect("milliseconds");
            actions.push((Duration::from_millis(at), Action::Interrupt));
            continue;
        }
        let at = key
            .strip_prefix("write")
            .and_then(|at| at.parse().ok())
            .unwrap_or_else(|| panic!("unknown argument {key}"));
        let text: String = values.iter().map(|line| format!("{line}\n")).collect();
        actions.push((Duration::from_millis(at), Action::Write(text)));
    }
    actions.sort_by_key(|(at, _)| *at);
    actions
}

fn render(label: &str, case: &datadriven::TestCase) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("stdin-contention-{label}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join("p.rex");
    std::fs::write(&file, &case.input).expect("the probe");
    let mut child = Command::new("env")
        .arg("--default-signal=INT,TERM,HUP")
        .arg(env!("CARGO_BIN_EXE_rexx-run"))
        .arg(&file)
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rexx-run starts");
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");
    let out = std::thread::spawn(move || {
        let mut all = Vec::new();
        let _ = stdout.read_to_end(&mut all);
        all
    });
    let err = std::thread::spawn(move || {
        let mut all = Vec::new();
        let _ = stderr.read_to_end(&mut all);
        all
    });
    let began = Instant::now();
    let mut input = child.stdin.take().expect("piped stdin");
    let mut sent = true;
    for (at, action) in actions(case) {
        if let Some(left) = at.checked_sub(began.elapsed()) {
            std::thread::sleep(left);
        }
        match action {
            Action::Write(line) => {
                let _ = input
                    .write_all(line.as_bytes())
                    .and_then(|()| input.flush());
            }
            Action::Interrupt => {
                sent &= Command::new("kill")
                    .args(["-INT", "--", &child.id().to_string()])
                    .status()
                    .is_ok_and(|status| status.success());
            }
        }
    }
    let mut status = None;
    while began.elapsed() < END_WAIT {
        if let Ok(Some(ended)) = child.try_wait() {
            status = Some(ended);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    if status.is_none() {
        let _ = child.kill();
    }
    let _ = child.wait();
    drop(input);
    assert!(sent, "the kill in {label} failed");
    let stdout = out.join().expect("the stdout reader");
    let stderr = err.join().expect("the stderr reader");
    std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
    let mut rendered = match status.and_then(|status| status.code()) {
        Some(code) => format!("rc> {code}\n"),
        None => "rc> none\n".to_string(),
    };
    for line in String::from_utf8_lossy(&stdout).lines() {
        rendered.push_str(&format!("out> {line}\n"));
    }
    let path = file.to_string_lossy().into_owned();
    for line in String::from_utf8_lossy(&stderr)
        .replace(&path, "PROGRAM")
        .lines()
    {
        rendered.push_str(&format!("err> {line}\n"));
    }
    rendered
}

fn file_text(filename: &str) -> String {
    std::fs::read_to_string(filename).expect("the case file")
}

/// What `text` records after the stanza whose program is `input`: the
/// lines from its `----` to the next blank line. `datadriven` keeps its own
/// copy private.
fn expected(text: &str, input: &str) -> String {
    let marker = format!("{input}----\n");
    let Some(at) = text.find(&marker) else {
        return String::new();
    };
    let mut recorded = String::new();
    for line in text[at + marker.len()..].lines() {
        if line.is_empty() {
            break;
        }
        recorded.push_str(line);
        recorded.push('\n');
    }
    recorded
}

#[test]
fn readers_of_standard_input_are_served_as_the_oracles_are() {
    let mut cases = 0usize;
    datadriven::walk(CASE_DIR, |file| {
        let filename = file.filename.clone();
        let mut in_file = 0usize;
        file.run(|case| {
            assert_eq!(
                case.directive, "run",
                "unknown directive {:?}",
                case.directive
            );
            cases += 1;
            in_file += 1;
            let label = format!("case{in_file}");
            let first = render(&label, case);
            if first == expected(&file_text(&filename), &case.input) {
                return first;
            }
            eprintln!("{label}: rerun after\n{first}");
            render(&label, case)
        });
    });
    assert!(cases > 0, "{CASE_DIR} produced no cases");
}
