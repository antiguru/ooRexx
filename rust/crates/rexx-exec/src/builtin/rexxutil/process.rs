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

//! The `REXXUTIL` routines over the process and its terminal:
//! `interpreter/platform/unix/SysRexxUtil.cpp` and
//! `interpreter/runtime/RexxUtilCommon.cpp`.

use std::os::fd::{AsFd, BorrowedFd, IntoRawFd as _};

use rexx_core::ObjRef;
use rustix::io::Errno;
use rustix::termios::{LocalModes, OptionalActions, SpecialCodeIndex};

use super::{answer, c_string, check_arity, int_argument, optional_text, required};
use crate::error::Raised;
use crate::{Failure, Interp, Loud};

/// `SysUtilVersion()`: `ORX_VER.ORX_REL.ORX_MOD`.
pub(crate) fn util_version(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    Ok(interp.text_built(crate::version::VERSION_NUMBER.to_vec()))
}

/// `SysLoadFuncs()` and `SysDropFuncs()`, one body: nothing to do, `0`.
pub(crate) fn load_funcs(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    Ok(answer(interp, 0))
}

/// `SysCls()`: `system("clear")`, answering its raw wait status.
/// The child's output reaches this interpreter's own sinks.
pub(crate) fn cls(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
    check_arity(args, 0)?;
    let mut command = std::process::Command::new("/bin/sh");
    // `system`'s own argument vector, which names the shell in its messages.
    command.arg0("sh").arg("-c").arg("clear").env_clear();
    for (name, value) in &interp.env {
        command.env(
            std::ffi::OsStr::from_bytes(name),
            std::ffi::OsStr::from_bytes(value),
        );
    }
    command.current_dir(interp.cwd_text());
    let status = match command.output() {
        Ok(output) => {
            interp.write_out(&output.stdout);
            interp.write_err(&output.stderr);
            output.status.into_raw()
        }
        // `system` answers -1 where no child could be made.
        Err(_) => -1,
    };
    Ok(answer(interp, i64::from(status)))
}

/// `SysGetKey([echo])`: one byte read from standard input with the terminal
/// out of canonical mode, echoed unless `NOECHO`. The null string where
/// standard input is not a terminal, as `getkey` answers there.
pub(crate) fn get_key(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let echo = match optional_text(interp, args, 1)? {
        None => true,
        Some(text) => {
            let text = c_string(&text);
            if text.eq_ignore_ascii_case(b"NOECHO") {
                false
            } else if text.eq_ignore_ascii_case(b"ECHO") {
                true
            } else {
                return Err(Raised::invalid_routine_option(
                    "SysGetKey",
                    "echo",
                    "'ECHO' or 'NOECHO'",
                    text,
                )
                .into());
            }
        }
    };
    let stdin = std::io::stdin();
    if !interp.input_is_process_stdin() || !rustix::termios::isatty(stdin.as_fd()) {
        return Ok(interp.text_built(Vec::new()));
    }
    interp.hand_over_output();
    let tty = std::fs::File::open("/dev/tty").ok();
    let key = read_key(tty.as_ref().map(AsFd::as_fd), stdin.as_fd(), echo);
    Ok(interp.text_built(key_text(key)))
}

/// What `getkey` stores of the `getchar` answer `key`: the byte as a C
/// string, so a NUL is the null string, and `EOF` truncated to `'FF'x`.
fn key_text(key: Option<u8>) -> Vec<u8> {
    match key {
        Some(0) => Vec::new(),
        Some(byte) => vec![byte],
        None => vec![0xFF],
    }
}

/// Reads one byte from `input` with `tty`, where there is one, out of
/// canonical mode and with echo off unless `echo`, then restores `tty`'s
/// settings. `None` at end of input or on an error, interrupted reads
/// included.
fn read_key(tty: Option<BorrowedFd<'_>>, input: BorrowedFd<'_>, echo: bool) -> Option<u8> {
    let original = tty.and_then(|tty| rustix::termios::tcgetattr(tty).ok());
    if let (Some(tty), Some(original)) = (tty, &original) {
        let mut raw = original.clone();
        raw.local_modes.remove(LocalModes::ICANON);
        if !echo {
            raw.local_modes.remove(LocalModes::ECHO);
        }
        raw.special_codes[SpecialCodeIndex::VMIN] = 1;
        raw.special_codes[SpecialCodeIndex::VTIME] = 0;
        let _ = rustix::termios::tcsetattr(tty, OptionalActions::Now, &raw);
    }
    let mut byte = [0u8; 1];
    let key = match rustix::io::read(input, &mut byte) {
        Ok(1) => Some(byte[0]),
        _ => None,
    };
    if let (Some(tty), Some(original)) = (tty, &original) {
        let _ = rustix::termios::tcsetattr(tty, OptionalActions::Now, original);
    }
    key
}

/// `SysQueryProcess([selector])`: `PID` (the default), `PPID`, `PGID` (the
/// **parent's** group, `getpgid(getppid())`), `PPRIO`, `PTIME`, `PMEM`,
/// `PSWAPS` or `PRCVDSIG`, without regard to case; 40.1 for any other.
pub(crate) fn query_process(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let selector = optional_text(interp, args, 1)?;
    let selector = selector.as_deref().map(c_string);
    let is = |wanted: &[u8]| selector.is_some_and(|text| text.eq_ignore_ascii_case(wanted));
    let pid_number =
        |pid: Option<rustix::process::Pid>| i64::from(rustix::process::Pid::as_raw(pid));
    if selector.is_none() || is(b"PID") {
        let pid = pid_number(Some(rustix::process::getpid()));
        return Ok(answer(interp, pid));
    }
    if is(b"PPID") {
        let pid = pid_number(rustix::process::getppid());
        return Ok(answer(interp, pid));
    }
    if is(b"PGID") {
        let group = rustix::process::getpgid(rustix::process::getppid())
            .map_or(-1, |group| pid_number(Some(group)));
        return Ok(answer(interp, group));
    }
    if is(b"PPRIO") {
        let priority = rustix::process::getpriority_process(None).unwrap_or(-1);
        return Ok(answer(interp, i64::from(priority)));
    }
    let usage = match Usage::of_this_process() {
        Ok(usage) => usage,
        Err(errno) => {
            let text = format!("System error; errno = {}", errno.raw_os_error());
            return Ok(interp.text_built(text.into_bytes()));
        }
    };
    let text = if is(b"PTIME") {
        usage.cpu_summary()
    } else if is(b"PMEM") {
        format!("Max_Memory_RSS: {}", usage.max_rss)
    } else if is(b"PSWAPS") {
        "Memory_swaps: 0".to_owned()
    } else if is(b"PRCVDSIG") {
        "Received_signals: 0".to_owned()
    } else {
        return Err(Raised::routine_failed(&name.to_ascii_uppercase()).into());
    };
    Ok(interp.text_built(text.into_bytes()))
}

/// The `getrusage(RUSAGE_SELF)` fields `SysQueryProcess` reads, from
/// `/proc/self`. Linux keeps no `ru_nswap` or `ru_nsignals`, which are always
/// 0 (`getrusage(2)`), so they have no field here.
struct Usage {
    /// User and system CPU time as seconds and microseconds, at the
    /// resolution of a clock tick.
    user: (u64, u64),
    system: (u64, u64),
    /// The resident set's high-water mark in kilobytes: `VmHWM`.
    max_rss: u64,
}

impl Usage {
    fn of_this_process() -> Result<Usage, Errno> {
        let read = |path: &str| {
            std::fs::read(path).map_err(|error| {
                Errno::from_raw_os_error(error.raw_os_error().unwrap_or(Errno::IO.raw_os_error()))
            })
        };
        let stat = read("/proc/self/stat")?;
        // The fields after the parenthesised command name, which may itself
        // hold blanks: `state` first, then `utime` and `stime` (`proc(5)`
        // fields 14 and 15) at offsets 11 and 12.
        let after_name = stat
            .iter()
            .rposition(|byte| *byte == b')')
            .map_or(&stat[..], |at| &stat[at + 1..]);
        let fields: Vec<&[u8]> = after_name
            .split(|byte| *byte == b' ')
            .filter(|f| !f.is_empty())
            .collect();
        let ticks = |at: usize| -> Result<u64, Errno> {
            fields
                .get(at)
                .and_then(|field| std::str::from_utf8(field).ok()?.parse().ok())
                .ok_or(Errno::IO)
        };
        let hertz = rustix::param::clock_ticks_per_second().max(1);
        let time = |ticks: u64| (ticks / hertz, (ticks % hertz) * 1_000_000 / hertz);
        let user = time(ticks(11)?);
        let system = time(ticks(12)?);
        let status = read("/proc/self/status")?;
        let max_rss = status
            .split(|byte| *byte == b'\n')
            .find_map(|line| line.strip_prefix(b"VmHWM:"))
            .and_then(|rest| {
                std::str::from_utf8(rest)
                    .ok()?
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()
            })
            .unwrap_or(0);
        Ok(Usage {
            user,
            system,
            max_rss,
        })
    }

    /// `PTIME`'s text: the total, kernel and user times, each as
    /// `%2d:%.2d:%.2d:%.3d` hours, minutes, seconds and milliseconds, the
    /// total's milliseconds the sum of the two truncated parts.
    fn cpu_summary(&self) -> String {
        // The C++ works in `unsigned int`.
        let narrow = |value: u64| value as u32;
        let mut milliseconds = narrow(self.user.1 / 1000) + narrow(self.system.1 / 1000);
        let mut seconds = 0;
        if milliseconds >= 1000 {
            seconds = milliseconds / 1000;
            milliseconds %= 1000;
        }
        seconds += narrow(self.user.0);
        seconds += narrow(self.system.0);
        let kernel = clock(narrow(self.system.0), narrow(self.system.1 / 1000));
        let user = clock(narrow(self.user.0), narrow(self.user.1 / 1000));
        format!(
            "CPU_Time Summary: {}  Kernel: {}  User: {}",
            clock(seconds, milliseconds),
            kernel,
            user
        )
    }
}

/// `seconds` and `milliseconds` as `%2d:%.2d:%.2d:%.3d`.
fn clock(seconds: u32, milliseconds: u32) -> String {
    format!(
        "{:2}:{:02}:{:02}:{:03}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60,
        milliseconds
    )
}

/// `SysSetPriority([class], priority)`: `setpriority` of this thread's nice
/// value to `-priority`, answering `0` or its errno. The class is checked as
/// a whole number and otherwise ignored.
pub(crate) fn set_priority(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    if let Some(class) = args.first().copied().flatten() {
        int_argument(interp, class, 1)?;
    }
    let value = required(args, 2)?;
    let priority = int_argument(interp, value, 2)?;
    let nice = i32::try_from(priority)
        .expect("int_argument answers an int")
        .wrapping_neg();
    let code = match rustix::process::setpriority_process(None, nice) {
        Ok(()) => 0,
        Err(errno) => errno.raw_os_error(),
    };
    Ok(answer(interp, i64::from(code)))
}

/// `SysFork()`: refuses, since `fork` is an `unsafe` call this crate holds no
/// grant for.
pub(crate) fn fork(
    _interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    Err(Loud::unsafe_grant(name, "fork(2)").into())
}

/// What `SysWait` answers where `wait` finds no child or fails: the oracle
/// answers a status it never set (`corpus/oracle-crashes.txt` entry 45).
const NO_CHILD: i64 = -1;

/// `SysWait()`: `wait` for any child of this process, answering its raw
/// status, or [`NO_CHILD`]. Any child: another activity's command, or one of
/// another interpreter in the process, as the oracle's `wait` reaps any.
pub(crate) fn wait(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 0)?;
    let status = match rustix::process::wait(rustix::process::WaitOptions::empty()) {
        Ok(Some((_, status))) => i64::from(status.as_raw()),
        Ok(None) | Err(_) => NO_CHILD,
    };
    Ok(answer(interp, status))
}

/// `SysCreatePipe([blocking])`: a pipe as `"read write"`, its descriptors
/// left open and inherited by every child, the read end non-blocking where
/// the argument starts with `N` or `n`. A first character other than `B`,
/// `b`, `N` or `n` is 40.920.
pub(crate) fn create_pipe(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let blocking = match optional_text(interp, args, 1)? {
        None => true,
        Some(text) => match c_string(&text).first() {
            Some(b'n' | b'N') => false,
            Some(b'b' | b'B') => true,
            _ => {
                return Err(Raised::invalid_routine_option(
                    "SysCreatePipe",
                    "blocking",
                    "'B' or 'N'",
                    c_string(&text),
                )
                .into());
            }
        },
    };
    let (reader, writer) = match std::io::pipe() {
        Ok(ends) => ends,
        Err(error) => return Ok(pipe_failed(interp, "Creating pipe", &error)),
    };
    let inherited = [reader.as_fd(), writer.as_fd()]
        .into_iter()
        .try_for_each(|end| rustix::io::fcntl_setfd(end, rustix::io::FdFlags::empty()));
    if let Err(errno) = inherited {
        return Ok(pipe_failed(interp, "Creating pipe", &errno.into()));
    }
    if !blocking {
        let flags = rustix::fs::fcntl_getfl(&reader).and_then(|flags| {
            rustix::fs::fcntl_setfl(&reader, flags | rustix::fs::OFlags::NONBLOCK)
        });
        if let Err(errno) = flags {
            return Ok(pipe_failed(interp, "Setting NONBLOCK flag", &errno.into()));
        }
    }
    let text = format!("{} {}", reader.into_raw_fd(), writer.into_raw_fd());
    Ok(interp.text_built(text.into_bytes()))
}

/// The C++'s `perror` of `what` on standard error, and the null string.
fn pipe_failed(interp: &mut Interp, what: &str, error: &std::io::Error) -> ObjRef {
    let message = std::io::Error::from_raw_os_error(error.raw_os_error().unwrap_or(0)).to_string();
    let message = message
        .rsplit_once(" (os error")
        .map_or(message.as_str(), |(text, _)| text);
    interp.write_err(format!("*** ERROR: {what}: {message}\n").as_bytes());
    interp.text_built(Vec::new())
}

#[cfg(test)]
mod tests;
