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

//! The `Stream` entry points that answer without opening anything:
//! initialisation, the state and description words, the name queries, and
//! close, flush and uninit. The reading and writing halves are separate work;
//! everything here answers from the name, the file system's metadata, or the
//! state block itself.

use chrono::TimeZone;
use rexx_core::{Body, NativeState, ObjRef, StandardStream, StreamState, StreamStatus};

use super::Cleared;
use crate::error::Raised;
use crate::{Failure, Interp};

/// The state block, or 48.1 when `~uninit` has already dropped it. The oracle
/// segfaults for the unguarded half of its own entry points here
/// (`corpus/oracle-crashes.txt` entry 12b); answering 48.1 from every one of
/// them is this crate's licensed divergence.
fn require(interp: &Interp, receiver: ObjRef) -> Result<&StreamState, Failure> {
    match interp.stream(receiver) {
        Some(state) => Ok(state),
        None => Err(Raised::syntax(48, 1, vec![b"Stream not initialized".to_vec()]).into()),
    }
}

/// [`require`] for a caller that changes the state.
fn require_mut(interp: &mut Interp, receiver: ObjRef) -> Result<&mut StreamState, Failure> {
    match interp.stream_mut(receiver) {
        Some(state) => Ok(state),
        None => Err(Raised::syntax(48, 1, vec![b"Stream not initialized".to_vec()]).into()),
    }
}

/// The qualified path as a `String`, for the file-system calls below.
fn qualified_path(interp: &Interp, receiver: ObjRef) -> Result<String, Failure> {
    let state = require(interp, receiver)?;
    Ok(String::from_utf8_lossy(&state.qualified).into_owned())
}

/// `stream_init`: records the name as written and the path it resolves to.
/// The resolution happens **here**, so a later `DIRECTORY()` cannot move a
/// stream that has already been named.
pub(super) fn init(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let name = match args.first().copied().flatten() {
        Some(value) => interp.to_text(value).into_owned(),
        None => Vec::new(),
    };
    let qualified = if name.is_empty() {
        Vec::new()
    } else {
        let cwd = interp.cwd_text();
        crate::paths::normalize(&String::from_utf8_lossy(&name), &cwd).into_bytes()
    };
    let state = StreamState::new(name, qualified);
    let Some(object) = interp.heap.get_mut(receiver) else {
        unreachable!("a live receiver")
    };
    let Body::Instance { native, .. } = &mut object.body else {
        unreachable!("Stream is allocated as Body::Instance")
    };
    *native = Some(Box::new(NativeState::Stream(state)));
    Ok(None)
}

/// `std_set`: marks a stream named `STDIN`, `STDOUT` or `STDERR`, with or
/// without a trailing colon. The Rexx half decides which name qualifies and
/// only then sends this, so the marker is taken from the name it recorded.
pub(super) fn std_set(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = require_mut(interp, receiver)?;
    let mut upper = state.name.to_ascii_uppercase();
    if upper.last() == Some(&b':') {
        upper.pop();
    }
    state.standard = match upper.as_slice() {
        b"STDIN" => Some(StandardStream::In),
        b"STDOUT" => Some(StandardStream::Out),
        b"STDERR" => Some(StandardStream::Err),
        _ => None,
    };
    Ok(None)
}

/// `stream_state`: `UNKNOWN`, `READY`, `NOTREADY` or `ERROR`.
pub(super) fn state(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let word = require(interp, receiver)?.state_word().to_vec();
    Ok(Some(interp.text_built(word)))
}

/// `stream_description`: the state word, a colon, and what more the state
/// knows -- the errno and its text for an error, `EOF` for a stream that read
/// past the end, nothing for the other two.
pub(super) fn description(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = require(interp, receiver)?;
    let mut out = state.state_word().to_vec();
    out.push(b':');
    match state.status {
        StreamStatus::NotReady => out.extend_from_slice(b"EOF"),
        StreamStatus::Error(errno) => {
            out.extend_from_slice(errno.to_string().as_bytes());
            let text = std::io::Error::from_raw_os_error(errno).to_string();
            // `io::Error`'s rendering appends " (os error N)", which the
            // oracle's `strerror` does not.
            let text = match text.find(" (os error") {
                Some(at) => text[..at].to_string(),
                None => text,
            };
            if !text.is_empty() {
                out.push(b' ');
                out.extend_from_slice(text.as_bytes());
            }
        }
        StreamStatus::Unknown | StreamStatus::Ready => {}
    }
    Ok(Some(interp.text_built(out)))
}

/// `qualify`: the resolved name, whether or not anything is there.
pub(super) fn qualify(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let qualified = require(interp, receiver)?.qualified.clone();
    Ok(Some(interp.text_built(qualified)))
}

/// `query_exists`: the qualified name when it names a file, and the null
/// string otherwise -- a directory answers nothing, measured, because the
/// oracle's own check is `fileExists`.
pub(super) fn query_exists(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = require(interp, receiver)?;
    if state.handle.is_some() {
        return Ok(Some(interp.text(b"")));
    }
    let path = qualified_path(interp, receiver)?;
    let exists = std::fs::metadata(&path).is_ok_and(|meta| meta.is_file());
    let answer = if exists {
        path.into_bytes()
    } else {
        Vec::new()
    };
    Ok(Some(interp.text_built(answer)))
}

/// `query_size`: the byte count for a regular file, `0` for anything else
/// that stats, and the null string when nothing does. Measured: a directory
/// answers `0` here where `query_exists` answers nothing.
pub(super) fn query_size(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = qualified_path(interp, receiver)?;
    match std::fs::metadata(&path) {
        Ok(meta) => {
            let size = if meta.is_file() { meta.len() } else { 0 };
            Ok(Some(interp.text_built(size.to_string().into_bytes())))
        }
        Err(_) => Ok(Some(interp.text(b""))),
    }
}

/// `query_time`: the modification time as `ctime` renders it, which the Rexx
/// half re-parses into both the `DATETIME` and `TIMESTAMP` answers -- so the
/// shape here is fixed by that parse (`. month day time year`) and not by
/// taste.
pub(super) fn query_time(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = qualified_path(interp, receiver)?;
    let modified = std::fs::metadata(&path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|at| at.duration_since(std::time::UNIX_EPOCH).ok());
    let Some(since_epoch) = modified else {
        return Ok(Some(interp.text(b"")));
    };
    let seconds = i64::try_from(since_epoch.as_secs()).unwrap_or(0);
    let rendered = match chrono::Local.timestamp_opt(seconds, 0) {
        chrono::offset::LocalResult::Single(at) | chrono::offset::LocalResult::Ambiguous(at, _) => {
            at.format("%a %b %e %H:%M:%S %Y").to_string()
        }
        chrono::offset::LocalResult::None => return Ok(Some(interp.text(b""))),
    };
    Ok(Some(interp.text_built(rendered.into_bytes())))
}

/// `query_streamtype`: `UNKNOWN` until something opens the stream, which is
/// what tells a persistent stream from a transient one.
pub(super) fn query_streamtype(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    require(interp, receiver)?;
    Ok(Some(interp.text(b"UNKNOWN")))
}

/// `query_handle`: the null string until the stream is open.
pub(super) fn query_handle(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    require(interp, receiver)?;
    Ok(Some(interp.text(b"")))
}

/// `stream_close`: `READY:` for a stream that was open, and the null string
/// for one that never was. Nothing here is open yet, so the second is the
/// answer this phase gives.
pub(super) fn close(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = require_mut(interp, receiver)?;
    let was_open = state.status == StreamStatus::Ready;
    state.status = StreamStatus::Unknown;
    let answer: &[u8] = if was_open { b"READY:" } else { b"" };
    Ok(Some(interp.text(answer)))
}

/// `stream_flush`: `READY:`, including for a stream nothing has opened --
/// measured, and it leaves the state alone.
pub(super) fn flush(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    require(interp, receiver)?;
    Ok(Some(interp.text(b"READY:")))
}

/// `stream_uninit`: drops the state block, after which every entry point here
/// raises 48.1. **Answers nothing**, so a value context is 91.999 -- measured
/// on the oracle, which returns no result either.
pub(super) fn uninit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if let Some(object) = interp.heap.get_mut(receiver)
        && let Body::Instance { native, .. } = &mut object.body
    {
        *native = None;
    }
    Ok(None)
}
