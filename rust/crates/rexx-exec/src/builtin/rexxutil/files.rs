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

//! The `REXXUTIL` routines over files and search paths:
//! `interpreter/runtime/RexxUtilCommon.cpp`, `platform/unix/SysRexxUtil.cpp`
//! and the `platform/unix/SysFileSystem.cpp` calls they make.

use chrono::{
    Datelike as _, LocalResult, NaiveDate, NaiveDateTime, TimeDelta, TimeZone as _, Timelike as _,
};
use rexx_core::ObjRef;
use rustix::fs::{AtFlags, FileType, Mode, OFlags, Timespec, Timestamps};
use rustix::io::Errno;

use super::{
    answer, c_string, check_arity, marshalled, optional_text, os_path, path_argument, required,
    required_c_string,
};
use crate::builtin::platform::qualify_name;
use crate::error::Raised;
use crate::{Failure, Interp};

/// `EEXIST`, which the copy and the move answer for a source and target that
/// are one file, and the move for a target that is a file already.
const EEXIST: i32 = 17;

/// The type of whatever `path` names, following a symbolic link.
fn file_type(path: &[u8]) -> Option<FileType> {
    let stat = rustix::fs::stat(path).ok()?;
    Some(FileType::from_raw_mode(stat.st_mode))
}

/// `SysFileSystem::fileExists`: whether `path` names a regular file.
fn is_regular(path: &[u8]) -> bool {
    file_type(path) == Some(FileType::RegularFile)
}

/// `samePaths`: whether both names exist and resolve to one path. Every
/// file system is taken to be case-sensitive.
fn same_paths(first: &[u8], second: &[u8]) -> bool {
    let Ok(first) = std::fs::canonicalize(os_path(first)) else {
        return false;
    };
    std::fs::canonicalize(os_path(second)).is_ok_and(|second| first == second)
}

/// A whole second, as `utime` sets it.
fn whole_second(seconds: i64) -> Timespec {
    Timespec {
        tv_sec: seconds,
        tv_nsec: 0,
    }
}

/// `copyFileDereferenceSymbolicLinks`: `from`'s data written over `to`, a
/// link at either end followed. Answers `0` or the errno of the step that
/// failed, and a target opened before a failed read keeps what was written.
fn copy_following_links(from: &[u8], to: &[u8], preserve_mode: bool) -> i32 {
    if same_paths(from, to) {
        return EEXIST;
    }
    let source_stat = match rustix::fs::stat(from) {
        Ok(stat) => stat,
        Err(errno) => return errno.raw_os_error(),
    };
    let source = match rustix::fs::open(from, OFlags::RDONLY, Mode::empty()) {
        Ok(fd) => fd,
        Err(errno) => return errno.raw_os_error(),
    };
    let created = rustix::fs::stat(to).is_err();
    let flags = OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC;
    let target = match rustix::fs::open(to, flags, Mode::from_raw_mode(0o666)) {
        Ok(fd) => fd,
        Err(errno) => return errno.raw_os_error(),
    };
    let mut buffer = [0u8; 4096];
    loop {
        let count = match rustix::io::read(&source, &mut buffer) {
            Ok(0) => break,
            Ok(count) => count,
            Err(errno) => return errno.raw_os_error(),
        };
        if let Err(errno) = rustix::io::write(&target, &buffer[..count]) {
            return errno.raw_os_error();
        }
    }
    drop(source);
    drop(target);
    let times = Timestamps {
        last_access: whole_second(source_stat.st_atime),
        last_modification: whole_second(source_stat.st_mtime),
    };
    let _ = rustix::fs::utimensat(rustix::fs::CWD, to, &times, AtFlags::empty());
    if created || preserve_mode {
        let _ = rustix::fs::chmod(to, Mode::from_raw_mode(source_stat.st_mode));
    }
    0
}

/// `copyFileDontDereferenceSymbolicLinks` as the move calls it, never
/// replacing a target: a link is copied as a link, and an existing target
/// where either end is a link answers `EEXIST`.
fn copy_keeping_links(from: &[u8], to: &[u8]) -> i32 {
    if same_paths(from, to) {
        return EEXIST;
    }
    let source = match rustix::fs::lstat(from) {
        Ok(stat) => FileType::from_raw_mode(stat.st_mode),
        Err(errno) => return errno.raw_os_error(),
    };
    let target = rustix::fs::lstat(to)
        .ok()
        .map(|stat| FileType::from_raw_mode(stat.st_mode));
    if let Some(target) = target
        && (source == FileType::Symlink || target == FileType::Symlink)
    {
        return EEXIST;
    }
    if source != FileType::Symlink {
        return copy_following_links(from, to, true);
    }
    let link = match rustix::fs::readlink(from, Vec::new()) {
        Ok(link) => link,
        Err(errno) => return errno.raw_os_error(),
    };
    match rustix::fs::symlink(link.as_c_str(), to) {
        Ok(()) => 0,
        Err(errno) => errno.raw_os_error(),
    }
}

/// `temporaryFilename`: `path` with six digits appended, naming no regular
/// file, or `None` where every one does.
fn spare_name(path: &[u8]) -> Option<Vec<u8>> {
    let start = random() % 1_000_000;
    let mut number = start;
    loop {
        let mut name = path.to_vec();
        name.extend_from_slice(format!("{number:06}").as_bytes());
        if !is_regular(&name) {
            return Some(name);
        }
        number = (number + 1) % 1_000_000;
        if number == start {
            return None;
        }
    }
}

/// A number from the process's random keys, for a name nobody predicts.
fn random() -> u64 {
    use std::hash::BuildHasher as _;
    std::hash::RandomState::new().hash_one(0u8)
}

/// `SysFileCopy(from, to)`: `SysFileSystem::copyFile`, which keeps the
/// source's times to the second and gives a target it creates the source's
/// mode. Answers `0` or an errno.
pub(crate) fn file_copy(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let from = path_argument(interp, args, 1)?;
    let to = path_argument(interp, args, 2)?;
    let code = copy_following_links(&from, &to, false);
    Ok(answer(interp, code.into()))
}

/// `SysFileMove(from, to)`: `SysFileSystem::moveFile`. A target that is a
/// regular file already, or the source itself, is `EEXIST`. Across file
/// systems it copies, keeping links, and then unlinks the source, and a
/// failed unlink there answers `-1` rather than an errno.
pub(crate) fn file_move(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let from = path_argument(interp, args, 1)?;
    let to = path_argument(interp, args, 2)?;
    let code = move_file(&from, &to);
    Ok(answer(interp, code.into()))
}

/// [`file_move`]'s body.
fn move_file(from: &[u8], to: &[u8]) -> i32 {
    if same_paths(from, to) || is_regular(to) {
        return EEXIST;
    }
    match rustix::fs::rename(from, to) {
        Ok(()) => return 0,
        Err(errno) if errno != Errno::XDEV => return errno.raw_os_error(),
        Err(_) => {}
    }
    // The source must be one this process can unlink before anything is
    // copied, which a rename in place shows.
    let Some(spare) = spare_name(from) else {
        return EEXIST;
    };
    if let Err(errno) = rustix::fs::rename(from, spare.as_slice()) {
        return errno.raw_os_error();
    }
    if let Err(errno) = rustix::fs::rename(spare.as_slice(), from) {
        return errno.raw_os_error();
    }
    let copied = copy_keeping_links(from, to);
    if copied != 0 {
        return copied;
    }
    match rustix::fs::unlink(from) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// `SysIsFileDirectory(name)`: whether it names a directory, a link to one
/// included.
pub(crate) fn is_file_directory(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let found = file_type(&path) == Some(FileType::Directory);
    Ok(answer(interp, i64::from(found)))
}

/// `SysIsFileLink(name)`: whether the name itself is a symbolic link. The
/// name is qualified first, so a trailing `/` is gone before `lstat` reads it.
pub(crate) fn is_file_link(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let path = path_argument(interp, args, 1)?;
    let found = rustix::fs::lstat(path.as_slice())
        .is_ok_and(|stat| FileType::from_raw_mode(stat.st_mode) == FileType::Symlink);
    Ok(answer(interp, i64::from(found)))
}

/// A local time's fields as `struct tm` holds them: the full year, the month
/// from zero, and the rest as read.
#[derive(Clone, Copy)]
struct CivilTime {
    year: i64,
    month0: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
}

/// `localtime`: the local reading of `seconds` since the epoch.
fn local_time(seconds: i64) -> Option<CivilTime> {
    let at = chrono::Local.timestamp_opt(seconds, 0).earliest()?;
    Some(CivilTime {
        year: i64::from(at.year()),
        month0: i64::from(at.month0()),
        day: i64::from(at.day()),
        hour: i64::from(at.hour()),
        minute: i64::from(at.minute()),
        second: i64::from(at.second()),
    })
}

/// `mktime` with `tm_isdst` of `-1`: fields past their range carry into the
/// next, a reading the zone repeats is its earlier instant, and a reading the
/// zone skips is taken at the offset in force before the skip. `-1` where the
/// result is outside what the calendar holds.
///
/// glibc takes a skipped reading at whichever offset lands it in daylight
/// saving time, which is the earlier offset wherever daylight saving is the
/// later one. In a zone whose daylight saving is the earlier offset, as
/// `Europe/Dublin`'s winter is, it lands an hour earlier than here: measured,
/// `2020-03-29 01:30:00` there is `1585441800` on the oracle and `1585445400`
/// here. chrono does not say which offset is daylight saving time.
fn make_time(fields: CivilTime) -> i64 {
    let months = fields.year * 12 + fields.month0;
    let first = i32::try_from(months.div_euclid(12)).ok().and_then(|year| {
        let month = u32::try_from(months.rem_euclid(12) + 1).ok()?;
        NaiveDate::from_ymd_opt(year, month, 1)?.and_hms_opt(0, 0, 0)
    });
    let seconds =
        (fields.day - 1) * 86_400 + fields.hour * 3_600 + fields.minute * 60 + fields.second;
    let Some(local) =
        first.and_then(|first| first.checked_add_signed(TimeDelta::try_seconds(seconds)?))
    else {
        return -1;
    };
    let candidates = match chrono::Local.from_local_datetime(&local) {
        LocalResult::Single(at) => [Some(at), None],
        LocalResult::Ambiguous(one, other) => [Some(one), Some(other)],
        LocalResult::None => [None, None],
    };
    // chrono answers both readings for the instant a transition happens at,
    // so each is kept only where the zone reads that instant back as `local`.
    candidates
        .into_iter()
        .flatten()
        .map(|at| at.timestamp())
        .filter(|seconds| {
            chrono::Local
                .timestamp_opt(*seconds, 0)
                .earliest()
                .is_some_and(|at| at.naive_local() == local)
        })
        .min()
        .unwrap_or_else(|| skipped_reading(local))
}

/// [`make_time`] for a reading inside a skipped hour.
fn skipped_reading(local: NaiveDateTime) -> i64 {
    let Some(before) = local.checked_sub_signed(TimeDelta::days(1)) else {
        return -1;
    };
    let offset = chrono::Local.offset_from_utc_datetime(&before);
    local.and_utc().timestamp() - i64::from(offset.local_minus_utc())
}

/// `sscanf` over `text` with the format `%<w>d<sep>%<w>d<sep>%<w>d`, the
/// widths in `widths`: the three numbers where all three convert. Each
/// number skips white space before it and then reads a sign and digits,
/// `width` bytes at most with the sign counted. A separator matches only
/// itself, with no white space before it, and whatever follows the third
/// number is ignored.
fn scan_triple(text: &[u8], separator: u8, widths: [usize; 3]) -> Option<[i64; 3]> {
    let mut at = 0;
    let mut out = [0; 3];
    for (index, width) in widths.into_iter().enumerate() {
        if index > 0 {
            if text.get(at) != Some(&separator) {
                return None;
            }
            at += 1;
        }
        while text
            .get(at)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'..=b'\r'))
        {
            at += 1;
        }
        let end = (at + width).min(text.len());
        let mut cursor = at;
        let negative = match text.get(cursor) {
            Some(b'-') if cursor < end => {
                cursor += 1;
                true
            }
            Some(b'+') if cursor < end => {
                cursor += 1;
                false
            }
            _ => false,
        };
        let digits_start = cursor;
        let mut value: i64 = 0;
        while cursor < end && text[cursor].is_ascii_digit() {
            value = value * 10 + i64::from(text[cursor] - b'0');
            cursor += 1;
        }
        if cursor == digits_start {
            return None;
        }
        out[index] = if negative { -value } else { value };
        at = cursor;
    }
    Some(out)
}

/// `SysGetFileDateTime(name [, selector])`: the local time of the last write,
/// or with a selector starting `A` the last access, as
/// `YYYY-MM-DD HH:MM:SS` with the year space-padded to four. `-1` where the
/// name does not resolve.
pub(crate) fn get_file_date_time(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let path = path_argument(interp, args, 1)?;
    let access = match optional_text(interp, args, 2)? {
        None => false,
        Some(text) => {
            let selector = c_string(&text);
            match selector.first().map(u8::to_ascii_uppercase) {
                Some(b'A') => true,
                Some(b'W') => false,
                _ => {
                    return Err(Raised::invalid_routine_option(
                        "SysGetFileDateTime",
                        "time selector",
                        "'A' or 'W'",
                        selector,
                    )
                    .into());
                }
            }
        }
    };
    let Ok(stat) = rustix::fs::stat(path.as_slice()) else {
        return Ok(answer(interp, -1));
    };
    let seconds = if access { stat.st_atime } else { stat.st_mtime };
    let Some(at) = local_time(seconds) else {
        return Ok(answer(interp, -1));
    };
    let text = format!(
        "{:4}-{:02}-{:02} {:02}:{:02}:{:02}",
        at.year,
        at.month0 + 1,
        at.day,
        at.hour,
        at.minute,
        at.second
    );
    Ok(interp.text_built(text.into_bytes()))
}

/// `SysSetFileDateTime(name [, date] [, time])`: the last-write time set to
/// the local `date` (`YYYY-MM-DD`) and `time` (`HH:MM:SS`), either one
/// omitted keeping that half of the current value, both omitted meaning now.
/// The access time stays, to the second. `0`, or `-1` where the name does not
/// resolve, a given half does not scan, or the update fails.
pub(crate) fn set_file_date_time(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let path = path_argument(interp, args, 1)?;
    let date = optional_text(interp, args, 2)?;
    let time = optional_text(interp, args, 3)?;
    let Ok(stat) = rustix::fs::stat(path.as_slice()) else {
        return Ok(answer(interp, -1));
    };
    let modified = if date.is_none() && time.is_none() {
        interp
            .wall_now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| {
                i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
            })
    } else {
        let Some(mut fields) = local_time(stat.st_mtime) else {
            return Ok(answer(interp, -1));
        };
        if let Some(date) = date {
            let Some([year, month, day]) = scan_triple(c_string(&date), b'-', [4, 2, 2]) else {
                return Ok(answer(interp, -1));
            };
            (fields.year, fields.month0, fields.day) = (year, month - 1, day);
        }
        if let Some(time) = time {
            let Some([hour, minute, second]) = scan_triple(c_string(&time), b':', [2, 2, 2]) else {
                return Ok(answer(interp, -1));
            };
            (fields.hour, fields.minute, fields.second) = (hour, minute, second);
        }
        make_time(fields)
    };
    let times = Timestamps {
        last_access: whole_second(stat.st_atime),
        last_modification: whole_second(modified),
    };
    let set = rustix::fs::utimensat(rustix::fs::CWD, path.as_slice(), &times, AtFlags::empty());
    Ok(answer(interp, if set.is_ok() { 0 } else { -1 }))
}

/// `SysTempFileName(template [, filler])`: `getUniqueFileName`. Each filler
/// character in the template becomes a digit, the first the lowest, from a
/// random start counting up until the qualified name names nothing. The null
/// string where the template has no filler or more than nine, or every name
/// is taken.
pub(crate) fn temp_file_name(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let template = required_c_string(interp, args, 1)?;
    let filler = match optional_text(interp, args, 2)? {
        None => b'?',
        Some(text) => match c_string(&text) {
            [filler] => *filler,
            other => {
                return Err(Raised::routine_argument_not_a_character(
                    "SysTempFileName",
                    "filler",
                    other,
                )
                .into());
            }
        },
    };
    let name = unique_name(interp, &template, filler);
    Ok(interp.text_built(name))
}

/// [`temp_file_name`]'s search.
fn unique_name(interp: &Interp, template: &[u8], filler: u8) -> Vec<u8> {
    let places = template.iter().filter(|byte| **byte == filler).count();
    if places == 0 || places > 9 {
        return Vec::new();
    }
    let limit = 10u64.pow(u32::try_from(places).expect("nine at most"));
    let start = random() % limit;
    let mut number = start;
    let mut candidate = template.to_vec();
    loop {
        let mut rest = number;
        for (at, byte) in template.iter().enumerate() {
            if *byte == filler {
                candidate[at] = b'0' + u8::try_from(rest % 10).expect("a digit");
                rest /= 10;
            }
        }
        let qualified = qualify_name(interp, &candidate);
        if rustix::fs::stat(qualified.as_slice()).is_err() {
            return qualified;
        }
        number = (number + 1) % limit;
        if number == start {
            return Vec::new();
        }
    }
}

/// `SysSearchPath(variable, file [, option])`: the qualified name of the first
/// regular file called `file` along the directories the environment variable
/// lists, the current directory first unless the option starts `N`. The null
/// string where it is not found.
pub(crate) fn search_path(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let variable = required_c_string(interp, args, 1)?;
    let file = required_c_string(interp, args, 2)?;
    let current_first = match optional_text(interp, args, 3)? {
        None => true,
        Some(text) => {
            let option = c_string(&text);
            match option.first().map(u8::to_ascii_uppercase) {
                Some(b'C') => true,
                Some(b'N') => false,
                _ => {
                    return Err(Raised::invalid_routine_option(
                        "SysSearchPath",
                        "option",
                        "'C' or 'N'",
                        option,
                    )
                    .into());
                }
            }
        }
    };
    let value = interp.env_get(&variable).unwrap_or_default().to_vec();
    let path = if current_first {
        let mut path = interp.cwd_bytes().to_vec();
        if !value.is_empty() {
            path.push(b':');
            path.extend_from_slice(&value);
        }
        path
    } else {
        value
    };
    let found = search(interp, &file, &path);
    Ok(interp.text_built(found))
}

/// `SysFileSystem::searchPath`. A name starting `/`, `./`, `../` or `~` is
/// only qualified. Otherwise each non-empty entry of `path` is tried in turn,
/// and the first that names anything ends the search: a regular file is the
/// answer, anything else is not found.
fn search(interp: &Interp, file: &[u8], path: &[u8]) -> Vec<u8> {
    let has_directory = file.first() == Some(&b'~')
        || file.first() == Some(&b'/')
        || file.starts_with(b"./")
        || file.starts_with(b"../");
    if has_directory {
        let qualified = qualify_name(interp, file);
        return if is_regular(&qualified) {
            qualified
        } else {
            Vec::new()
        };
    }
    for entry in path.split(|byte| *byte == b':') {
        if entry.is_empty() {
            continue;
        }
        let mut candidate = entry.to_vec();
        if candidate.last() != Some(&b'/') {
            candidate.push(b'/');
        }
        candidate.extend_from_slice(file);
        let qualified = qualify_name(interp, &candidate);
        if qualified.is_empty() {
            continue;
        }
        match file_type(&qualified) {
            Some(FileType::RegularFile) => return qualified,
            Some(_) => return Vec::new(),
            None => {}
        }
    }
    Vec::new()
}

/// Where `SysFileSearch` puts what it finds: `StemHandler`.
#[derive(Clone, Copy)]
enum Found {
    /// An `Array`, emptied first and appended to.
    Array(ObjRef),
    /// A stem, written from tail 1 with the count in tail 0 at the end. Tails
    /// past the count are left as they were.
    Stem(ObjRef),
}

impl Found {
    /// The third argument as `StemHandler::setStem` resolves it: an instance
    /// of `Array` itself, a stem, or a string naming a stem variable. 40.925
    /// for anything else.
    fn resolve(interp: &mut Interp, value: ObjRef) -> Result<Found, Failure> {
        if interp.array_body(value).is_some() {
            let caller = interp.caller();
            interp.send_message(value, b"EMPTY", None, &[], caller)?;
            return Ok(Found::Array(value));
        }
        if interp.is_stem_object(value) {
            return Ok(Found::Stem(value));
        }
        // `ResolveStemVariable` clears whatever the string conversion raised.
        if let Ok(Some(text)) = interp.string_conversion(value) {
            let name = interp.to_text(text).into_owned();
            if let Some(stem) = interp.context_stem_named(&name) {
                return Ok(Found::Stem(stem));
            }
        }
        Err(Raised::not_an_array_or_stem(3).into())
    }

    /// Adds `line` as the `count`th value.
    fn add(self, interp: &mut Interp, count: usize, line: Vec<u8>) -> Result<(), Failure> {
        let value = interp.text_built(line);
        match self {
            Found::Array(array) => {
                interp.roots.activity_mut().push_temp(value);
                interp.array_append(array, value)?;
            }
            Found::Stem(stem) => {
                interp.stem_object_write(stem, count.to_string().as_bytes(), value)
            }
        }
        Ok(())
    }

    /// `StemHandler::complete`: a stem's count, into tail 0.
    fn complete(self, interp: &mut Interp, count: usize) {
        if let Found::Stem(stem) = self {
            let count = answer(interp, i64::try_from(count).unwrap_or(i64::MAX));
            interp.stem_object_write(stem, b"0", count);
        }
    }
}

/// The lines `LineReader` reads: each ends at a LF, which is dropped with one
/// CR before it, and a last line with no LF ends at the end of the file.
fn lines(contents: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut rest = contents;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        match rest.iter().position(|byte| *byte == b'\n') {
            Some(end) => {
                let line = &rest[..end];
                rest = &rest[end + 1..];
                Some(line.strip_suffix(b"\r").unwrap_or(line))
            }
            None => Some(std::mem::take(&mut rest)),
        }
    })
}

/// `mystrstr`: whether `needle` occurs in `line`, ASCII case folded when
/// `caseless`. An empty needle occurs nowhere.
fn contains(line: &[u8], needle: &[u8], caseless: bool) -> bool {
    if needle.is_empty() || needle.len() > line.len() {
        return false;
    }
    line.windows(needle.len()).any(|window| {
        if caseless {
            window.eq_ignore_ascii_case(needle)
        } else {
            window == needle
        }
    })
}

/// The longest message `SysFileSearch` formats for a bad option: its
/// `snprintf` buffer less the NUL.
const OPTION_MESSAGE_LIMIT: usize = 255;

/// `SysFileSearch(needle, file, stem [, options])`: the lines of `file`
/// holding `needle`, into a stem or an array, each after its line number and
/// a blank under option `N`, matched without case unless option `C`. Answers
/// `0`, or `3` where the file is a directory or does not open, with nothing
/// found.
pub(crate) fn file_search(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 4)?;
    let needle = required(args, 1)?;
    let Some(needle) = interp.string_conversion(needle)? else {
        return Err(marshalled(Raised::argument_needs_a_string_value(1)));
    };
    let needle = interp.to_text(needle).into_owned();
    let file = required_c_string(interp, args, 2)?;
    let target = required(args, 3)?;
    let options = optional_text(interp, args, 4)?;

    let (mut numbered, mut caseless) = (false, true);
    if let Some(options) = &options {
        let options = c_string(options);
        for letter in options {
            match letter.to_ascii_uppercase() {
                b'N' => numbered = true,
                b'I' => caseless = true,
                b'C' => caseless = false,
                _ => {
                    let mut text = b"SysFileSearch options argument must be a combination of \
                                     C, I, or N; found \""
                        .to_vec();
                    text.extend_from_slice(options);
                    text.push(b'"');
                    text.truncate(OPTION_MESSAGE_LIMIT);
                    return Err(Raised::routine_user_defined(&text).into());
                }
            }
        }
    }

    let found = Found::resolve(interp, target)?;
    let path = qualify_name(interp, &file);
    let contents = match file_type(&path) {
        Some(FileType::Directory) => None,
        _ => std::fs::read(os_path(&path)).ok(),
    };
    let Some(contents) = contents else {
        found.complete(interp, 0);
        return Ok(interp.text(b"3"));
    };
    let mut count = 0;
    for (index, line) in lines(&contents).enumerate() {
        if !contains(line, &needle, caseless) {
            continue;
        }
        let mut value = Vec::with_capacity(line.len() + 8);
        if numbered {
            value.extend_from_slice(format!("{} ", index + 1).as_bytes());
        }
        value.extend_from_slice(line);
        count += 1;
        found.add(interp, count, value)?;
    }
    found.complete(interp, count);
    Ok(interp.text(b"0"))
}

#[cfg(test)]
mod tests;
