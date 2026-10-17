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
//! `rexx`, with main's output written and without spinning. Each test
//! kills the process once it has seen that.

#![expect(
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

/// Runs `source` under `rexx`, waits for `main_output` on its stdout,
/// then asserts the process is still running after [`WATCH`] and, on
/// Linux, that it spent at most a tenth of that on the CPU.
fn stays_alive_after(name: &str, source: &str, main_output: &str) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("program-end-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the probe directory");
    let file = dir.join(format!("{name}.rex"));
    std::fs::write(&file, source).expect("the probe");
    let mut child = Command::new(env!("CARGO_BIN_EXE_rexx"))
        .arg(&file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rexx starts");
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

/// Standard stream buffering under a kill (Deviation 40): standard error and
/// a terminal standard output are handed over at each line end, and a
/// redirected standard output in blocks.
mod buffering {
    use std::fs::File;
    use std::io::Read;
    use std::os::fd::OwnedFd;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    /// How long a program may take to show it is ready to be killed.
    const READY_WAIT: Duration = Duration::from_secs(30);

    /// Where a descriptor of the child goes.
    #[derive(Clone, Copy)]
    enum Target {
        Pipe,
        File,
        Terminal,
    }

    /// One of the child's two output descriptors.
    #[derive(Clone, Copy)]
    enum Stream {
        Out,
        Err,
    }

    /// What reached one descriptor so far.
    enum Reading {
        Pipe(Arc<Mutex<Vec<u8>>>, std::thread::JoinHandle<()>),
        File(PathBuf),
        Terminal(OwnedFd, Vec<u8>),
    }

    impl Reading {
        fn open(target: Target, path: PathBuf) -> (Stdio, Reading) {
            match target {
                Target::Pipe => {
                    let (mut reader, writer) = std::io::pipe().expect("a pipe");
                    let seen = Arc::new(Mutex::new(Vec::new()));
                    let filled = Arc::clone(&seen);
                    let read = std::thread::spawn(move || {
                        let mut buffer = [0u8; 4096];
                        while let Ok(read) = reader.read(&mut buffer) {
                            if read == 0 {
                                break;
                            }
                            filled.lock().expect("the bytes").extend(&buffer[..read]);
                        }
                    });
                    (Stdio::from(writer), Reading::Pipe(seen, read))
                }
                Target::File => (
                    Stdio::from(File::create(&path).expect("the output file")),
                    Reading::File(path),
                ),
                Target::Terminal => {
                    let (controller, terminal) = pseudo_terminal();
                    (
                        Stdio::from(terminal),
                        Reading::Terminal(controller, Vec::new()),
                    )
                }
            }
        }

        /// The bytes so far, without waiting for more.
        fn seen(&mut self) -> Vec<u8> {
            match self {
                Reading::Pipe(seen, _) => seen.lock().expect("the bytes").clone(),
                Reading::File(path) => std::fs::read(path).expect("the output file"),
                Reading::Terminal(controller, seen) => {
                    use rustix::event::{PollFd, PollFlags, Timespec};
                    let wait = Timespec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    let mut buffer = [0u8; 4096];
                    loop {
                        let mut fds = [PollFd::new(&*controller, PollFlags::IN)];
                        if rustix::event::poll(&mut fds, Some(&wait)).unwrap_or(0) == 0 {
                            break;
                        }
                        match rustix::io::read(&*controller, &mut buffer) {
                            Ok(read) if read > 0 => seen.extend_from_slice(&buffer[..read]),
                            _ => break,
                        }
                    }
                    seen.clone()
                }
            }
        }

        /// Every byte, once the child is gone.
        fn finish(mut self) -> Vec<u8> {
            if let Reading::Pipe(seen, read) = self {
                read.join().expect("the reader");
                return std::mem::take(&mut *seen.lock().expect("the bytes"));
            }
            self.seen()
        }
    }

    /// A pseudo-terminal: its controlling side and the terminal side.
    fn pseudo_terminal() -> (OwnedFd, OwnedFd) {
        use rustix::fs::{Mode, OFlags};
        use rustix::pty::OpenptFlags;
        let controller = rustix::pty::openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY)
            .expect("a pseudo-terminal");
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

    fn probe(name: &str, source: &str) -> (PathBuf, PathBuf) {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("buffering-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the probe directory");
        let file = dir.join(format!("{name}.rex"));
        std::fs::write(&file, source).expect("the probe");
        (dir, file)
    }

    /// What reached stdout and stderr when `source`, run under `rexx` with
    /// each on its target, was killed once `ready` reached the descriptor it
    /// names. Panics if it did not within [`READY_WAIT`], or if the program
    /// ended first.
    fn killed(
        name: &str,
        source: &str,
        out: Target,
        err: Target,
        ready: (Stream, &str),
    ) -> (Vec<u8>, Vec<u8>) {
        let (dir, file) = probe(name, source);
        let (stdout, mut read_out) = Reading::open(out, dir.join("stdout"));
        let (stderr, mut read_err) = Reading::open(err, dir.join("stderr"));
        let mut child = Command::new(env!("CARGO_BIN_EXE_rexx"))
            .arg(&file)
            .current_dir(&dir)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .expect("rexx starts");
        let (stream, marker) = ready;
        let began = Instant::now();
        let mut was_ready = false;
        while !was_ready && began.elapsed() < READY_WAIT {
            let seen = match stream {
                Stream::Out => read_out.seen(),
                Stream::Err => read_err.seen(),
            };
            was_ready = seen
                .windows(marker.len())
                .any(|window| window == marker.as_bytes());
            if !was_ready {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let running = child.try_wait().expect("the child's status").is_none();
        super::kill(&mut child);
        let seen = (read_out.finish(), read_err.finish());
        std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
        assert!(was_ready, "{name} never wrote {marker:?}");
        assert!(running, "{name} ended before the kill");
        seen
    }

    /// What reached stdout and stderr sharing one pipe, for a program that
    /// ends.
    fn merged(name: &str, source: &str) -> Vec<u8> {
        let (dir, file) = probe(name, source);
        let (mut reader, writer) = std::io::pipe().expect("a pipe");
        let mut child = Command::new(env!("CARGO_BIN_EXE_rexx"))
            .arg(&file)
            .current_dir(&dir)
            .stdin(Stdio::null())
            .stdout(writer.try_clone().expect("a second writer"))
            .stderr(writer)
            .spawn()
            .expect("rexx starts");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).expect("the pipe");
        assert!(child.wait().expect("rexx ends").success());
        std::fs::remove_dir_all(&dir).expect("the probe directory is removed");
        bytes
    }

    const TARGETS: [Target; 2] = [Target::Pipe, Target::File];

    #[test]
    fn standard_error_keeps_each_line() {
        for target in TARGETS {
            for (source, seen) in [
                (".error~say('started X'); do forever; end\n", "started X\n"),
                (
                    "trace r; x = 1; trace o; do forever; end\n",
                    "     1 *-* x = 1;\n       >>>   \"1\"\n     1 *-* trace o;\n",
                ),
            ] {
                let (out, err) = killed("error", source, Target::Pipe, target, (Stream::Err, seen));
                assert_eq!(String::from_utf8_lossy(&err), seen);
                assert!(out.is_empty());
            }
        }
    }

    /// A standard error line hands nothing on standard output over, so the
    /// `ready` after the `SAY`s does not disturb what they left buffered.
    #[test]
    fn a_redirected_standard_output_loses_what_it_still_holds() {
        for target in TARGETS {
            for source in [
                "say 'started X'; .error~say('ready'); do forever; end\n",
                "do i = 1 to 300; say 'line' i; end; .error~say('ready'); do forever; end\n",
            ] {
                let (out, _) = killed("held", source, target, Target::Pipe, (Stream::Err, "ready"));
                assert_eq!(String::from_utf8_lossy(&out), "");
            }
        }
    }

    /// The command writes until its reader is gone, so it dies of `SIGPIPE`
    /// with `rexx`.
    #[test]
    fn a_command_starts_after_what_came_before() {
        for target in TARGETS {
            for source in [
                "say 'started X'; 'while :; do echo; sleep 0.1; done'\n",
                "say 'started X'; address system 'while :; do echo; sleep 0.1; done' \
                 with output stem lines.\n",
            ] {
                let (out, _) = killed(
                    "slow-command",
                    source,
                    target,
                    Target::Pipe,
                    (Stream::Out, "started X\n"),
                );
                assert!(out.starts_with(b"started X\n"));
            }
        }
    }

    /// Past 64 KiB the buffer is handed over whole, so what survives is a
    /// prefix of whole hand-overs, and the tail is not. A `SAY`, a `CHAROUT`
    /// to standard output and a `CHAROUT` to standard error each pass it; the
    /// last ends no line, and its `ready` is a line on a terminal standard
    /// output.
    #[test]
    fn a_redirected_standard_output_keeps_what_it_handed_over() {
        let lines: Vec<u8> = (1..=20000)
            .flat_map(|i| format!("line {i}\n").into_bytes())
            .collect();
        let pieces = b"abcdefgh.".repeat(20000);
        for target in TARGETS {
            let cases = [
                (
                    "do i = 1 to 20000; say 'line' i; end; .error~say('ready'); do forever; end\n",
                    Stream::Out,
                    &lines,
                ),
                (
                    "do 20000; call charout 'STDOUT', 'abcdefgh.'; end; .error~say('ready'); \
                     do forever; end\n",
                    Stream::Out,
                    &pieces,
                ),
                (
                    "do 20000; call charout 'STDERR', 'abcdefgh.'; end; say 'ready'; \
                     do forever; end\n",
                    Stream::Err,
                    &pieces,
                ),
            ];
            for (source, stream, all) in cases {
                let (kept, ready) = match stream {
                    Stream::Out => (target, Target::Pipe),
                    Stream::Err => (Target::Terminal, target),
                };
                let (out, err) = match stream {
                    Stream::Out => killed("bound", source, kept, ready, (Stream::Err, "ready\n")),
                    Stream::Err => killed("bound", source, kept, ready, (Stream::Out, "ready\r\n")),
                };
                let (subject, marker) = match stream {
                    Stream::Out => (out, err),
                    Stream::Err => (err, out),
                };
                assert!(String::from_utf8_lossy(&marker).starts_with("ready"));
                assert!(
                    subject.len() > 64 << 10,
                    "{} bytes kept: {source}",
                    subject.len()
                );
                assert!(
                    subject.len() < all.len(),
                    "{} bytes kept: {source}",
                    subject.len()
                );
                assert!(all.starts_with(&subject));
            }
        }
    }

    /// The oracle orders both as written. Here a command's output follows
    /// what came before it, and a standard error line is handed over ahead
    /// of the standard output still buffered.
    #[test]
    fn a_shared_pipe_interleaves_by_hand_over() {
        assert_eq!(
            String::from_utf8_lossy(&merged("command", "say 'a'; 'echo b'; say 'c'\n")),
            "a\nb\nc\n"
        );
        assert_eq!(
            String::from_utf8_lossy(&merged(
                "lineout",
                "say 'a'; call lineout 'STDERR', 'b'; say 'c'\n"
            )),
            "b\na\nc\n"
        );
    }

    /// A write that ends a line hands over the whole buffer, the line not yet
    /// ended after it too, and a write that ends none waits, as Python's
    /// line buffering does, which counts a carriage return as a line end too.
    /// The terminal turns `\n` into `\r\n`.
    #[test]
    fn a_terminal_standard_output_takes_each_line() {
        for (source, seen, ready) in [
            (
                "say 'started X'; do forever; end\n",
                "started X\r\n",
                (Stream::Out, "started X\r\n"),
            ),
            (
                "call charout , 'a'; .error~say('ready'); do forever; end\n",
                "",
                (Stream::Err, "ready"),
            ),
            (
                "call charout , 'a'; call charout , 'b' || '0a'x || 'c'; do forever; end\n",
                "ab\r\nc",
                (Stream::Out, "ab\r\nc"),
            ),
            (
                "call charout , 'a' || '0d'x || 'b'; do forever; end\n",
                "a\rb",
                (Stream::Out, "a\rb"),
            ),
        ] {
            let (out, _) = killed("terminal", source, Target::Terminal, Target::Pipe, ready);
            assert_eq!(String::from_utf8_lossy(&out), seen);
        }
    }
}
