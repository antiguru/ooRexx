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

use super::{Cleared, new_instance};
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

/// A write argument's bytes. `LINEOUT` and `CHAROUT` require a *string
/// value*, not a string rendering: an object answering `makeString` writes
/// what that answers, and one answering none refuses. Measured,
/// `.stdout~charout(.K~new)` is `Argument 1 must have a string value` at
/// rc 168 where `~string` would have answered `a K`, and an instance whose
/// class defines `makeString` writes what the method returns.
/// The refusal's own `Compiled method` traceback line comes from the dispatch
/// layer, not from here -- measured, blaming it at this site as well puts the
/// line in twice.
fn write_text(interp: &mut Interp, value: ObjRef) -> Result<Vec<u8>, Failure> {
    let text = super::required_string_argument(interp, value, 1)?;
    Ok(interp.to_text(text).into_owned())
}

/// The qualified path as a `String`, for the file-system calls below.
fn qualified_path(interp: &Interp, receiver: ObjRef) -> Result<String, Failure> {
    let state = require(interp, receiver)?;
    Ok(String::from_utf8_lossy(&state.qualified).into_owned())
}

/// One of the three standard streams, built for `.local` rather than by a
/// program. **Not `paths::normalize`d**: measured, `.stdout~qualify` answers
/// the bare `STDOUT`, where every other stream answers a path, so the
/// qualified name is the name. Ready from the start (`~state` `READY`,
/// `~description` `READY:`) and marked with `standard`, which is what parts it
/// from the ordinary `Stream` a program builds for the same name --
/// `.Stream~new('STDOUT')` is a different object.
///
/// No `INIT` runs, so the `stream_name` object variable `::METHOD string`
/// exposes is assigned here; an unset exposed variable would otherwise render
/// as its own derived name.
pub(crate) fn standard_stream(
    interp: &mut Interp,
    class: ObjRef,
    which: StandardStream,
) -> Result<ObjRef, Failure> {
    let name: &[u8] = match which {
        StandardStream::In => b"STDIN",
        StandardStream::Out => b"STDOUT",
        StandardStream::Err => b"STDERR",
    };
    let object = new_instance(interp, class)?;
    // Takes no part in finalization: it lives in `.local` for the run and
    // wraps a descriptor this crate did not open, and the sweep would
    // otherwise drop its state before a program's own finalizer writes
    // through it.
    interp.heap.clear_uninit_all(&[object]);
    let named = interp.text(name);
    interp.set_pool_variable(object, class, b"STREAM_NAME", named);
    let mut state = StreamState::new(name.to_vec(), name.to_vec());
    state.standard = Some(which);
    state.status = StreamStatus::Ready;
    let Some(held) = interp.heap.get_mut(object) else {
        unreachable!("new_instance just allocated and rooted it")
    };
    let Body::Instance { native, .. } = &mut held.body else {
        unreachable!("new_instance allocates Body::Instance")
    };
    *native = Some(Box::new(NativeState::Stream(state)));
    Ok(object)
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
        // A zero errno carries no text: the oracle appends `strerror` only
        // when it has an errno to render, so a `CHARIN` past the end is a
        // bare `ERROR:0`.
        StreamStatus::Error(0) => out.extend_from_slice(b"0"),
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
/// what tells a persistent stream from a transient one. A standard stream
/// answers from its descriptor instead, which it has without being opened.
pub(super) fn query_streamtype(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
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

/// `query_handle`: the descriptor a standard stream stands for, and the null
/// string for every other stream. Measured, the three answer `0`, `1` and `2`
/// whatever the descriptors are redirected to.
pub(super) fn query_handle(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let answer: &[u8] = match standard_of(interp, receiver)? {
        Some(StandardStream::In) => b"0",
        Some(StandardStream::Out) => b"1",
        Some(StandardStream::Err) => b"2",
        None => b"",
    };
    Ok(Some(interp.text(answer)))
}

/// The errno a failed open reports when there is nothing to open. A directory
/// opens on unix and `SysFile::open` closes it again and substitutes this
/// (`common/platform/unix/SysFile.cpp:139`-`:147`), so a directory answers
/// `ERROR:2` rather than `ERROR:21`.
const ENOENT: i32 = 2;

/// One token of an options string: runs of non-blanks, except that `=`, `+`,
/// `-` and `<` are single-character tokens and also end the run before them
/// (`streamLibrary/StreamCommandParser.cpp:56`-`:99`).
struct Tokens<'a> {
    source: &'a [u8],
    offset: usize,
    length: usize,
}

/// The bytes that end a token and, alone, form one.
const SPECIAL: &[u8] = b"=+-<";

impl<'a> Tokens<'a> {
    fn new(source: &'a [u8]) -> Tokens<'a> {
        Tokens {
            source,
            offset: 0,
            length: 0,
        }
    }

    fn next(&mut self) -> Option<&'a [u8]> {
        self.offset += self.length;
        while self.source.get(self.offset) == Some(&b' ') {
            self.offset += 1;
        }
        let Some(&first) = self.source.get(self.offset) else {
            self.length = 0;
            return None;
        };
        self.length = if SPECIAL.contains(&first) {
            1
        } else {
            self.source[self.offset..]
                .iter()
                .take_while(|byte| **byte != b' ' && !SPECIAL.contains(byte))
                .count()
        };
        Some(&self.source[self.offset..self.offset + self.length])
    }

    /// `previousToken`: the next scan re-reads the token just returned.
    fn back(&mut self) {
        self.length = 0;
    }
}

/// `StreamToken::toNumber`: digits only -- no sign, no exponent -- and no
/// overflow. An empty token converts to zero, which every caller rejects.
fn to_number(token: &[u8]) -> Option<u64> {
    let mut value: u64 = 0;
    for byte in token {
        let digit = byte.checked_sub(b'0').filter(|digit| *digit <= 9)?;
        value = value.checked_mul(10)?.checked_add(u64::from(digit))?;
    }
    Some(value)
}

/// An option the open table admits.
#[derive(Copy, Clone, PartialEq, Eq)]
enum OpenOption {
    Read,
    Write,
    Both,
    Append,
    Replace,
    NoBuffer,
    Binary,
    RecLength,
    Shared,
}

/// The table in its own order with each entry's minimum abbreviation: the
/// first entry whose leading bytes match caselessly wins, so `re` reaches
/// READ rather than REPLACE and is then too short for it (`StreamNative.cpp`'s
/// `tts`). The three SHARE options share a row because every `RX_SH_*` is `0`
/// on unix -- they are parsed for their conflicts and change no open.
const OPEN_TABLE: &[(&[u8], usize, OpenOption)] = &[
    (b"READ", 3, OpenOption::Read),
    (b"WRITE", 1, OpenOption::Write),
    (b"BOTH", 2, OpenOption::Both),
    (b"APPEND", 2, OpenOption::Append),
    (b"REPLACE", 3, OpenOption::Replace),
    (b"NOBUFFER", 3, OpenOption::NoBuffer),
    (b"BINARY", 2, OpenOption::Binary),
    (b"RECLENGTH", 3, OpenOption::RecLength),
    (b"SHARED", 6, OpenOption::Shared),
    (b"SHAREREAD", 6, OpenOption::Shared),
    (b"SHAREWRITE", 6, OpenOption::Shared),
];

/// What the options parsed to, in the C++'s own terms: the mode booleans and
/// the two `oflag` bits the table's mutual-exclusion actions test.
#[derive(Default)]
struct Parsed {
    read_only: bool,
    write_only: bool,
    read_write: bool,
    append: bool,
    truncate: bool,
    nobuffer: bool,
    record_based: bool,
    shared_set: bool,
    record_length: u64,
}

/// `parser()` over the open table. `Err` is the bare 93 every conflicting,
/// repeated, too-short or unknown option answers.
fn parse_open_options(options: &[u8]) -> Result<Parsed, ()> {
    let mut parsed = Parsed::default();
    let mut tokens = Tokens::new(options);
    while let Some(token) = tokens.next() {
        let found = OPEN_TABLE.iter().find(|(name, _, _)| {
            name.len() >= token.len() && name[..token.len()].eq_ignore_ascii_case(token)
        });
        let Some((_, minimum, option)) = found else {
            return Err(());
        };
        if token.len() < *minimum {
            return Err(());
        }
        match option {
            OpenOption::Read => {
                if parsed.read_only || parsed.read_write || parsed.write_only || parsed.append {
                    return Err(());
                }
                if parsed.truncate {
                    return Err(());
                }
                parsed.read_only = true;
            }
            OpenOption::Write => {
                if parsed.read_only || parsed.read_write || parsed.write_only {
                    return Err(());
                }
                parsed.write_only = true;
            }
            OpenOption::Both => {
                if parsed.read_only || parsed.read_write || parsed.write_only {
                    return Err(());
                }
                parsed.read_write = true;
            }
            OpenOption::Append => {
                if parsed.read_only || parsed.append || parsed.truncate {
                    return Err(());
                }
                parsed.append = true;
            }
            OpenOption::Replace => {
                if parsed.read_only || parsed.append || parsed.truncate {
                    return Err(());
                }
                parsed.truncate = true;
            }
            OpenOption::NoBuffer => {
                if parsed.nobuffer {
                    return Err(());
                }
                parsed.nobuffer = true;
            }
            OpenOption::Binary => {
                if parsed.record_based {
                    return Err(());
                }
                parsed.record_based = true;
            }
            OpenOption::RecLength => {
                // `MIB`: BINARY has to have come first.
                if !parsed.record_based {
                    return Err(());
                }
                match tokens.next() {
                    Some(number) if parsed.record_length == 0 => {
                        match to_number(number).filter(|length| *length != 0) {
                            Some(length) => parsed.record_length = length,
                            None => return Err(()),
                        }
                    }
                    // A length already set, or nothing left: the record length
                    // defaults and the token goes back.
                    _ => tokens.back(),
                }
            }
            OpenOption::Shared => {
                if parsed.shared_set {
                    return Err(());
                }
                parsed.shared_set = true;
            }
        }
    }
    Ok(parsed)
}

/// `stream_open`: `READY:`, or `ERROR:n` for an open the file system refused.
/// A conflicting, repeated, too-short or unknown option is a bare 93, as is a
/// `BINARY` whose record length ends up zero.
pub(super) fn open(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let options = match args.first().copied().flatten() {
        Some(value) => interp.to_text(value).into_owned(),
        None => Vec::new(),
    };
    let state = require_mut(interp, receiver)?;
    // A second open closes and reopens.
    state.open = None;
    state.mode = rexx_core::OpenMode::default();
    state.status = StreamStatus::Unknown;
    // The three standard streams take only NOBUFFER, and are ready without
    // touching the file system (`StreamNative.cpp`'s `openStd`).
    if state.standard.is_some() {
        state.status = StreamStatus::Ready;
        return Ok(Some(interp.text(b"READY:")));
    }
    let path = String::from_utf8_lossy(&state.qualified).into_owned();

    let Ok(mut parsed) = parse_open_options(&options) else {
        return Err(Raised::syntax(93, 0, Vec::new()).into());
    };
    // A truncating BINARY open has nothing to take a record length from.
    if parsed.record_based && parsed.truncate && parsed.record_length == 0 {
        return Err(Raised::syntax(93, 0, Vec::new()).into());
    }
    // No explicit WRITE or BOTH, and not READ: the default is BOTH.
    if !parsed.write_only && !parsed.read_write && !parsed.read_only {
        parsed.read_write = true;
    }
    // WRITE is rewritten to read-write, which is why `linein` reads from a
    // stream opened for writing.
    if parsed.write_only {
        parsed.read_write = true;
    }

    if parsed.read_only && !std::fs::metadata(&path).is_ok_and(|meta| meta.is_file()) {
        let state = require_mut(interp, receiver)?;
        state.status = StreamStatus::Error(ENOENT);
        let name = state.name.clone();
        interp.raise_notready(&name)?;
        return Ok(Some(
            interp.text_built(format!("ERROR:{ENOENT}").into_bytes()),
        ));
    }

    let mut opening = std::fs::OpenOptions::new();
    if parsed.read_only {
        opening.read(true);
    } else {
        opening.read(true).write(true).create(true);
        if parsed.truncate {
            opening.truncate(true);
        }
    }
    let opened = match opening.open(&path) {
        Ok(file) => file,
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(ENOENT);
            let state = require_mut(interp, receiver)?;
            state.status = StreamStatus::Error(errno);
            let name = state.name.clone();
            interp.raise_notready(&name)?;
            return Ok(Some(
                interp.text_built(format!("ERROR:{errno}").into_bytes()),
            ));
        }
    };
    // A directory opens on unix and is then rejected as if it had not
    // (`common/platform/unix/SysFile.cpp:139`-`:147`), which is why a
    // directory answers `ERROR:2` and not `ERROR:21`.
    let metadata = opened.metadata().ok();
    if metadata.as_ref().is_none_or(std::fs::Metadata::is_dir) {
        let state = require_mut(interp, receiver)?;
        state.status = StreamStatus::Error(ENOENT);
        let name = state.name.clone();
        interp.raise_notready(&name)?;
        return Ok(Some(
            interp.text_built(format!("ERROR:{ENOENT}").into_bytes()),
        ));
    }
    let size = metadata.map_or(0, |meta| meta.len());
    let transient = is_transient(&opened);
    // `checkStreamType`: a BINARY open with no RECLENGTH takes the file's own
    // size as the record, and a size of zero is the bare 93.
    if parsed.record_based && parsed.record_length == 0 {
        if size == 0 {
            return Err(Raised::syntax(93, 0, Vec::new()).into());
        }
        parsed.record_length = size;
    }
    // A persistent writeable stream writes at the end; a read-only one has no
    // write position at all, which is the `0` `QUERY POSITION WRITE` answers.
    let write_position: i64 = if parsed.read_only { 0 } else { size as i64 + 1 };

    let state = require_mut(interp, receiver)?;
    state.mode = rexx_core::OpenMode {
        read_only: parsed.read_only,
        write_only: parsed.write_only,
        read_write: parsed.read_write,
        append: parsed.append,
        nobuffer: parsed.nobuffer,
        record_based: parsed.record_based,
        record_length: parsed.record_length,
    };
    state.open = Some(rexx_core::OpenFile {
        file: opened,
        read_position: 1,
        write_position,
        line_read: 1,
        line_write: 0,
        line_read_char: 1,
        line_write_char: 0,
        line_size: 0,
        last_op_was_read: true,
        transient,
    });
    state.status = StreamStatus::Ready;
    Ok(Some(interp.text(b"READY:")))
}

/// How much of a file one read asks the system for.
const CHUNK: usize = 4096;

/// The byte a file may end with that a `LINEOUT` overwrites rather than
/// appends after (`StreamNative.cpp:2520`, a Windows convention the unix
/// build applies too).
const CTRL_Z: u8 = 0x1A;

/// A validated positive argument as a character position. The validator
/// answers `u64` because it rejects zero and everything below it; the
/// positions it feeds are signed because a later seek can drive them
/// negative.
fn as_position(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// A logical character position as a file offset. A position below 1 is not
/// a legal offset -- a `SEEK` can drive one negative and the oracle's `lseek`
/// fails there -- so it clamps, and the read or write then finds nothing.
fn offset_of(position: i64) -> u64 {
    u64::try_from(position).unwrap_or(0)
}

/// Bytes from `position` (1-based), stopping at the end of the file.
fn read_from(file: &mut std::fs::File, position: u64, len: usize) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::Start(position.saturating_sub(1)))?;
    let mut out = vec![0u8; len];
    let mut filled = 0;
    while filled < len {
        match file.read(&mut out[filled..])? {
            0 => break,
            got => filled += got,
        }
    }
    out.truncate(filled);
    Ok(out)
}

/// One line from `position`, and the position after it. The line drops its
/// terminating `\n` and the `\r` immediately before that -- a bare `\r` is
/// data (`SysFile::gets`) -- and a final unterminated line is still a line.
/// `None` at the end of the file.
fn read_line_from(
    file: &mut std::fs::File,
    position: u64,
) -> std::io::Result<Option<(Vec<u8>, i64)>> {
    let mut line = Vec::new();
    let mut at = position;
    loop {
        let chunk = read_from(file, at, CHUNK)?;
        if chunk.is_empty() {
            return Ok(if line.is_empty() {
                None
            } else {
                Some((line, at as i64))
            });
        }
        match chunk.iter().position(|byte| *byte == b'\n') {
            Some(end) => {
                line.extend_from_slice(&chunk[..end]);
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(Some((line, (at + end as u64 + 1) as i64)));
            }
            None => {
                line.extend_from_slice(&chunk);
                at += chunk.len() as u64;
            }
        }
    }
}

/// The lines from `position` to the end, counted the way `SysFile::countLines`
/// counts them: an unterminated final line counts.
fn count_lines_from(file: &mut std::fs::File, position: u64) -> std::io::Result<u64> {
    let mut count = 0;
    let mut at = position;
    loop {
        let chunk = read_from(file, at, CHUNK)?;
        if chunk.is_empty() {
            return Ok(count);
        }
        count += chunk.iter().filter(|byte| **byte == b'\n').count() as u64;
        at += chunk.len() as u64;
        if chunk.len() < CHUNK {
            // A tail with no newline after the last one is a line of its own.
            if chunk.last() != Some(&b'\n') {
                count += 1;
            }
            return Ok(count);
        }
    }
}

/// Whether an open descriptor is a transient stream: a character device
/// or a FIFO, which is what `QUERY STREAMTYPE` answers `TRANSIENT` for
/// (`SysFile::getStreamTypeInfo`). A regular file is persistent.
fn is_transient(file: &std::fs::File) -> bool {
    use std::os::unix::fs::FileTypeExt;
    file.metadata()
        .is_ok_and(|meta| meta.file_type().is_char_device() || meta.file_type().is_fifo())
}

/// The file's size, or 0 when it cannot be stat'd.
fn size_of(file: &std::fs::File) -> u64 {
    file.metadata().map_or(0, |meta| meta.len())
}

/// Opens the stream if nothing has, the way an implicit open does: read-write
/// first, then write-only or read-only depending on what the caller wants
/// (`StreamInfo::implicitOpen`). A read never creates the file.
fn ensure_open(interp: &mut Interp, receiver: ObjRef, for_write: bool) -> Result<bool, Failure> {
    if require(interp, receiver)?.open.is_some() {
        return Ok(true);
    }
    let path = String::from_utf8_lossy(&require(interp, receiver)?.qualified).into_owned();
    let mut opening = std::fs::OpenOptions::new();
    // **Never truncates.** An implicit open creates a missing file but keeps
    // what an existing one holds -- `implicitOpen` uses `RDWR_CREAT` with no
    // `O_TRUNC` -- which is what lets a `lineout` to an existing file append
    // at the end instead of emptying it.
    opening.read(true).write(true).truncate(false);
    if for_write {
        opening.create(true);
    }
    let (opened, mode) = match opening.open(&path) {
        Ok(file) => (
            file,
            rexx_core::OpenMode {
                read_write: true,
                ..rexx_core::OpenMode::default()
            },
        ),
        // `O_WRONLY` alone, as `implicitOpen`'s own fallback is: it creates
        // nothing, because the read-write attempt above already carried
        // `O_CREAT` for a write, so reaching here means the open failed for
        // some reason other than the file being absent.
        Err(_) if for_write => match std::fs::OpenOptions::new().write(true).open(&path) {
            Ok(file) => (
                file,
                rexx_core::OpenMode {
                    write_only: true,
                    ..rexx_core::OpenMode::default()
                },
            ),
            Err(error) => return fail_open(interp, receiver, &error),
        },
        Err(_) => match std::fs::OpenOptions::new().read(true).open(&path) {
            Ok(file) => (
                file,
                rexx_core::OpenMode {
                    read_only: true,
                    ..rexx_core::OpenMode::default()
                },
            ),
            Err(error) => return fail_open(interp, receiver, &error),
        },
    };
    let size = size_of(&opened);
    let transient = is_transient(&opened);
    let state = require_mut(interp, receiver)?;
    state.mode = mode;
    state.status = StreamStatus::Ready;
    state.open = Some(rexx_core::OpenFile {
        file: opened,
        read_position: 1,
        write_position: if for_write { size as i64 + 1 } else { 1 },
        line_read: 1,
        line_write: 0,
        line_read_char: 1,
        line_write_char: 0,
        line_size: 0,
        last_op_was_read: !for_write,
        transient,
    });
    Ok(true)
}

/// Records an implicit open's failure on the state and answers "not open".
fn fail_open(
    interp: &mut Interp,
    receiver: ObjRef,
    error: &std::io::Error,
) -> Result<bool, Failure> {
    let errno = error.raw_os_error().unwrap_or(ENOENT);
    let state = require_mut(interp, receiver)?;
    state.status = StreamStatus::Error(errno);
    let name = state.name.clone();
    // Every entry point that opens implicitly raises when the open fails --
    // measured, `lines` and `chars` on a name nothing resolves to each fire a
    // `CALL ON NOTREADY` handler once, and both still answer 0.
    interp.raise_notready(&name)?;
    Ok(false)
}

/// Which standard stream `receiver` is, when it is one. Every entry point
/// below asks this before opening anything: a standard stream has no
/// `OpenFile` and never touches the file system, so its branch has to come
/// ahead of `ensure_open`, which would otherwise open a *file* named `STDOUT`.
fn standard_of(interp: &Interp, receiver: ObjRef) -> Result<Option<StandardStream>, Failure> {
    Ok(require(interp, receiver)?.standard)
}

/// A write to a standard stream: to the sink for the two output ones, and a
/// refusal for `.STDIN`. Answers what the entry point answers -- `0` written,
/// or the residual count. Measured: `.stdin~lineout('x')` raises `NOTREADY`
/// and answers `1`, and the raise happens whether or not a trap is armed.
fn standard_write(
    interp: &mut Interp,
    receiver: ObjRef,
    which: StandardStream,
    bytes: &[u8],
    residual: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    match which {
        StandardStream::Out => interp.write_out(bytes),
        StandardStream::Err => interp.write_err(bytes),
        StandardStream::In => {
            let state = require_mut(interp, receiver)?;
            state.status = StreamStatus::Error(EACCES);
            let name = state.name.clone();
            interp.raise_notready(&name)?;
            return Ok(Some(interp.text_built(residual.to_vec())));
        }
    }
    let state = require_mut(interp, receiver)?;
    state.status = StreamStatus::Ready;
    Ok(Some(interp.text(b"0")))
}

/// Whether the descriptor behind a standard stream is transient, as the
/// `Invocation` reported it. **Not read from this process**: the interpreter
/// writes to a buffer rather than a descriptor, so the host's own fds say
/// nothing about the embedding's, and reading them made one program answer
/// differently under a binary, a test harness and a library.
fn standard_is_transient(interp: &Interp, which: StandardStream) -> bool {
    interp.standard_transient[match which {
        StandardStream::In => 0,
        StandardStream::Out => 1,
        StandardStream::Err => 2,
    }]
}

/// Positioning a transient stream is 93.958, and a standard stream reaches
/// that check by what the `Invocation` says rather than by an `OpenFile` it
/// does not have. **Only when the descriptor is transient**: measured against
/// the oracle, the same `CHAROUT( , 'xxxxx', 15)` answers `0` and writes when
/// fd 1 is a regular file and raises when it is a pipe.
fn refuse_transient_position(
    interp: &Interp,
    which: StandardStream,
    positioned: bool,
) -> Result<(), Failure> {
    if positioned && standard_is_transient(interp, which) {
        return Err(Raised::transient_positioning().into());
    }
    Ok(())
}

/// Everything a `charin`/`charout` invalidates: the line positions and the
/// cached count (`StreamInfo::resetLinePositions`).
fn reset_line_positions(open: &mut rexx_core::OpenFile) {
    open.line_read = 0;
    open.line_read_char = 0;
    open.line_size = 0;
}

/// `stream_charin`: `length` characters from `start`, or from the read
/// position. A short read is the end of the file and leaves `ERROR:0`, which
/// is what parts it from `LINEIN`'s `NOTREADY:EOF`.
pub(super) fn charin(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = optional_position(interp, args.first().copied().flatten(), 1)?;
    let length = match args.get(1).copied().flatten() {
        Some(value) => whole_number(interp, value, 2)?,
        None => 1,
    };
    if let Some(which) = standard_of(interp, receiver)? {
        refuse_transient_position(interp, which, start.is_some())?;
        let StandardStream::In = which else {
            return Ok(Some(interp.text(b"")));
        };
        let wanted = usize::try_from(length).unwrap_or(usize::MAX);
        // A zero-length ask reads nothing and leaves the state alone, as it
        // does on the file path: measured, `.stdin~charin( , 0)` answers the
        // null string and `~description` stays `READY:`, where treating it as
        // a read of nothing would leave `ERROR:0`.
        if wanted == 0 {
            return Ok(Some(interp.text(b"")));
        }
        let read = interp.input_bytes(wanted);
        // Three outcomes, not two, and the standard stream's rule is its own
        // rather than the file path's. Measured over a six-byte standard
        // input: asking for 50 answers all six and leaves `NOTREADY:EOF`,
        // asking for exactly six answers them and stays `READY:`, and a
        // further ask answers nothing and leaves `ERROR:0`.
        let status = match read.len() {
            0 => StreamStatus::Error(0),
            got if got < wanted => StreamStatus::NotReady,
            _ => StreamStatus::Ready,
        };
        let state = require_mut(interp, receiver)?;
        state.status = status;
        if status != StreamStatus::Ready {
            let name = state.name.clone();
            interp.raise_notready(&name)?;
        }
        return Ok(Some(interp.text_built(read)));
    }
    if !ensure_open(interp, receiver, false)? {
        return Ok(Some(interp.text(b"")));
    }
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"")));
    };
    if let Some(start) = start {
        if open.transient {
            return Err(Raised::transient_positioning().into());
        }
        open.read_position = as_position(start);
    }
    let at = open.read_position;
    let wanted = usize::try_from(length).unwrap_or(usize::MAX);
    if wanted == 0 {
        return Ok(Some(interp.text(b"")));
    }
    let read = read_from(&mut open.file, offset_of(at), wanted).unwrap_or_default();
    open.read_position = at + read.len() as i64;
    open.last_op_was_read = true;
    reset_line_positions(open);
    let short = read.len() < wanted;
    let name = if short {
        state.status = StreamStatus::Error(0);
        Some(state.name.clone())
    } else {
        None
    };
    if let Some(name) = name {
        interp.raise_notready(&name)?;
    }
    Ok(Some(interp.text_built(read)))
}

/// `stream_linein`: one line from `line`, or from the read position. At the
/// end of the file the answer is the null string and the state is
/// `NOTREADY:EOF`.
pub(super) fn linein(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let line = optional_position(interp, args.first().copied().flatten(), 1)?;
    let count = match args.get(1).copied().flatten() {
        Some(value) => whole_number(interp, value, 2)?,
        None => 1,
    };
    if !matches!(count, 0 | 1) {
        return Err(Raised::syntax(93, 0, Vec::new()).into());
    }
    if let Some(which) = standard_of(interp, receiver)? {
        refuse_transient_position(interp, which, line.is_some())?;
        let StandardStream::In = which else {
            return Ok(Some(interp.text(b"")));
        };
    } else {
        if !ensure_open(interp, receiver, false)? {
            return Ok(Some(interp.text(b"")));
        }
        if let Some(line) = line {
            seek_to_line(interp, receiver, line)?;
        }
    }
    if count == 0 {
        return Ok(Some(interp.text(b"")));
    }
    Ok(Some(match next_line(interp, receiver)? {
        Some(text) => interp.text_built(text),
        None => interp.text(b""),
    }))
}

/// One line from the read position, or `None` when there is none: the reading
/// half of `stream_linein`, shared with `stream_arrayin` so the two cannot
/// grow separate answers at the end of input.
///
/// **The end of input raises here.** Whether the caller sees that as an error
/// is [`Interp::raise_notready`]'s decision, and a `SIGNAL ON NOTREADY` makes
/// it one -- which is how the orx `arrayIn` ends its fill and answers the
/// array it passed in.
fn next_line(interp: &mut Interp, receiver: ObjRef) -> Result<Option<Vec<u8>>, Failure> {
    if let Some(which) = standard_of(interp, receiver)? {
        let StandardStream::In = which else {
            return Ok(None);
        };
        return match interp.input_line() {
            Some(text) => {
                // A read that succeeds clears whatever a refused write left:
                // measured, `.stdin~lineout` leaves `ERROR:13` and the next
                // `.stdin~linein` reads its line and answers `READY:` again.
                require_mut(interp, receiver)?.status = StreamStatus::Ready;
                Ok(Some(text))
            }
            None => {
                let state = require_mut(interp, receiver)?;
                state.status = StreamStatus::NotReady;
                let name = state.name.clone();
                interp.raise_notready(&name)?;
                Ok(None)
            }
        };
    }
    if !ensure_open(interp, receiver, false)? {
        return Ok(None);
    }
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(None);
    };
    let at = open.read_position;
    match read_line_from(&mut open.file, offset_of(at)).unwrap_or(None) {
        Some((text, next)) => {
            open.read_position = next;
            if open.line_read != 0 {
                open.line_read += 1;
                open.line_read_char = next;
            }
            open.last_op_was_read = true;
            Ok(Some(text))
        }
        None => {
            state.status = StreamStatus::NotReady;
            let name = state.name.clone();
            interp.raise_notready(&name)?;
            Ok(None)
        }
    }
}

/// `stream_arrayin`: fills the caller's array with the lines that are left.
///
/// **The array is the orx `arrayIn`'s own**, built as `.array~new(self~lines)`
/// and filled in place, and this answers nothing: the end of input raises, and
/// that method's `SIGNAL ON NOTREADY` is what returns the array. Appending is
/// right despite the array arriving pre-sized -- measured on both engines,
/// `.array~new(3)` holds no items and the first append lands on slot 1.
pub(super) fn arrayin(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(array) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    while let Some(text) = next_line(interp, receiver)? {
        let item = interp.text_built(text);
        // Rooted before the append, which allocates when the array grows.
        interp.roots.push_temp(item);
        super::collection::append_slot(interp, array, item)?;
    }
    Ok(None)
}

/// Moves the read position to the start of `line`, counting from the top.
fn seek_to_line(interp: &mut Interp, receiver: ObjRef, line: u64) -> Result<(), Failure> {
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(());
    };
    // The same rule `SEEK` follows, reached by naming a line instead:
    // measured, `.Stream~new('/dev/null')~supplier` raises this, because
    // `StreamSupplier~init` reads line 1 of a stream it has decided is not
    // transient before opening it.
    if open.transient {
        return Err(Raised::transient_positioning().into());
    }
    let mut at: i64 = 1;
    for _ in 1..line.max(1) {
        match read_line_from(&mut open.file, offset_of(at)).unwrap_or(None) {
            Some((_, next)) => at = next,
            None => break,
        }
    }
    open.read_position = at;
    open.line_read = line.max(1);
    open.line_read_char = at;
    Ok(())
}

/// `stream_charout`: writes `data` at `start` or the write position and
/// answers the count it could not write, which is `0` for every write that
/// completes. With neither argument it closes the stream.
pub(super) fn charout(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let data = match args.first().copied().flatten() {
        Some(value) => Some(write_text(interp, value)?),
        None => None,
    };
    let start = optional_position(interp, args.get(1).copied().flatten(), 2)?;
    if data.is_none() && start.is_none() {
        return close(interp, _cleared, receiver, &[]).map(|_| Some(interp.text(b"0")));
    }
    if let Some(which) = standard_of(interp, receiver)? {
        refuse_transient_position(interp, which, start.is_some())?;
        let data = data.unwrap_or_default();
        let residual = data.len().to_string().into_bytes();
        return standard_write(interp, receiver, which, &data, &residual);
    }
    if !ensure_open(interp, receiver, true)? {
        let residual = data.as_ref().map_or(0, Vec::len);
        return Ok(Some(interp.text_built(residual.to_string().into_bytes())));
    }
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    if let Some(start) = start {
        open.write_position = as_position(start);
    }
    let Some(data) = data else {
        return Ok(Some(interp.text(b"0")));
    };
    let at = open.write_position;
    match write_at(&mut open.file, offset_of(at), &data) {
        Ok(()) => {
            open.write_position = at + data.len() as i64;
            open.last_op_was_read = false;
            reset_line_positions(open);
            Ok(Some(interp.text(b"0")))
        }
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            state.status = StreamStatus::Error(errno);
            let name = state.name.clone();
            let residual = data.len().to_string().into_bytes();
            interp.raise_notready(&name)?;
            Ok(Some(interp.text_built(residual)))
        }
    }
}

/// `stream_lineout`: writes `data` and a newline, answering `0` on success and
/// `1` on failure. With neither argument it closes the stream.
pub(super) fn lineout(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let data = match args.first().copied().flatten() {
        Some(value) => Some(write_text(interp, value)?),
        None => None,
    };
    let line = optional_position(interp, args.get(1).copied().flatten(), 2)?;
    if data.is_none() && line.is_none() {
        return close(interp, _cleared, receiver, &[]).map(|_| Some(interp.text(b"0")));
    }
    if let Some(which) = standard_of(interp, receiver)? {
        refuse_transient_position(interp, which, line.is_some())?;
        let mut bytes = data.unwrap_or_default();
        bytes.push(b'\n');
        return standard_write(interp, receiver, which, &bytes, b"1");
    }
    if !ensure_open(interp, receiver, true)? {
        return Ok(Some(interp.text(b"1")));
    }
    if require(interp, receiver)?.mode.read_only {
        let state = require_mut(interp, receiver)?;
        state.status = StreamStatus::Error(EACCES);
        let name = state.name.clone();
        interp.raise_notready(&name)?;
        return Ok(Some(interp.text(b"1")));
    }
    let mode = require(interp, receiver)?.mode;
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"1")));
    };
    // `setLineWritePosition`: a line number names a line, not a character, so
    // it goes through the same walk `SEEK ... LINE` uses. Writing at the number
    // itself overwrites whatever happens to sit at that offset.
    if let Some(line) = line {
        let record_length = if mode.record_based {
            mode.record_length
        } else {
            0
        };
        let (from_line, from_char) = (open.line_write, open.line_write_char);
        let (number, at) = seek_line(open, as_position(line), record_length, from_line, from_char);
        open.line_write = number;
        open.line_write_char = at;
        open.write_position = at;
    }
    let Some(data) = data else {
        return Ok(Some(interp.text(b"0")));
    };
    // A file whose last byte is ctrl-Z has it overwritten rather than kept.
    let size = size_of(&open.file) as i64;
    let mut at = open.write_position;
    if at == size + 1 && size > 0 {
        let tail = read_from(&mut open.file, offset_of(size), 1).unwrap_or_default();
        if tail.first() == Some(&CTRL_Z) {
            at = size;
        }
    }
    let mut bytes = data;
    bytes.push(b'\n');
    match write_at(&mut open.file, offset_of(at), &bytes) {
        Ok(()) => {
            let appended = at == size + 1 || at == size;
            // **Measured, and the two disagree**: after a write placed by line
            // number the pointer lands at the end of the file plus what was
            // written, not after the bytes. An append reaches the same answer
            // by either arithmetic.
            open.write_position = if line.is_some() {
                size + bytes.len() as i64 + 1
            } else {
                at + bytes.len() as i64
            };
            open.last_op_was_read = false;
            if open.line_write > 0 {
                open.line_write += 1;
                open.line_write_char = open.write_position;
            }
            if appended && open.line_size != 0 {
                open.line_size += 1;
            } else if !appended {
                open.line_size = 0;
            }
            Ok(Some(interp.text(b"0")))
        }
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            state.status = StreamStatus::Error(errno);
            let name = state.name.clone();
            interp.raise_notready(&name)?;
            Ok(Some(interp.text(b"1")))
        }
    }
}

/// Writes `bytes` at `position` (1-based).
fn write_at(file: &mut std::fs::File, position: u64, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::{Seek, SeekFrom, Write};
    file.seek(SeekFrom::Start(position.saturating_sub(1)))?;
    file.write_all(bytes)
}

/// `stream_chars`: the characters left from the read position, which is the
/// size less what has been read, floored at zero.
pub(super) fn chars(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if let Some(which) = standard_of(interp, receiver)? {
        let StandardStream::In = which else {
            return Ok(Some(interp.text(b"0")));
        };
        // An unmeasurable source answers `1` while it is live: measured, a
        // pipe on the oracle's own standard input answers `1` where a
        // redirected regular file answers the true byte count.
        let left = interp.input_remaining().unwrap_or(1);
        return Ok(Some(interp.text_built(left.to_string().into_bytes())));
    }
    if !ensure_open(interp, receiver, false)? {
        return Ok(Some(interp.text(b"0")));
    }
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    let left = (size_of(&open.file) as i64 - (open.read_position - 1)).max(0);
    Ok(Some(interp.text_built(left.to_string().into_bytes())))
}

/// `stream_lines`: `Count` -- the method's default -- answers the lines left,
/// `Normal` answers 1 or 0. **The count carries the oracle's cache and its
/// off-by-one**: after a `CHARIN` has zeroed the line position, the cache is
/// stored one short and the next `Count` answers one less than this one.
pub(super) fn lines(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let quick = match args.first().copied().flatten() {
        Some(value) => interp
            .to_text(value)
            .first()
            .is_some_and(|byte| byte.eq_ignore_ascii_case(&b'N')),
        None => false,
    };
    if let Some(which) = standard_of(interp, receiver)? {
        let StandardStream::In = which else {
            return Ok(Some(interp.text(b"0")));
        };
        let left = interp.input_remaining_lines().unwrap_or(1);
        let answer = if quick { u64::from(left > 0) } else { left };
        return Ok(Some(interp.text_built(answer.to_string().into_bytes())));
    }
    if !ensure_open(interp, receiver, false)? {
        return Ok(Some(interp.text(b"0")));
    }
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    let size = size_of(&open.file) as i64;
    if open.read_position > size {
        return Ok(Some(interp.text(b"0")));
    }
    if quick {
        return Ok(Some(interp.text(b"1")));
    }
    let answer = if open.line_size > 0 && open.line_read > 0 {
        open.line_size - open.line_read + 1
    } else if open.line_size > 0 {
        // `countStreamLines`' own early return, which is what answers one
        // short after a `CHARIN` stored the cache with a zero line position.
        open.line_size
    } else {
        let count = count_lines_from(&mut open.file, offset_of(open.read_position)).unwrap_or(0);
        open.line_size = (count + open.line_read).saturating_sub(1);
        count
    };
    Ok(Some(interp.text_built(answer.to_string().into_bytes())))
}

/// An optional 1-based position argument: absent, or a positive whole number.
fn optional_position(
    interp: &mut Interp,
    value: Option<ObjRef>,
    which: usize,
) -> Result<Option<u64>, Failure> {
    let Some(value) = value else {
        return Ok(None);
    };
    let text = interp.to_text(value).into_owned();
    match to_number(&text).filter(|number| *number != 0) {
        Some(number) => Ok(Some(number)),
        None => Err(Raised::method_argument_not_positive(which, &text).into()),
    }
}

/// A whole-number argument that may be zero.
fn whole_number(interp: &mut Interp, value: ObjRef, which: usize) -> Result<u64, Failure> {
    let text = interp.to_text(value).into_owned();
    match to_number(&text) {
        Some(number) => Ok(number),
        None => Err(Raised::native_argument_out_of_range_unsigned(
            &format!("{which}"),
            0,
            u64::MAX,
            &text,
        )
        .into()),
    }
}

/// The errno a write to a read-only stream reports, which the stream layer
/// chooses rather than the system call reporting it.
const EACCES: i32 = 13;

/// Which end a `SEEK` offset is measured from.
#[derive(Copy, Clone, PartialEq, Eq)]
enum SeekFrom {
    Start,
    End,
    Forward,
    Backward,
}

/// What a `SEEK` option string parsed to.
struct Positioning {
    style: SeekFrom,
    style_set: bool,
    read: bool,
    write: bool,
    by_char: bool,
    by_line: bool,
    offset: Option<i64>,
}

/// `streamPosition`'s option table. **Every word takes one letter**, unlike
/// OPEN's per-option minimums, so `r`, `w`, `c` and `l` all match; a bare
/// number is the offset, and anything else fails.
fn parse_positioning(options: &[u8]) -> Result<Positioning, ()> {
    let mut parsed = Positioning {
        style: SeekFrom::Start,
        style_set: false,
        read: false,
        write: false,
        by_char: false,
        by_line: false,
        offset: None,
    };
    let mut tokens = Tokens::new(options);
    while let Some(token) = tokens.next() {
        let style = match token {
            b"=" => Some(SeekFrom::Start),
            b"<" => Some(SeekFrom::End),
            b"+" => Some(SeekFrom::Forward),
            b"-" => Some(SeekFrom::Backward),
            _ => None,
        };
        if let Some(style) = style {
            if parsed.style_set {
                return Err(());
            }
            parsed.style = style;
            parsed.style_set = true;
            continue;
        }
        if matches(token, b"READ") {
            if parsed.read || parsed.write {
                return Err(());
            }
            parsed.read = true;
        } else if matches(token, b"WRITE") {
            if parsed.read || parsed.write {
                return Err(());
            }
            parsed.write = true;
        } else if matches(token, b"CHAR") {
            if parsed.by_char || parsed.by_line {
                return Err(());
            }
            parsed.by_char = true;
        } else if matches(token, b"LINE") {
            if parsed.by_char || parsed.by_line {
                return Err(());
            }
            parsed.by_line = true;
        } else if parsed.offset.is_none() {
            // The fallback token: an offset, once, and digits only -- which
            // is why `seek 1e1` and `seek =1.5` are the bare 93.
            match to_number(token) {
                Some(number) => parsed.offset = Some(as_position(number)),
                None => return Err(()),
            }
        } else {
            return Err(());
        }
    }
    Ok(parsed)
}

/// A token against a table word, caselessly over the token's own length.
fn matches(token: &[u8], word: &[u8]) -> bool {
    !token.is_empty()
        && word.len() >= token.len()
        && word[..token.len()].eq_ignore_ascii_case(token)
}

/// `stream_position`, which both `~seek` and `~position` bind: moves a
/// position and answers where it landed. Nothing is range-checked -- `<99` on
/// a 29-byte file answers `-69` and the stream stays `READY`.
pub(super) fn position(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **The options string is required, and the check is the native
    // boundary's rather than this body's.** `RexxMethod2(int64_t,
    // stream_position, CSELF, streamPtr, CSTRING, options)` declares it
    // without `OPTIONAL_`, so omitting it is 88.901 against the package
    // rather than any answer of the stream's -- measured, `s~position` with
    // no argument reports `Error 88 running REXX` and names argument 1,
    // where `s~seek('write')` reaches the 93.903 below.
    let Some(value) = args.first().copied().flatten() else {
        return Err(Raised::missing_internal_argument("1").into());
    };
    let options = interp.to_text(value).into_owned();
    let Ok(mut parsed) = parse_positioning(&options) else {
        return Err(Raised::syntax(93, 0, Vec::new()).into());
    };
    // Both checks come **after** parsing: a transient stream with a bad option
    // answers the bare 93, not 93.958.
    if let Some(which) = standard_of(interp, receiver)? {
        refuse_transient_position(interp, which, true)?;
    }
    if require(interp, receiver)?
        .open
        .as_ref()
        .is_some_and(|open| open.transient)
    {
        return Err(Raised::transient_positioning().into());
    }
    let Some(offset) = parsed.offset else {
        return Err(Raised::missing_argument_named("SEEK").into());
    };
    let mode = require(interp, receiver)?.mode;
    require_mut(interp, receiver)?.status = StreamStatus::Ready;
    // Neither READ nor WRITE given: whichever the open admits, and both when
    // it admits both. An unopened stream admits neither and so takes the both
    // branch -- measured, a seek on one moves both pointers to the target.
    let collapse = if parsed.read || parsed.write {
        false
    } else if mode.read_only {
        parsed.read = true;
        false
    } else if mode.write_only {
        parsed.write = true;
        false
    } else {
        parsed.read = true;
        parsed.write = true;
        true
    };
    // The implicit open comes after the flags and never creates the file: a
    // seek naming one that does not exist answers 0 and leaves state ERROR.
    // Asking for the read-shaped open is what selects that, because it is the
    // `operation_nocreate` one whichever pointer is about to move.
    if !ensure_open(interp, receiver, false)? {
        return Ok(Some(interp.text(b"0")));
    }
    let record_length = if mode.record_based {
        mode.record_length
    } else {
        0
    };
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    // The C++ branches on `last_op_was_read` here and its own comment says the
    // flag is always true (`bugs:#1739`); measured, both pointers move together
    // whichever operation ran last, so this does not branch.
    if collapse {
        open.write_position = open.read_position;
        open.line_write = open.line_read;
    }
    if parsed.read {
        open.line_size = 0;
    }
    if !parsed.by_char && !parsed.by_line {
        parsed.by_char = true;
    }
    let offset = if parsed.style == SeekFrom::Backward {
        -offset
    } else {
        offset
    };
    let size = size_of(&open.file) as i64;
    if parsed.by_char {
        let current = if parsed.read {
            open.read_position
        } else {
            open.write_position
        };
        // `setPosition` seeks and adds one, so an absolute offset is used as
        // written and everything else lands relative, unchecked either way.
        let landed = match parsed.style {
            SeekFrom::Start => offset,
            SeekFrom::End => size - offset + 1,
            SeekFrom::Forward | SeekFrom::Backward => current + offset,
        };
        reset_line_positions(open);
        if parsed.read {
            open.read_position = landed;
            if parsed.write {
                open.write_position = landed;
            }
        } else {
            open.write_position = landed;
        }
        return Ok(Some(interp.text_built(landed.to_string().into_bytes())));
    }
    // Line positioning on a stream that cannot read answers 0.
    if !(mode.read_write || mode.read_only) {
        return Ok(Some(interp.text(b"0")));
    }
    let (current_line, current_char) = if parsed.read {
        (open.line_read, open.line_read_char)
    } else {
        (open.line_write, open.line_write_char)
    };
    let target = match parsed.style {
        SeekFrom::Start => offset,
        SeekFrom::End => as_position(line_count(open, record_length)) - offset,
        SeekFrom::Forward | SeekFrom::Backward => as_position(current_line) + offset,
    };
    // Clamped at the bottom only: `seek 99 read line` on a three-line file
    // answers 4, and `seek 0 read line` answers 1.
    let (line, at) = seek_line(
        open,
        target.max(1),
        record_length,
        current_line,
        current_char,
    );
    if parsed.read {
        open.line_read = line;
        open.line_read_char = at;
        open.read_position = at;
        if parsed.write {
            open.line_write = line;
            open.line_write_char = at;
            open.write_position = at;
        }
    } else {
        open.line_write = line;
        open.line_write_char = at;
        open.write_position = at;
    }
    Ok(Some(interp.text_built(line.to_string().into_bytes())))
}

/// The lines a stream holds, for a `<n` line seek. A record-based stream
/// divides rather than scans, and a partial trailing record still counts.
fn line_count(open: &mut rexx_core::OpenFile, record_length: u64) -> u64 {
    let size = size_of(&open.file);
    match size.checked_div(record_length) {
        Some(whole) => whole + u64::from(!size.is_multiple_of(record_length)),
        None => count_lines_from(&mut open.file, 1).unwrap_or(0),
    }
}

/// Walks to the start of `target`, answering the line it reached and that
/// line's character position. A target past the last line stops at the end,
/// which is what clamps `seek 99 read line` to the last line plus one.
fn seek_line(
    open: &mut rexx_core::OpenFile,
    target: i64,
    record_length: u64,
    current_line: u64,
    current_char: i64,
) -> (u64, i64) {
    if target <= 1 {
        return (1, 1);
    }
    if record_length > 0 {
        let line = u64::try_from(target).unwrap_or(1);
        return (line, as_position(record_length) * (target - 1) + 1);
    }
    // `seekToVariableLine`: already being there is a no-op that leaves a
    // mid-line character position alone, and a backward move restarts the walk
    // from line 1 rather than scanning backwards.
    if as_position(current_line) == target {
        return (current_line, current_char);
    }
    let (mut line, mut at) = if current_line == 0 || as_position(current_line) > target {
        (1u64, 1i64)
    } else {
        (current_line, current_char)
    };
    while (line as i64) < target {
        match read_line_from(&mut open.file, offset_of(at)).unwrap_or(None) {
            Some((_, next)) => {
                at = next;
                line += 1;
            }
            None => break,
        }
    }
    (line, at)
}

/// `getLineReadPosition`: the tracked read line, recomputed from the character
/// position when a character operation invalidated it -- measured, a `CHARIN`
/// of three bytes leaves `QUERY POSITION READ LINE` answering 1, not 0.
fn line_read_position(open: &mut rexx_core::OpenFile, record_length: u64) -> i64 {
    if record_length > 0 {
        return (open.read_position - 1) / as_position(record_length) + 1;
    }
    if open.line_read == 0 {
        open.line_read = count_lines_upto(&mut open.file, open.read_position).unwrap_or(0);
    }
    open.line_read_char = open.read_position;
    as_position(open.line_read)
}

/// `getLineWritePosition`, which differs from the read side by a trailing
/// `+ 1`: measured, a write position at the end of a three-line file answers 4
/// where the read position on line 2 answers 2. The tracker starts untracked,
/// so a stream that has written nothing recomputes here rather than answering
/// the 1 an initialised tracker would hold.
fn line_write_position(open: &mut rexx_core::OpenFile, record_length: u64) -> i64 {
    if record_length > 0 {
        let reclen = as_position(record_length);
        return open.write_position / reclen + i64::from(open.write_position % reclen != 0);
    }
    if open.line_write == 0 {
        open.line_write = count_lines_upto(&mut open.file, open.write_position).unwrap_or(0) + 1;
    }
    open.line_write_char = open.write_position;
    as_position(open.line_write)
}

/// `queryLinePosition`: which line a 1-based character position lies in,
/// counting from the start of the file, an unterminated tail counting as a
/// line. **The range runs through that position rather than up to it** -- the
/// C++ counts `[0, position - 1]` inclusive, so position 1 counts one byte and
/// answers 1, which is what makes a write position of 1 answer line 2.
fn count_lines_upto(file: &mut std::fs::File, position: i64) -> std::io::Result<u64> {
    let bytes = offset_of(position.max(1));
    let mut count = 0;
    let mut at = 1u64;
    let mut remaining = bytes;
    let mut last = b'\n';
    while remaining > 0 {
        let want = usize::try_from(remaining.min(CHUNK as u64)).unwrap_or(CHUNK);
        let chunk = read_from(file, at, want)?;
        let Some(tail) = chunk.last() else { break };
        count += chunk.iter().filter(|byte| **byte == b'\n').count() as u64;
        last = *tail;
        at += chunk.len() as u64;
        remaining -= chunk.len() as u64;
    }
    if last != b'\n' {
        count += 1;
    }
    Ok(count)
}

/// `stream_query_position`: `SYS` answers the descriptor's own offset, and
/// the rest answer a logical position. An unopened stream answers the null
/// string, a transient one answers `1`.
pub(super) fn query_position(
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
    let mode = require(interp, receiver)?.mode;
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"")));
    };
    if open.transient {
        return Ok(Some(interp.text(b"1")));
    }
    if sys {
        use std::io::Seek;
        let at = open.file.stream_position().unwrap_or(0);
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
    let answer = match (write, by_line) {
        (true, true) => line_write_position(open, record_length),
        (true, false) => open.write_position,
        (false, true) => line_read_position(open, record_length),
        (false, false) => open.read_position,
    };
    Ok(Some(interp.text_built(answer.to_string().into_bytes())))
}

/// `stream_close`: `READY:` for a stream that was open, and the null string
/// for one that never was. Either way the state goes back to `UNKNOWN`.
///
/// A standard stream has no [`rexx_core::OpenFile`] to take, so it answers
/// from its state instead. Measured, and the three rows agree on one rule --
/// anything but `UNKNOWN` closes as `READY:`: the first close answers
/// `READY:` and the second the null string, a write after a close makes the
/// next close `READY:` again, and an exhausted `.STDIN` sitting at `NOTREADY`
/// also closes as `READY:`.
pub(super) fn close(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = require_mut(interp, receiver)?;
    let was_open = if state.standard.is_some() {
        state.status != StreamStatus::Unknown
    } else {
        state.open.take().is_some()
    };
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
