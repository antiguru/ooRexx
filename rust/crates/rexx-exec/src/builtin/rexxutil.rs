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

//! The `REXXUTIL` package's file-system, sleep, semaphore and version routines --
//! `interpreter/platform/unix/SysRexxUtil.cpp` and the shared
//! `RexxUtilCommon.cpp`.

use std::time::Instant;

use rexx_core::{Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};

use crate::error::Raised;
use crate::scheduler::ParkReason;
use crate::semaphores::{SEM_POLL, SemaphoreKey, SemaphoreWait, WaitKind};
use crate::{Failure, Interp, Loud};

/// The one path argument these routines take, resolved against the
/// interpreter's own directory rather than the process's.
///
/// `position` is the argument number the missing-argument message names.
fn path_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: &str,
    arity: usize,
) -> Result<String, Failure> {
    if args.len() > arity {
        return Err(Raised::too_many_internal_arguments(arity).into());
    }
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_internal_argument(position).into());
    };
    let text = interp.to_text(value).into_owned();
    if text.is_empty() {
        return Ok(String::new());
    }
    let cwd = interp.cwd_text();
    Ok(crate::paths::normalize(
        &String::from_utf8_lossy(&text),
        &cwd,
    ))
}

/// The answer these routines give: a whole number, as text.
fn answer(interp: &mut Interp, value: i64) -> ObjRef {
    ObjRef::small_int(value).unwrap_or_else(|| interp.text_built(value.to_string().into_bytes()))
}

/// `SysFileExists(name)`: whether anything of that name is there, a directory
/// included -- measured, `SysFileExists('.')` is `1` where `SysIsFile('.')` is
/// `0`.
pub(crate) fn file_exists(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let path = path_argument(interp, args, "1", 1)?;
    let found = !path.is_empty() && std::fs::metadata(&path).is_ok();
    Ok(answer(interp, i64::from(found)))
}

/// `SysIsFile(name)`: whether it is there **and** is a regular file.
pub(crate) fn is_file(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let path = path_argument(interp, args, "1", 1)?;
    let found = !path.is_empty() && std::fs::metadata(&path).is_ok_and(|meta| meta.is_file());
    Ok(answer(interp, i64::from(found)))
}

/// `SysFileDelete(name)`: `SysFileSystem::deleteFile`, which is an
/// `access(W_OK)` pre-check and then `unlink`.
///
/// **The pre-check collapses every one of its own failures to `EACCES`**, so a
/// name that is not there answers `13` and not the `ENOENT` the delete itself
/// would give -- measured, and a port calling `std::fs::remove_file` and
/// mapping its error would answer `2`.
pub(crate) fn file_delete(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let path = path_argument(interp, args, "1", 1)?;
    const EACCES: i64 = 13;
    if path.is_empty() || !rustix::fs::access(path.as_str(), rustix::fs::Access::WRITE_OK).is_ok() {
        return Ok(answer(interp, EACCES));
    }
    let code = match rustix::fs::unlink(path.as_str()) {
        Ok(()) => 0,
        Err(errno) => i64::from(errno.raw_os_error()),
    };
    Ok(answer(interp, code))
}

/// `SysRmDir(name)`: `remove(2)`, which unlinks a name that is not a
/// directory -- measured, this deletes a plain file and answers `0`, where
/// `rmdir(2)` alone would answer `ENOTDIR`. `unlink` first and `rmdir` only
/// where that says the name is a directory is the same order glibc's own
/// `remove` takes, and it reproduces every measured code: `2` for a missing
/// name and `39` for a directory with entries.
pub(crate) fn rm_dir(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let path = path_argument(interp, args, "1", 1)?;
    let code = match rustix::fs::unlink(path.as_str()) {
        Ok(()) => 0,
        Err(errno) if errno == rustix::io::Errno::ISDIR || errno == rustix::io::Errno::PERM => {
            match rustix::fs::rmdir(path.as_str()) {
                Ok(()) => 0,
                Err(errno) => i64::from(errno.raw_os_error()),
            }
        }
        Err(errno) => i64::from(errno.raw_os_error()),
    };
    Ok(answer(interp, code))
}

/// `SysMkDir(name [, mode])`: `mkdir(2)` and its raw errno, the mode
/// defaulting to `0777` before the umask. The mode is **not** range-checked --
/// measured, `512` and `-1` are both accepted where the documented range ends
/// at `511`.
pub(crate) fn mk_dir(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let path = path_argument(interp, args, "1", 2)?;
    let mode = match args.get(1).copied().flatten() {
        None => 0o777,
        Some(value) => {
            let text = interp.to_text(value).into_owned();
            let Some(bits) = whole_number(&text) else {
                return Err(Raised::native_argument_not_a_number("mode", &text).into());
            };
            #[allow(clippy::cast_sign_loss, reason = "the oracle passes the bits through")]
            {
                (bits as u32) & 0o7777
            }
        }
    };
    let code = match rustix::fs::mkdir(path.as_str(), rustix::fs::Mode::from_bits_truncate(mode)) {
        Ok(()) => 0,
        Err(errno) => i64::from(errno.raw_os_error()),
    };
    Ok(answer(interp, code))
}

/// The upper bound `SysSleep` enforces, on every platform -- the C++ holds
/// this one number and the Unix documentation's `999999999` corresponds to
/// nothing it checks.
const SLEEP_MAXIMUM: f64 = 2_147_483.0;

/// `SysSleep(seconds)`: parks the running activity until the delay has
/// passed, fractional seconds accepted.
pub(crate) fn sleep(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 1 {
        return Err(Raised::too_many_internal_arguments(1).into());
    }
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_internal_argument("1").into());
    };
    let text = interp.to_text(value).into_owned();
    let Ok(seconds) = String::from_utf8_lossy(&text).trim().parse::<f64>() else {
        return Err(Raised::native_argument_not_a_number("delay", &text).into());
    };
    if !(0.0..=SLEEP_MAXIMUM).contains(&seconds) {
        return Err(
            Raised::native_argument_out_of_range("delay", 0, SLEEP_MAXIMUM as i64, &text).into(),
        );
    }
    park_point!(interp, crate::pinning::ParkKind::SysSleep);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs_f64(seconds);
    let done = answer(interp, 0);
    interp.park_routine(crate::scheduler::ParkReason::Sleep { deadline }, done)
}

/// `ERROR_SEM_TIMEOUT`: a timed wait ended with nothing taken.
const SEM_TIMEOUT: i64 = 121;

/// `ERROR_INVALID_HANDLE`: a handle no create made, or one closed since.
const INVALID_HANDLE: i64 = 6;

/// `SysCreateEventSem([name] [, reset])`: an unnamed semaphore of value zero,
/// by its handle. The reset argument means nothing on Unix.
pub(crate) fn create_event_sem(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 2 {
        return Err(Raised::too_many_internal_arguments(2).into());
    }
    if let Some(Some(value)) = args.first().copied() {
        return refuse_name(interp, name, value);
    }
    let handle = interp.create_counting(0);
    Ok(handle_answer(interp, handle))
}

/// `SysCreateMutexSem([name])`: an unnamed semaphore of value one, by its
/// handle. The oracle's unnamed form is an empty name; with no argument at
/// all it crashes (`corpus/oracle-crashes.txt` entry 27), and here it is
/// unnamed too.
pub(crate) fn create_mutex_sem(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 1 {
        return Err(Raised::too_many_internal_arguments(1).into());
    }
    if let Some(Some(value)) = args.first().copied()
        && !interp.to_text(value).is_empty()
    {
        return refuse_name(interp, name, value);
    }
    let handle = interp.create_counting(1);
    Ok(handle_answer(interp, handle))
}

/// A create given the name `value`: the empty string where `sem_open` rejects
/// the name and creates nothing (glibc: nothing left once leading slashes go,
/// or a slash after them, `EINVAL`), else the named-semaphore refusal.
fn refuse_name(interp: &mut Interp, name: &[u8], value: ObjRef) -> Result<ObjRef, Failure> {
    let text = interp.to_text(value);
    let rest = &text[text.iter().take_while(|&&byte| byte == b'/').count()..];
    if rest.is_empty() || rest.contains(&b'/') {
        return Ok(interp.text_built(Vec::new()));
    }
    Err(Loud::named_semaphore(name).into())
}

/// `SysPostEventSem(handle)`: `sem_post`.
pub(crate) fn post_event_sem(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let handle = only_handle(interp, args)?;
    let code = if interp.post_counting(handle) {
        0
    } else {
        INVALID_HANDLE
    };
    Ok(answer(interp, code))
}

/// `SysResetEventSem(handle)`: the value back to zero, answering `0` whatever
/// the handle.
pub(crate) fn reset_event_sem(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let handle = only_handle(interp, args)?;
    interp.reset_counting(handle);
    Ok(answer(interp, 0))
}

/// `SysReleaseMutexSem(handle)`: a post only where the value is zero, so it
/// never exceeds one.
pub(crate) fn release_mutex_sem(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let handle = only_handle(interp, args)?;
    let code = match interp.counting_value(handle) {
        None => INVALID_HANDLE,
        Some(0) => {
            interp.post_counting(handle);
            0
        }
        Some(_) => 0,
    };
    Ok(answer(interp, code))
}

/// `SysCloseEventSem(handle)` and `SysCloseMutexSem(handle)`, one body.
pub(crate) fn close_sem(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let handle = only_handle(interp, args)?;
    let code = if interp.close_counting(handle) {
        0
    } else {
        INVALID_HANDLE
    };
    Ok(answer(interp, code))
}

/// `SysWaitEventSem(handle [, timeout])` and `SysRequestMutexSem(handle [,
/// timeout])`, one body: a timeout of zero waits for ever, one below zero
/// answers `0` at once and takes nothing, and one above zero tries now and
/// every `SEM_POLL` while the timeout lasts, answering `121` one `SEM_POLL`
/// after its last try where nothing was taken.
pub(crate) fn wait_sem(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 2 {
        return Err(Raised::too_many_internal_arguments(2).into());
    }
    let handle = handle_argument(interp, args)?;
    let timeout = match args.get(1).copied().flatten() {
        None => 0,
        Some(value) => int_argument(interp, value, 2)?,
    };
    if timeout < 0 {
        return Ok(answer(interp, 0));
    }
    match interp.take_counting(handle) {
        None => return Ok(answer(interp, INVALID_HANDLE)),
        Some(true) => return Ok(answer(interp, 0)),
        Some(false) => {}
    }
    let (kind, deadline) = if timeout > 0 {
        let polls = u32::try_from(timeout.unsigned_abs().div_ceil(100)).unwrap_or(u32::MAX);
        (WaitKind::Poll(polls - 1), Some(Instant::now() + SEM_POLL))
    } else {
        (WaitKind::Counting, None)
    };
    park_point!(interp, crate::pinning::ParkKind::SysSemWait);
    let wait = SemaphoreWait {
        key: SemaphoreKey::Handle(handle),
        kind,
        deadline,
    };
    let receiver = handle_answer(interp, handle);
    interp.park_routine_with(ParkReason::Semaphore(wait), sem_woken, receiver)
}

fn sem_woken(interp: &mut Interp, _handle: ObjRef) -> Result<Option<ObjRef>, Failure> {
    let taken = interp.take_semaphore_answer();
    Ok(Some(answer(interp, if taken { 0 } else { SEM_TIMEOUT })))
}

/// A semaphore handle, as text.
fn handle_answer(interp: &mut Interp, handle: u64) -> ObjRef {
    answer(interp, i64::try_from(handle).expect("handles fit i64"))
}

/// The one handle argument of a routine that takes nothing else.
fn only_handle(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<u64, Failure> {
    if args.len() > 1 {
        return Err(Raised::too_many_internal_arguments(1).into());
    }
    handle_argument(interp, args)
}

/// The first argument as the oracle's `uintptr_t`: 88.901 where it is
/// missing, 88.907 where it is not a whole number from zero to `2^64 - 1`.
fn handle_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<u64, Failure> {
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_internal_argument("1").into());
    };
    let handle = match value.decode() {
        Decoded::SmallInt(number) => u64::try_from(number).ok(),
        _ => Number::parse_bytes(&interp.to_text(value))
            .and_then(|number| number.unsigned_int64_value(DIGITS64)),
    };
    handle.ok_or_else(|| {
        let found = interp.string_value_text(value);
        outside_range(1, 0, u64::MAX.into(), &found)
    })
}

/// `value`, the argument at `position`, as the oracle's `int`.
fn int_argument(interp: &mut Interp, value: ObjRef, position: usize) -> Result<i64, Failure> {
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => Number::parse_bytes(&interp.to_text(value))
            .and_then(|number| number.int64_value(DIGITS64)),
    };
    let (min, max) = (i64::from(i32::MIN), i64::from(i32::MAX));
    number
        .filter(|number| (min..=max).contains(number))
        .ok_or_else(|| {
            let found = interp.string_value_text(value);
            outside_range(position, min.into(), max.into(), &found)
        })
}

/// 88.907 for an argument of a routine of this package.
fn outside_range(position: usize, min: i128, max: i128, found: &[u8]) -> Failure {
    let mut raised = Raised::native_argument_outside_range(position, min, max, found);
    raised.delivery.internal_package = true;
    raised.into()
}

/// `SysVersion()` and `SysLinVer()`, which are one entry point: `uname`'s
/// `sysname` and `release`, a space between them.
pub(crate) fn version(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if !args.is_empty() {
        return Err(Raised::too_many_internal_arguments(0).into());
    }
    let info = rustix::system::uname();
    let mut text = info.sysname().to_bytes().to_vec();
    text.push(b' ');
    text.extend_from_slice(info.release().to_bytes());
    Ok(interp.text_built(text))
}

/// A whole number's value, or `None` for text that is not one.
fn whole_number(text: &[u8]) -> Option<i64> {
    String::from_utf8_lossy(text).trim().parse::<i64>().ok()
}

/// What one `SysFileTree` call is looking for and how it reports what it
/// finds, from the option letters.
#[derive(Clone, Copy)]
struct TreeOptions {
    files: bool,
    directories: bool,
    recurse: bool,
    /// `O`: the qualified name alone, with no date, size or attributes.
    names_only: bool,
    /// `I`: fold case when matching the pattern.
    caseless: bool,
    stamp: Stamp,
}

/// Which of the three date layouts the entry line carries.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stamp {
    /// `M/D/YY   H:MMa`.
    Short,
    /// `L`: `YYYY-MM-DD HH:MM:SS`.
    Long,
    /// `T`: `YY/MM/DD/HH/MM`.
    Combined,
}

impl TreeOptions {
    /// The letters, in any order and any case. `B` is the default and means
    /// both kinds; naming neither `F` nor `D` means the same.
    fn parse(text: &[u8]) -> TreeOptions {
        let mut options = TreeOptions {
            files: false,
            directories: false,
            recurse: false,
            names_only: false,
            caseless: false,
            stamp: Stamp::Short,
        };
        for letter in text {
            match letter.to_ascii_uppercase() {
                b'F' => options.files = true,
                b'D' => options.directories = true,
                b'B' => {
                    options.files = true;
                    options.directories = true;
                }
                b'S' => options.recurse = true,
                b'O' => options.names_only = true,
                b'I' => options.caseless = true,
                b'L' => options.stamp = Stamp::Long,
                b'T' => options.stamp = Stamp::Combined,
                _ => {}
            }
        }
        if !options.files && !options.directories {
            options.files = true;
            options.directories = true;
        }
        options
    }
}

/// `fnmatch(3)` over one name, which is what the oracle matches with: `*`,
/// `?`, and a bracket expression with `-` ranges and a leading `!` negation.
///
/// `*` does not cross a `/`, and neither does anything else here, because the
/// caller only ever offers one path component.
fn matches(pattern: &[u8], name: &[u8], caseless: bool) -> bool {
    let fold = |byte: u8| {
        if caseless {
            byte.to_ascii_uppercase()
        } else {
            byte
        }
    };
    // The classic backtracking walk: `star` remembers where the last `*` was
    // so a failed tail can resume one byte later.
    let (mut pattern_at, mut name_at) = (0, 0);
    let (mut star, mut resume) = (None, 0);
    while name_at < name.len() {
        match pattern.get(pattern_at) {
            Some(b'*') => {
                star = Some(pattern_at);
                resume = name_at;
                pattern_at += 1;
            }
            Some(b'?') => {
                pattern_at += 1;
                name_at += 1;
            }
            Some(b'[') => match bracket(pattern, pattern_at, name[name_at], caseless) {
                Some(next) => {
                    pattern_at = next;
                    name_at += 1;
                }
                None => match star {
                    Some(at) => {
                        pattern_at = at + 1;
                        resume += 1;
                        name_at = resume;
                    }
                    None => return false,
                },
            },
            Some(byte) if fold(*byte) == fold(name[name_at]) => {
                pattern_at += 1;
                name_at += 1;
            }
            _ => match star {
                Some(at) => {
                    pattern_at = at + 1;
                    resume += 1;
                    name_at = resume;
                }
                None => return false,
            },
        }
    }
    pattern[pattern_at..].iter().all(|byte| *byte == b'*')
}

/// One bracket expression against one byte: the index just past the `]` when
/// it matches, `None` when it does not.
fn bracket(pattern: &[u8], open: usize, byte: u8, caseless: bool) -> Option<usize> {
    let fold = |value: u8| {
        if caseless {
            value.to_ascii_uppercase()
        } else {
            value
        }
    };
    let mut at = open + 1;
    let negated = matches!(pattern.get(at), Some(b'!' | b'^'));
    if negated {
        at += 1;
    }
    let mut hit = false;
    let mut first = true;
    while at < pattern.len() {
        // A `]` in the first position is the literal character, not the close.
        if pattern[at] == b']' && !first {
            let matched = hit != negated;
            return matched.then_some(at + 1);
        }
        first = false;
        let low = pattern[at];
        if pattern.get(at + 1) == Some(&b'-') && pattern.get(at + 2).is_some_and(|end| *end != b']')
        {
            let high = pattern[at + 2];
            if (fold(low)..=fold(high)).contains(&fold(byte)) {
                hit = true;
            }
            at += 3;
        } else {
            if fold(low) == fold(byte) {
                hit = true;
            }
            at += 1;
        }
    }
    // An unterminated bracket is a literal `[`, which only matches one.
    (byte == b'[').then_some(open + 1)
}

/// The ten-character mode string a directory listing shows.
fn attributes(meta: &std::fs::Metadata) -> Vec<u8> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = meta.permissions().mode();
    let mut out = Vec::with_capacity(10);
    out.push(if meta.is_dir() {
        b'd'
    } else if meta.file_type().is_symlink() {
        b'l'
    } else {
        b'-'
    });
    for shift in [6, 3, 0] {
        let bits = (mode >> shift) & 0o7;
        out.push(if bits & 0o4 == 0 { b'-' } else { b'r' });
        out.push(if bits & 0o2 == 0 { b'-' } else { b'w' });
        out.push(if bits & 0o1 == 0 { b'-' } else { b'x' });
    }
    out
}

/// One entry's line: the qualified name alone under `O`, and otherwise the
/// date, the size right-aligned, the attributes and the name.
fn entry_line(options: TreeOptions, path: &str, meta: &std::fs::Metadata) -> Vec<u8> {
    if options.names_only {
        return path.as_bytes().to_vec();
    }
    let base = meta
        .modified()
        .map(local_base_time)
        .unwrap_or(crate::builtin::datetime::UNIX_BASE_TIME);
    let when = crate::builtin::datetime::calendar_of(base);
    let mut out = Vec::new();
    match options.stamp {
        Stamp::Short => {
            let hour12 = match when.hours % 12 {
                0 => 12,
                other => other,
            };
            let meridiem = if when.hours < 12 { 'a' } else { 'p' };
            out.extend_from_slice(
                format!(
                    "{:>2}/{:02}/{:02}  {:>2}:{:02}{meridiem}",
                    when.month,
                    when.day,
                    when.year % 100,
                    hour12,
                    when.minutes
                )
                .as_bytes(),
            );
        }
        Stamp::Long => out.extend_from_slice(
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                when.year, when.month, when.day, when.hours, when.minutes, when.seconds
            )
            .as_bytes(),
        ),
        Stamp::Combined => out.extend_from_slice(
            format!(
                "{:02}/{:02}/{:02}/{:02}/{:02}",
                when.year % 100,
                when.month,
                when.day,
                when.hours,
                when.minutes
            )
            .as_bytes(),
        ),
    }
    out.extend_from_slice(format!("{:>12}  ", meta.len()).as_bytes());
    out.extend_from_slice(&attributes(meta));
    out.extend_from_slice(b"  ");
    out.extend_from_slice(path.as_bytes());
    out
}

/// A `SystemTime` as local civil microseconds since year 1.
fn local_base_time(time: std::time::SystemTime) -> i64 {
    let unix_micros = match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(since) => i64::try_from(since.as_micros()).unwrap_or(i64::MAX),
        Err(before) => -i64::try_from(before.duration().as_micros()).unwrap_or(i64::MAX),
    };
    let utc = crate::builtin::datetime::UNIX_BASE_TIME + unix_micros;
    utc + crate::builtin::datetime::local_offset_micros(utc)
}

/// `SysFileTree(filespec, stem [, options])`: the names matching `filespec`,
/// one stem tail each and the count in tail `0`.
///
/// **The directory part of the spec is literal.** Measured, `su*/deep.txt`,
/// `*/deep.txt` and `su?/deep.txt` all match nothing where `sub/deep.txt`
/// matches, so the spec splits at its last `/` into a directory to walk and
/// one pattern for the final name. A trailing `/` means every name under it.
///
/// The two attribute-mask arguments are accepted and not applied.
pub(crate) fn file_tree(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 5 {
        return Err(Raised::too_many_internal_arguments(5).into());
    }
    let Some(Some(spec)) = args.first().copied() else {
        return Err(Raised::missing_internal_argument("1").into());
    };
    let Some(Some(stem)) = args.get(1).copied() else {
        return Err(Raised::missing_internal_argument("2").into());
    };
    let spec = interp.to_text(spec).into_owned();
    let stem = interp.to_text(stem).into_owned();
    let options = match args.get(2).copied().flatten() {
        None => TreeOptions::parse(b""),
        Some(value) => {
            let letters = interp.to_text(value).into_owned();
            TreeOptions::parse(&letters)
        }
    };

    let spelled = String::from_utf8_lossy(&spec).into_owned();
    let cwd = interp.cwd_text();
    let qualified = crate::paths::normalize(&spelled, &cwd);
    let (directory, pattern) = match spelled.ends_with('/') {
        true => (qualified, "*".to_string()),
        false => match qualified.rsplit_once('/') {
            Some((head, tail)) => (head.to_string(), tail.to_string()),
            None => (cwd, qualified),
        },
    };
    let directory = if directory.is_empty() {
        "/".to_string()
    } else {
        directory
    };

    let mut found = Vec::new();
    collect(&directory, pattern.as_bytes(), options, &mut found);

    let mut prefix = String::from_utf8_lossy(&stem).to_ascii_uppercase();
    if !prefix.ends_with('.') {
        prefix.push('.');
    }
    for (index, line) in found.iter().enumerate() {
        let value = interp.text_built(line.clone());
        interp.assign_by_name(format!("{prefix}{}", index + 1).as_bytes(), value);
    }
    let count = interp.text_built(found.len().to_string().into_bytes());
    interp.assign_by_name(format!("{prefix}0").as_bytes(), count);
    Ok(answer(interp, 0))
}

/// One directory level, and then its subdirectories when `S` is set --
/// measured, a recursive run reports a level's own matches before descending.
fn collect(directory: &str, pattern: &[u8], options: TreeOptions, out: &mut Vec<Vec<u8>>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut deeper = Vec::new();
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let path = entry.path();
        let path = path.to_string_lossy();
        if meta.is_dir() {
            deeper.push(path.clone().into_owned());
        }
        let wanted = if meta.is_dir() {
            options.directories
        } else {
            options.files
        };
        if wanted
            && matches(
                pattern,
                entry.file_name().as_encoded_bytes(),
                options.caseless,
            )
        {
            out.push(entry_line(options, &path, &meta));
        }
    }
    if options.recurse {
        for sub in deeper {
            collect(&sub, pattern, options, out);
        }
    }
}
