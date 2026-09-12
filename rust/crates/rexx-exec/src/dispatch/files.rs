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

//! `.File`'s `LIBRARY REXX` entry points.
//!
//! The class itself is Rexx, in `StreamClasses.orx`: it keeps `path` and
//! `qualifiedPath`, and every native method here is reached as
//! `self~someImpl(self~qualifiedPath)`. So each one is a function of that path
//! alone, and none of them touches the receiver.

use rexx_core::{BehaviourId, Body, ObjRef};

use super::Cleared;
use crate::builtin::datetime::{UNIX_BASE_TIME, local_offset_micros};
use crate::error::Raised;
use crate::{Failure, Interp};

/// What a timestamp getter answers for a file it cannot stat. The Rexx half
/// tests for this exact value and answers `.nil`, so it is a sentinel and not
/// a time.
const MISSING_TIMESTAMP: i64 = -999_999_999_999_999_999;

/// The qualified path an entry point was handed.
fn path_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<String, Failure> {
    let Some(value) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    Ok(String::from_utf8_lossy(&interp.to_text(value)).into_owned())
}

/// `1` or `0`, which is how every `.File` predicate answers.
fn flag(interp: &mut Interp, yes: bool) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(if yes { b"1" } else { b"0" })))
}

/// Whether the running user may reach `path` for `mode`.
///
/// **`stat` is not the question.** A file's mode bits do not say whether *this*
/// user may read it, so this asks `access(2)`, which answers for the caller's
/// own credentials -- measured at uid 1000, a mode-`000` file answers 0 for
/// both where its owner's bits alone would say nothing useful. As root every
/// one of these answers 1 instead, which is a property of the run and not of
/// the code.
fn reachable(path: &str, mode: rustix::fs::Access) -> bool {
    rustix::fs::access(path, mode).is_ok()
}

/// `file_exists`: whether anything is there, following symlinks.
///
/// Measured, a dangling symlink answers `0` here and to every other predicate,
/// which is what `metadata` (rather than `symlink_metadata`) gives.
pub(super) fn exists(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let there = std::fs::metadata(&path).is_ok();
    flag(interp, there)
}

/// `file_isFile`: whether `path` names a regular file.
pub(super) fn is_file(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let yes = std::fs::metadata(&path).is_ok_and(|meta| meta.is_file());
    flag(interp, yes)
}

/// `file_isDirectory`: whether `path` names a directory.
pub(super) fn is_directory(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let yes = std::fs::metadata(&path).is_ok_and(|meta| meta.is_dir());
    flag(interp, yes)
}

/// `file_isHidden`: a name that exists and begins with a dot.
///
/// **The qualified path is what carries the dot.** The Rexx half hands this the
/// qualified spelling, so the test is for `/.` in it -- measured, `.hidden`
/// given as a bare relative name is hidden, and it has no separator of its own
/// to test.
pub(super) fn is_hidden(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let yes = std::fs::metadata(&path).is_ok() && path.contains("/.");
    flag(interp, yes)
}

/// `file_can_read`: whether the running user may read `path`.
pub(super) fn can_read(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let yes = reachable(&path, rustix::fs::Access::READ_OK);
    flag(interp, yes)
}

/// `file_can_write`: whether the running user may write `path`.
pub(super) fn can_write(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let yes = reachable(&path, rustix::fs::Access::WRITE_OK);
    flag(interp, yes)
}

/// `file_length`: the size `stat` reports, and `0` for anything it cannot
/// stat.
///
/// **A directory answers its own size, not `0`** -- measured, 40 and 60 for
/// two of them -- which is where this parts from `Stream`'s `query_size`, whose
/// own doc records that a directory answers `0` there. Reading permission is
/// not needed either: a mode-`000` file still answers 2.
pub(super) fn length(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let size = std::fs::metadata(&path).map_or(0, |meta| meta.len());
    Ok(Some(interp.text_built(size.to_string().into_bytes())))
}

/// `file_list`: the names directly inside `path`, or `.nil` when it is not a
/// directory.
///
/// **Raw `readdir` order, never sorted** -- measured, the oracle answers
/// `.dot`, `two.txt`, `one.txt`, `nested` for a directory whose `os.listdir`
/// gives exactly that. Dot files and subdirectories are in; `.` and `..` are
/// not, which `read_dir` already handles.
pub(super) fn list(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let Ok(entries) = std::fs::read_dir(&path) else {
        return Ok(Some(ObjRef::NIL));
    };
    let mut slots: Vec<Option<ObjRef>> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().into_encoded_bytes();
        let held = interp.text_built(name);
        // Rooted before the next allocation: the array below allocates, and
        // the names built so far are reachable from nothing until it exists.
        interp.roots.push_temp(held);
        slots.push(Some(held));
    }
    Ok(Some(
        interp.alloc_with(BehaviourId::ARRAY, Body::array(slots)),
    ))
}

/// One file timestamp as the Rexx half wants it: microseconds of **local**
/// civil time since year 1, or [`MISSING_TIMESTAMP`].
fn stamp(interp: &mut Interp, path: &str, accessed: bool) -> Result<Option<ObjRef>, Failure> {
    let read = std::fs::metadata(path).and_then(|meta| {
        if accessed {
            meta.accessed()
        } else {
            meta.modified()
        }
    });
    let answer = match read {
        Ok(time) => local_base_time(time),
        Err(_) => MISSING_TIMESTAMP,
    };
    Ok(Some(interp.text_built(answer.to_string().into_bytes())))
}

/// A `SystemTime` as local civil microseconds since year 1.
///
/// **At the offset in force at that instant**, not the one in force now:
/// measured, a March stamp and a November stamp both encode `+3600` here, and
/// a summer reading of the same file would be an hour out if today's offset
/// were used.
fn local_base_time(time: std::time::SystemTime) -> i64 {
    let unix_micros = match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(since) => i64::try_from(since.as_micros()).unwrap_or(i64::MAX),
        Err(before) => -i64::try_from(before.duration().as_micros()).unwrap_or(i64::MAX),
    };
    let utc = UNIX_BASE_TIME + unix_micros;
    utc + local_offset_micros(utc)
}

/// The `SystemTime` a local civil basetime names -- [`local_base_time`]
/// inverted.
///
/// **One refinement, because the offset depends on the instant it is solving
/// for.** The first guess reads the zone at the local reading itself, which is
/// off by the offset; re-reading it at the resulting instant settles every
/// case except the hour a DST jump repeats, where either reading is a real
/// local time.
fn system_time_of(base_time: i64) -> std::time::SystemTime {
    let mut utc = base_time - local_offset_micros(base_time);
    utc = base_time - local_offset_micros(utc);
    let unix_micros = utc - UNIX_BASE_TIME;
    let magnitude = std::time::Duration::from_micros(unix_micros.unsigned_abs());
    if unix_micros >= 0 {
        std::time::UNIX_EPOCH + magnitude
    } else {
        std::time::UNIX_EPOCH - magnitude
    }
}

/// `file_get_last_modified`.
pub(super) fn get_last_modified(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    stamp(interp, &path, false)
}

/// `file_get_last_accessed`.
pub(super) fn get_last_accessed(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    stamp(interp, &path, true)
}

/// The basetime a timestamp setter was handed as its second argument.
fn base_time_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<i64, Failure> {
    let Some(value) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let text = interp.to_text(value).into_owned();
    Ok(String::from_utf8_lossy(&text).trim().parse().unwrap_or(0))
}

/// `file_set_last_modified`: the modification stamp, leaving the access stamp
/// alone -- measured, the oracle keeps the other one in both directions, which
/// is why this is `set_file_mtime` and not `set_file_times`.
pub(super) fn set_last_modified(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let when = base_time_argument(interp, args)?;
    let stamp = filetime::FileTime::from_system_time(system_time_of(when));
    let done = filetime::set_file_mtime(&path, stamp).is_ok();
    flag(interp, done)
}

/// `file_set_last_accessed`: the access stamp, leaving the modification stamp
/// alone.
pub(super) fn set_last_accessed(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let when = base_time_argument(interp, args)?;
    let stamp = filetime::FileTime::from_system_time(system_time_of(when));
    let done = filetime::set_file_atime(&path, stamp).is_ok();
    flag(interp, done)
}

/// `file_make_dir`: the one directory, never its parents.
///
/// Measured: `0` when a parent is missing and `0` over a name that already
/// exists, which is what makes the orx `makeDirs` recurse.
pub(super) fn make_dir(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let made = std::fs::create_dir(&path).is_ok();
    flag(interp, made)
}

/// `file_delete_file`.
///
/// **The write check is on the file, not on its directory**, which is what the
/// oracle's own `SysFileSystem::deleteFile` asks -- unlike `unlink(2)`, whose
/// permission lives on the parent.
pub(super) fn delete_file(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    if !reachable(&path, rustix::fs::Access::WRITE_OK) {
        return flag(interp, false);
    }
    let gone = std::fs::remove_file(&path).is_ok();
    flag(interp, gone)
}

/// `file_delete_directory`: the directory itself, never its contents --
/// measured, a non-empty one answers `0`.
pub(super) fn delete_directory(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let gone = std::fs::remove_dir(&path).is_ok();
    flag(interp, gone)
}

/// `file_rename`: refuses a same-path rename and an existing target before it
/// asks the file system, because `rename(2)` would answer success for the
/// first and replace the second -- measured, the oracle answers `0` to both.
pub(super) fn rename(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let from = path_argument(interp, args)?;
    let Some(value) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let to = String::from_utf8_lossy(&interp.to_text(value)).into_owned();
    if from == to || std::fs::symlink_metadata(&to).is_ok() {
        return flag(interp, false);
    }
    let moved = std::fs::rename(&from, &to).is_ok();
    flag(interp, moved)
}

/// Rewrites `path`'s three write bits, for `file_set_read_only` and
/// `file_set_writable`. Answers **nothing**: measured, `say f~setReadOnly` is
/// 91.999 and the side effect happens anyway.
fn write_bits(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    writable: bool,
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    if let Ok(meta) = std::fs::metadata(&path) {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = meta.permissions().mode();
        // All three, not the owner's alone -- measured, `644` becomes `444`
        // and then `666`, so the setter ignores the umask.
        let rewritten = if writable {
            mode | 0o222
        } else {
            mode & !0o222
        };
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(rewritten));
    }
    let _ = interp;
    Ok(None)
}

/// `file_set_read_only`.
pub(super) fn set_read_only(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    write_bits(interp, args, false)
}

/// `file_set_writable`.
pub(super) fn set_writable(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    write_bits(interp, args, true)
}

/// `file_case_sensitive`: whether this platform's names are, which on the one
/// this crate targets they are.
pub(super) fn case_sensitive(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    flag(interp, true)
}

/// `this_file_case_sensitive`: the same question of one path, which answers
/// the same here -- a per-file answer would need the mount's own flag, and
/// every mount on this platform agrees.
pub(super) fn this_file_case_sensitive(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    flag(interp, true)
}

/// `file_list_roots`: the one root this platform has, as a string the Rexx
/// half does **not** wrap -- measured, `listRoots[1]~class~id` is `String`
/// where `temporaryPath` answers a `File`.
pub(super) fn list_roots(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let root = interp.text(b"/");
    interp.roots.push_temp(root);
    Ok(Some(interp.alloc_with(
        BehaviourId::ARRAY,
        Body::array(vec![Some(root)]),
    )))
}

/// `file_temporary_path`: a string the orx wraps in a `File`.
///
/// Read from the interpreter's own environment rather than the process's, so a
/// `SETLOCAL`'d `TMPDIR` is what a program sees.
pub(super) fn temporary_path(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let named = interp
        .env_get(b"TMPDIR")
        .filter(|value| !value.is_empty())
        .map(<[u8]>::to_vec);
    Ok(Some(match named {
        Some(value) => interp.text_built(value),
        None => interp.text(b"/tmp"),
    }))
}

/// `file_search_path_impl`: the first directory of `pathList` holding `file`,
/// joined, or `.nil`.
pub(super) fn search_path_impl(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let wanted = path_argument(interp, args)?;
    let Some(value) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let list = String::from_utf8_lossy(&interp.to_text(value)).into_owned();
    for directory in list.split(':').filter(|part| !part.is_empty()) {
        let candidate = format!("{}/{wanted}", directory.trim_end_matches('/'));
        if std::fs::metadata(&candidate).is_ok_and(|meta| meta.is_file()) {
            return Ok(Some(interp.text_built(candidate.into_bytes())));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// `file_qualify`: the absolute spelling of `path`, against the interpreter's
/// own current directory.
///
/// **Purely lexical, and that is the oracle's own choice** -- measured,
/// `a/b/../c.txt` answers `<cwd>/a/c.txt` although no such directory exists,
/// so nothing here resolves a symlink or asks the file system anything.
///
/// This is the entry point the rest of the class waits on: `init` qualifies
/// eagerly, so without it `.File~new` builds nothing at all.
pub(super) fn qualify(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = path_argument(interp, args)?;
    let cwd = interp.cwd_text();
    let qualified = crate::paths::normalize(&path, &cwd);
    Ok(Some(interp.text_built(qualified.into_bytes())))
}
