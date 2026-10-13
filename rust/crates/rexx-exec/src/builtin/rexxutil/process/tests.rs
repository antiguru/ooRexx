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

use std::os::fd::{AsFd as _, OwnedFd};
use std::time::Duration;

use rustix::termios::LocalModes;

use super::{Usage, key_text, read_key};

/// A pseudo-terminal: its controlling side and the terminal a program reads.
fn pseudo_terminal() -> (OwnedFd, OwnedFd) {
    use rustix::fs::{Mode, OFlags};
    use rustix::pty::OpenptFlags;
    let controller =
        rustix::pty::openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("a pseudo-terminal");
    rustix::pty::grantpt(&controller).expect("grantpt");
    rustix::pty::unlockpt(&controller).expect("unlockpt");
    let name = rustix::pty::ptsname(&controller, Vec::new()).expect("ptsname");
    let terminal = rustix::fs::open(
        name.as_c_str(),
        OFlags::RDWR | OFlags::NOCTTY,
        Mode::empty(),
    )
    .expect("the terminal side");
    (controller, terminal)
}

/// What the terminal echoed to its controlling side, waiting at most a
/// moment for it.
fn echoed(controller: &OwnedFd) -> Vec<u8> {
    use rustix::event::{PollFd, PollFlags, Timespec};
    let mut fds = [PollFd::new(controller, PollFlags::IN)];
    let wait = Timespec {
        tv_sec: 0,
        tv_nsec: 200_000_000,
    };
    if rustix::event::poll(&mut fds, Some(&wait)).unwrap_or(0) == 0 {
        return Vec::new();
    }
    let mut buffer = [0u8; 16];
    let read = rustix::io::read(controller, &mut buffer).unwrap_or(0);
    buffer[..read].to_vec()
}

/// Reads a key from the terminal's side with `echo`, after `typed` was typed
/// with no line end, answering the key, the echo and the terminal's local
/// modes after the read. A read still waiting after five seconds fails the
/// test rather than hanging it.
fn key_typed(typed: &[u8], echo: bool) -> (Option<u8>, Vec<u8>, LocalModes, LocalModes) {
    let (controller, terminal) = pseudo_terminal();
    let before = rustix::termios::tcgetattr(&terminal)
        .expect("terminal settings")
        .local_modes;
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = terminal.try_clone().expect("a second handle");
    std::thread::spawn(move || {
        let key = read_key(Some(reader.as_fd()), reader.as_fd(), echo);
        let _ = sender.send(key);
    });
    // Typed once the reader has left canonical mode, so the echo is the one
    // the reader asked for.
    let mut polls = 0;
    while rustix::termios::tcgetattr(&terminal)
        .expect("terminal settings")
        .local_modes
        .contains(LocalModes::ICANON)
    {
        polls += 1;
        assert!(polls < 5000, "the mode never changed");
        if let Ok(key) = receiver.recv_timeout(Duration::from_millis(1)) {
            panic!("answered {key:?} before a key was typed");
        }
    }
    rustix::io::write(&controller, typed).expect("typed");
    let key = receiver.recv_timeout(Duration::from_secs(5));
    if key.is_err() {
        let _ = rustix::io::write(&controller, b"\n");
    }
    let key = key.expect("the read answers without a line end");
    let after = rustix::termios::tcgetattr(&terminal)
        .expect("terminal settings")
        .local_modes;
    (key, echoed(&controller), before, after)
}

#[test]
fn a_key_is_read_without_a_line_end_and_echoed() {
    let (key, echo, before, after) = key_typed(b"k", true);
    assert_eq!(key, Some(b'k'));
    assert_eq!(echo, b"k");
    assert!(before.contains(LocalModes::ICANON | LocalModes::ECHO));
    assert_eq!(after, before, "the settings are restored");
}

#[test]
fn noecho_reads_one_key_and_echoes_nothing() {
    let (key, echo, before, after) = key_typed(b"qr", false);
    assert_eq!(key, Some(b'q'));
    assert_eq!(echo, b"");
    assert_eq!(after, before, "the settings are restored");
}

#[test]
fn a_key_is_its_c_string() {
    assert_eq!(key_text(Some(b'a')), b"a");
    assert_eq!(key_text(Some(0)), b"");
    assert_eq!(key_text(None), b"\xFF");
}

#[test]
fn ptime_carries_milliseconds_and_splits_the_clock() {
    let usage = Usage {
        user: (3725, 999_999),
        system: (1, 2_000),
        max_rss: 0,
    };
    assert_eq!(
        usage.cpu_summary(),
        "CPU_Time Summary:  1:02:07:001  Kernel:  0:00:01:002  User:  1:02:05:999"
    );
    let long = Usage {
        user: (360_000, 0),
        system: (0, 0),
        max_rss: 0,
    };
    assert_eq!(
        long.cpu_summary(),
        "CPU_Time Summary: 100:00:00:000  Kernel:  0:00:00:000  User: 100:00:00:000"
    );
}

#[test]
fn this_process_has_a_usage() {
    let usage = Usage::of_this_process().expect("/proc/self is readable");
    assert!(usage.max_rss > 0);
}

/// A read that fails, from a descriptor open only for writing: no key, and
/// the terminal's settings restored all the same.
#[test]
fn a_failed_read_answers_none_and_restores_the_settings() {
    let (_controller, terminal) = pseudo_terminal();
    let before = rustix::termios::tcgetattr(&terminal)
        .expect("terminal settings")
        .local_modes;
    let (_reader, writer) = std::io::pipe().expect("a pipe");
    assert_eq!(
        read_key(Some(terminal.as_fd()), writer.as_fd(), false),
        None
    );
    let after = rustix::termios::tcgetattr(&terminal)
        .expect("terminal settings")
        .local_modes;
    assert_eq!(after, before, "the settings are restored");
}

/// **`SysCls` runs its shell in a working directory whose name is not
/// UTF-8**, as `system` does wherever the process stands. `-1` is the answer
/// for no child at all. The adjacent success is the same call from a
/// directory whose name is UTF-8.
#[test]
fn sys_cls_runs_in_a_directory_whose_name_is_not_utf8() {
    use std::os::unix::ffi::OsStrExt as _;
    let base = std::env::temp_dir().join(format!("rexx-cls-{}", std::process::id()));
    let plain = base.join("plain");
    let mut name = b"odd".to_vec();
    name.push(0xff);
    let odd = base.join(std::ffi::OsStr::from_bytes(&name));
    std::fs::create_dir_all(&plain).expect("a directory");
    std::fs::create_dir_all(&odd).expect("a directory");
    let mut source = b"'cd' '".to_vec();
    source.extend_from_slice(plain.as_os_str().as_bytes());
    source.extend_from_slice(b"'\nr = SysCls()\nsay 'plain' (r \\= -1)\n'cd' '");
    source.extend_from_slice(odd.as_os_str().as_bytes());
    source.extend_from_slice(b"'\nr = SysCls()\nsay 'odd' (r \\= -1)\n");
    let outcome = crate::run_program("/t.rex", source, crate::Invocation::none());
    let _ = std::fs::remove_dir_all(&base);
    assert_eq!(outcome.exit_code, 0);
    let stdout = String::from_utf8_lossy(&outcome.stdout).into_owned();
    let answers: Vec<&str> = ["plain ", "odd "]
        .iter()
        .filter_map(|label| {
            stdout
                .find(label)
                .map(|at| &stdout[at..at + label.len() + 1])
        })
        .collect();
    assert_eq!(answers, ["plain 1", "odd 1"], "{stdout:?}");
}
