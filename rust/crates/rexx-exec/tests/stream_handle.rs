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

//! The `HANDLE:n` shapes Deviation 35 licenses, run on this crate alone: the
//! oracle reads nothing through such a stream and crashes on a write
//! (`corpus/oracle-crashes.txt` entry 12a), so none of these programs may be
//! run on it. `corpus/lang/stream_handle_names.rex` holds the shapes that
//! agree.

use rexx_exec::{Invocation, Outcome, ProgramInput, run_program};

fn run(program: &str, invocation: Invocation) -> Outcome {
    run_program("stream_handle.rex", program.as_bytes().to_vec(), invocation)
}

fn stdout(outcome: &Outcome) -> String {
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

/// Deviation 35: `HANDLE:0` reads standard input, where the oracle's read
/// answers the null string at `NOTREADY:EOF`.
#[test]
fn handle_zero_reads_standard_input() {
    let program = "say linein('HANDLE:0')\n\
                   s = .Stream~new('HANDLE:0')\n\
                   say s~open('read') s~linein s~state\n";
    let outcome = run(
        program,
        Invocation::none().with_input(ProgramInput::Bytes(b"first\nsecond\n".to_vec())),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "first\nREADY: second READY\n");
}

/// Deviation 35 and `oracle-crashes.txt` entry 12a: `HANDLE:1` and
/// `HANDLE:2` write to standard output and standard error.
#[test]
fn handle_one_and_two_write_standard_output_and_error() {
    let program = "call lineout 'HANDLE:1', 'to out'\n\
                   call lineout 'HANDLE:2', 'to err'\n\
                   s = .Stream~new('handle:1')\n\
                   say s~charout('chars') s~state\n";
    let outcome = run(program, Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "to out\nchars0 READY\n");
    assert_eq!(String::from_utf8_lossy(&outcome.stderr), "to err\n");
}

/// Deviation 35: an opened `HANDLE:n` answers its descriptor for `QUERY
/// HANDLE`, where the oracle answers `18446744073709551615`. Unopened, both
/// answer the null string.
#[test]
fn query_handle_after_open_answers_the_descriptor() {
    let program = "do n = 0 to 2\n\
                   s = .Stream~new('HANDLE:' || n)\n\
                   before = s~query('handle')\n\
                   opened = s~open('read')\n\
                   say '[' || before || ']' s~query('handle')\n\
                   end\n";
    let outcome = run(program, Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "[] 0\n[] 1\n[] 2\n");
}

/// Deviation 35: a second open of a `HANDLE:n` stream opens the descriptor
/// again, where the oracle's opens a file of that name.
#[test]
fn a_second_open_opens_the_descriptor_again() {
    let program = "s = .Stream~new('HANDLE:0')\n\
                   say s~open('read') s~close s~open('read') s~state\n";
    let outcome = run(program, Invocation::none());
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(stdout(&outcome), "READY: READY: READY: READY\n");
}

/// An opened `HANDLE:n`'s stream type is the one the `Invocation` reports for
/// that standard stream. Unopened it is `UNKNOWN`, as on the oracle.
#[test]
fn the_stream_type_follows_the_invocation_once_opened() {
    let program = "do n = 0 to 2\n\
                   s = .Stream~new('HANDLE:' || n)\n\
                   before = s~query('streamtype')\n\
                   opened = s~open('read')\n\
                   say before s~query('streamtype')\n\
                   end\n";
    let outcome = run(
        program,
        Invocation::none().with_standard_transient([false, true, false]),
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        stdout(&outcome),
        "UNKNOWN PERSISTENT\nUNKNOWN TRANSIENT\nUNKNOWN PERSISTENT\n"
    );
}

/// Deviation 35: a descriptor other than `0`, `1` or `2` is refused loudly
/// when the stream is made, where the oracle makes an inert stream. The
/// adjacent `HANDLE:2` is made.
#[test]
fn another_descriptor_is_refused_loudly() {
    let made = run(
        "s = .Stream~new('HANDLE:2'); say s~state\n",
        Invocation::none(),
    );
    assert_eq!(made.exit_code, 0);
    assert_eq!(stdout(&made), "UNKNOWN\n");
    let refused = run(
        "s = .Stream~new('HANDLE:7'); say 'made'\n",
        Invocation::none(),
    );
    assert_eq!(refused.exit_code, 120);
    assert_eq!(stdout(&refused), "");
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        "rexx-exec: a HANDLE: stream on descriptor 7 is not implemented\n"
    );
}

/// An `APPEND` stream opened on a write-only file still writes at the end of
/// it, past what a command appended meanwhile, where the oracle drops the
/// append flag and overwrites that text.
#[test]
fn an_append_stream_on_a_write_only_file_writes_at_the_end() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("rexx-append-wo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a private directory");
    let file = dir.join("wo.txt");
    std::fs::write(&file, "y\n").expect("the file is writable");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o222)).expect("chmod 222");
    let path = file.display();
    let program = format!(
        "w = .stream~new('{path}'); say w~open('write append')\n\
         say w~lineout('w1')\n\
         address system \"echo wcmd >> '{path}'\"\n\
         say w~lineout('w2'); say w~close\n"
    );
    let outcome = run(&program, Invocation::none());
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).expect("chmod 644");
    let held = std::fs::read(&file).expect("the file reads back");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(stdout(&outcome), "READY:\n0\n0\nREADY:\n");
    assert_eq!(String::from_utf8_lossy(&held), "y\nwcmd\nw1\nw2\n");
}
