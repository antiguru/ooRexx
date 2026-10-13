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

//! The `REXXUTIL` package's file-system, sleep, semaphore, stem and version routines --
//! `interpreter/platform/unix/SysRexxUtil.cpp` and the shared
//! `RexxUtilCommon.cpp`.

pub(crate) mod files;
pub(crate) mod stems;
pub(crate) mod tree;

use rexx_core::{Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};

use crate::error::Raised;
use crate::scheduler::ParkReason;
use crate::semaphores::{SEM_POLL, SemaphoreKey, SemaphoreWait, WaitKind};
use crate::{Failure, Interp, Loud};

/// A raise the argument marshalling makes, which the oracle reports against
/// the package with no line.
fn marshalled(mut raised: Raised) -> Failure {
    raised.delivery.lineless = true;
    raised.delivery.internal_package = true;
    raised.into()
}

/// 88.922 where more than `arity` arguments were passed.
fn check_arity(args: &[Option<ObjRef>], arity: usize) -> Result<(), Failure> {
    if args.len() > arity {
        return Err(Raised::too_many_internal_arguments(arity).into());
    }
    Ok(())
}

/// The argument at `position`, 88.901 where it is omitted.
fn required(args: &[Option<ObjRef>], position: usize) -> Result<ObjRef, Failure> {
    args.get(position - 1)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_internal_argument(&position.to_string()).into())
}

/// An `OPTIONAL_CSTRING` argument's bytes, 88.909 where it has no string
/// value: `requiredString`, with no `STRING` fallback.
fn optional_text(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<Vec<u8>>, Failure> {
    let Some(value) = args.get(position - 1).copied().flatten() else {
        return Ok(None);
    };
    let Some(text) = interp.string_conversion(value)? else {
        return Err(marshalled(Raised::argument_needs_a_string_value(position)));
    };
    Ok(Some(interp.to_text(text).into_owned()))
}

/// A `CSTRING` argument as the routine reads it: the bytes before the first
/// NUL, which ends a C string.
fn c_string(text: &[u8]) -> &[u8] {
    match text.iter().position(|byte| *byte == 0) {
        Some(end) => &text[..end],
        None => text,
    }
}

/// [`optional_text`] for a `CSTRING` argument that is required, cut at its
/// first NUL by [`c_string`].
fn required_c_string(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Vec<u8>, Failure> {
    required(args, position)?;
    let text = optional_text(interp, args, position)?.expect("the argument is present");
    Ok(c_string(&text).to_vec())
}

/// A path argument, `RoutineQualifiedName`: the `CSTRING` at `position`
/// qualified by [`crate::builtin::platform::qualify_name`]. The null string
/// where the name cannot be qualified, which no file system call finds.
fn path_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Vec<u8>, Failure> {
    let text = required_c_string(interp, args, position)?;
    Ok(crate::builtin::platform::qualify_name(interp, &text))
}

/// A path's bytes as the `Path` the standard library opens, unchanged.
fn os_path(path: &[u8]) -> &std::path::Path {
    use std::os::unix::ffi::OsStrExt as _;
    std::path::Path::new(std::ffi::OsStr::from_bytes(path))
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
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let found = rustix::fs::stat(path.as_slice()).is_ok();
    Ok(answer(interp, i64::from(found)))
}

/// `SysIsFile(name)`: whether it is there **and** is a regular file.
pub(crate) fn is_file(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let found = std::fs::metadata(os_path(&path)).is_ok_and(|meta| meta.is_file());
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
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    const EACCES: i64 = 13;
    if rustix::fs::access(path.as_slice(), rustix::fs::Access::WRITE_OK).is_err() {
        return Ok(answer(interp, EACCES));
    }
    let code = match rustix::fs::unlink(path.as_slice()) {
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
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let code = match rustix::fs::unlink(path.as_slice()) {
        Ok(()) => 0,
        Err(errno) if errno == rustix::io::Errno::ISDIR || errno == rustix::io::Errno::PERM => {
            match rustix::fs::rmdir(path.as_slice()) {
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
    check_arity(args, 2)?;
    let path = path_argument(interp, args, 1)?;
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
    let code = match rustix::fs::mkdir(path.as_slice(), rustix::fs::Mode::from_bits_truncate(mode))
    {
        Ok(()) => 0,
        Err(errno) => i64::from(errno.raw_os_error()),
    };
    Ok(answer(interp, code))
}

/// The upper bound `SysSleep` enforces, on every platform -- the C++ holds
/// this one number and the Unix documentation's `999999999` corresponds to
/// nothing it checks.
const SLEEP_MAXIMUM: f64 = 2_147_483.0;

/// The shortest sleep: one microsecond, `TIME`'s resolution. The oracle's
/// sleep is a `nanosleep` call even for a zero delay
/// (`SysThread::longSleep`, `common/platform/unix/SysThread.cpp:191`), and
/// `TIME('E')` after it read 0 in none of 2000 passes for a delay of 0 or
/// 0.00000001.
const SLEEP_FLOOR: std::time::Duration = std::time::Duration::from_micros(1);

/// `SysSleep(seconds)`: parks the running activity until the delay, at least
/// [`SLEEP_FLOOR`], has passed, fractional seconds accepted.
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
    let deadline = interp.now() + std::time::Duration::from_secs_f64(seconds).max(SLEEP_FLOOR);
    let done = answer(interp, 0);
    interp.park_routine_with(
        crate::scheduler::ParkReason::Sleep { deadline },
        |interp, done| {
            interp.activity.asleep = None;
            if std::mem::take(&mut interp.activity.woken_by_halt) {
                return Ok(Some(answer(interp, EINTR)));
            }
            Ok(Some(done))
        },
        done,
    )
}

/// What a sleep a halt ended answers: `nanosleep`'s errno
/// (`SysThread::longSleep`, `common/platform/unix/SysThread.cpp:191`).
const EINTR: i64 = 4;

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
        (WaitKind::Poll(polls - 1), Some(interp.now() + SEM_POLL))
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
