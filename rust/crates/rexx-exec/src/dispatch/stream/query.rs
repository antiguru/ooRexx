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

//! Stream metadata and logical or descriptor position queries.

use super::*;
use chrono::TimeZone;

/// `query_exists`: the qualified name when it names a file, and the null
/// string otherwise -- a directory answers nothing, measured, because the
/// oracle's own check is `fileExists`.
pub(in crate::dispatch) fn query_exists(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = require(interp, receiver)?;
    if state.handle.is_some() {
        let answer = match state.status {
            StreamStatus::Unknown => Vec::new(),
            _ => state.name.clone(),
        };
        return Ok(Some(interp.text_built(answer)));
    }
    // An open standard stream's qualified name is its name as written
    // (`StreamInfo::openStd`), which it answers without a lookup.
    if state.standard.is_some() && state.status != StreamStatus::Unknown {
        let answer = state.name.clone();
        return Ok(Some(interp.text_built(answer)));
    }
    // `StreamInfo::streamExists`: an open stream answers without a lookup, a
    // device by the name as written and anything else by its qualified name.
    if let Some(open) = state
        .open
        .as_ref()
        .filter(|_| state.status != StreamStatus::Unknown)
    {
        let answer = if is_device(open.file()) {
            state.name.clone()
        } else {
            state.qualified.to_vec()
        };
        return Ok(Some(interp.text_built(answer)));
    }
    // Closing an owned standard alias retains its descriptor and qualification
    // for reopening, but EXISTS must still check the ordinary named file.
    let retained_standard_name = state.standard.is_some() && state.open.is_some();
    let path = qualified_path(interp, receiver)?;
    let exists = std::fs::metadata(&path).is_ok_and(|meta| meta.is_file());
    let answer = if exists {
        if retained_standard_name {
            require(interp, receiver)?.qualified.to_vec()
        } else {
            path.into_os_string().into_encoded_bytes()
        }
    } else {
        Vec::new()
    };
    Ok(Some(interp.text_built(answer)))
}

/// Whether this stream uses the embedding's owned regular-file stdout descriptor.
pub(super) fn owned_standard_output(interp: &Interp, receiver: ObjRef) -> Result<bool, Failure> {
    Ok(
        require(interp, receiver)?.standard == Some(StandardStream::Out)
            && !interp.standard_transient[1]
            && interp
                .sinks
                .as_ref()
                .is_some_and(|sinks| sinks.own_descriptors),
    )
}

/// `query_size`: the byte count for a regular file, `0` for anything else
/// that stats, and the null string when nothing does. Measured: a directory
/// answers `0` here where `query_exists` answers nothing. An open stream
/// answers from its descriptor, once its held writes are in it
/// (`StreamInfo::getStreamSize`).
pub(in crate::dispatch) fn query_size(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let unopened = unopened_standard(interp, receiver)?;
    if !unopened && owned_standard_output(interp, receiver)? {
        interp.hand_over_stdout();
        if owned_stdout_alias(interp, receiver)? {
            if let Some(error) = ensure_owned_stdout_file(interp, receiver)? {
                return open_failed(interp, receiver, &error);
            }
        } else {
            let size = rustix::fs::fstat(std::io::stdout()).ok().and_then(|stat| {
                (rustix::fs::FileType::from_raw_mode(stat.st_mode)
                    == rustix::fs::FileType::RegularFile)
                    .then_some(stat.st_size)
            });
            return Ok(Some(match size {
                Some(size) => interp.text_built(size.to_string().into_bytes()),
                None => interp.text(b""),
            }));
        }
    }
    let open = if unopened {
        None
    } else if standard_input_file(interp, receiver)? {
        interp.input_file()
    } else {
        require_mut(interp, receiver)?.open.as_mut()
    };
    if let Some(open) = open {
        let size = open
            .flushed()
            .metadata()
            .ok()
            .filter(std::fs::Metadata::is_file)
            .map_or(0, |meta| meta.len());
        return Ok(Some(interp.text_built(size.to_string().into_bytes())));
    }
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
pub(in crate::dispatch) fn query_time(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let metadata = match standard_input_file(interp, receiver)? {
        true => interp.input_file().map(|open| open.file().metadata()),
        false => None,
    };
    let metadata = match metadata {
        Some(metadata) => metadata,
        None => std::fs::metadata(qualified_path(interp, receiver)?),
    };
    let modified = metadata
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
/// what tells a persistent stream from a transient one. A standard stream
/// answers from its descriptor instead, which it has without being opened.
pub(in crate::dispatch) fn query_streamtype(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if unopened_standard(interp, receiver)? {
        return Ok(Some(interp.text(b"UNKNOWN")));
    }
    if let Some(which) = standard_of(interp, receiver)? {
        let answer: &[u8] = if standard_is_transient(interp, which) {
            b"TRANSIENT"
        } else {
            b"PERSISTENT"
        };
        return Ok(Some(interp.text(answer)));
    }
    let answer: &[u8] = match require(interp, receiver)?.open.as_ref() {
        None => b"UNKNOWN",
        Some(open) if open.transient => b"TRANSIENT",
        Some(_) => b"PERSISTENT",
    };
    Ok(Some(interp.text(answer)))
}

/// `query_handle`: the descriptor a standard stream stands for, the
/// descriptor of an open stream, and the null string for an unopened one.
/// Measured, the standard three answer `0`, `1` and `2` whatever the
/// descriptors are redirected to.
pub(in crate::dispatch) fn query_handle(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    use std::os::fd::AsRawFd;
    if unopened_standard(interp, receiver)? {
        return Ok(Some(interp.text(b"")));
    }
    let answer: &[u8] = match standard_of(interp, receiver)? {
        Some(StandardStream::In) => b"0",
        Some(StandardStream::Out) => b"1",
        Some(StandardStream::Err) => b"2",
        None => {
            let handle = require(interp, receiver)?
                .open
                .as_ref()
                .map(|open| open.file().as_raw_fd().to_string().into_bytes());
            return Ok(Some(interp.text_built(handle.unwrap_or_default())));
        }
    };
    Ok(Some(interp.text(answer)))
}

/// `stream_query_position`: `SYS` answers the descriptor's own offset, and
/// the rest answer a logical position. An unopened stream answers the null
/// string, a transient one answers `1`.
pub(in crate::dispatch) fn query_position(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let options = match args.first().copied().flatten() {
        Some(value) => interp.to_text(value).into_owned(),
        None => Vec::new(),
    };
    let mut sys = false;
    let mut read = false;
    let mut write = false;
    let mut by_char = false;
    let mut by_line = false;
    let mut tokens = Tokens::new(&options);
    // Each token is mutually exclusive with a set of the others and the check
    // runs before the flag is set, so a repeat is an error too: `SYS` combines
    // with nothing and `READ WRITE` is a bare 93, while `READ LINE` is the pair
    // `StreamSupplier~init` itself sends.
    while let Some(token) = tokens.next() {
        let clash = if matches(token, b"SYS") {
            let clash = sys || read || write || by_char || by_line;
            sys = true;
            clash
        } else if matches(token, b"READ") {
            let clash = sys || read || write;
            read = true;
            clash
        } else if matches(token, b"WRITE") {
            let clash = sys || read || write;
            write = true;
            clash
        } else if matches(token, b"CHAR") {
            let clash = sys || by_char || by_line;
            by_char = true;
            clash
        } else if matches(token, b"LINE") {
            let clash = sys || by_char || by_line;
            by_line = true;
            clash
        } else {
            true
        };
        if clash {
            return Err(Raised::syntax(93, 0, Vec::new()).into());
        }
    }
    if unopened_standard(interp, receiver)? {
        return Ok(Some(interp.text(b"")));
    }
    if owned_stdout_alias(interp, receiver)? {
        interp.hand_over_stdout();
        if let Some(error) = ensure_owned_stdout_file(interp, receiver)? {
            return open_failed(interp, receiver, &error);
        }
    } else if !by_line && owned_standard_output(interp, receiver)? {
        if sys {
            interp.hand_over_stdout();
        }
        let at = rustix::fs::seek(std::io::stdout(), rustix::fs::SeekFrom::Current(0)).ok();
        let answer = if sys {
            at
        } else if !write {
            Some(1)
        } else {
            at.map(|at| at.saturating_add(interp.out.len() as u64).saturating_add(1))
        };
        return Ok(Some(match answer {
            Some(at) => interp.text_built(at.to_string().into_bytes()),
            None => interp.text(b""),
        }));
    }
    let mode = require(interp, receiver)?.mode;
    let mode = if standard_input_file(interp, receiver)? {
        rexx_core::OpenMode {
            read_only: true,
            ..rexx_core::OpenMode::default()
        }
    } else {
        mode
    };
    let open = match standard_input_file(interp, receiver)? {
        true => interp.input_file(),
        false => require_mut(interp, receiver)?.open.as_mut(),
    };
    let Some(open) = open else {
        return Ok(Some(interp.text(b"")));
    };
    if open.transient {
        return Ok(Some(interp.text(b"1")));
    }
    if sys {
        let at = open.offset();
        return Ok(Some(interp.text_built(at.to_string().into_bytes())));
    }
    // Neither given: the write position only for a write-only stream.
    if !read && !write {
        write = mode.write_only;
    }
    let record_length = if mode.record_based {
        mode.record_length
    } else {
        0
    };
    // By lines, the line the position is in (rexxref funct.xml:4550-4555); a
    // READ stream's write position is 0 in lines too.
    let answer = match (write, by_line) {
        (true, true) if open.write_position == 0 => Ok(0),
        (true, true) => line_of(open, open.write_position, record_length),
        (true, false) => Ok(open.write_position),
        (false, true) => line_of(open, open.read_position, record_length),
        (false, false) => Ok(open.read_position),
    };
    // A line count that cannot read the file raises with its errno: measured,
    // `ERROR:9` on a stream opened write-only.
    match answer {
        Ok(answer) => Ok(Some(interp.text_built(answer.to_string().into_bytes()))),
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            notready(interp, receiver, StreamStatus::Error(errno), Vec::new())
        }
    }
}
