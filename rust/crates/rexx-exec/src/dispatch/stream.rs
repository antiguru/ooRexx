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
use rexx_core::{Body, Decoded, NativeState, ObjRef, StandardStream, StreamState, StreamStatus};
use rexx_num::{DIGITS64, Number};

use super::{Cleared, new_instance};
use crate::error::Raised;
use crate::lines::{
    CHUNK, as_position, line_count, line_of, lines_after, offset_of, read_from, seek_line, size_of,
};
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
    let qualified = crate::builtin::platform::qualify_name(interp, &name);
    let state = StreamState::new(name, qualified);
    let Some(object) = interp.heap.get_mut(receiver) else {
        unreachable!("a live receiver")
    };
    // The receiver may hold a state of another class's, a buffer's among
    // them, where this method was run against it.
    let before = object.body.held_bytes();
    let Body::Instance { native, .. } = &mut object.body else {
        unreachable!("Stream is allocated as Body::Instance")
    };
    *native = Some(Box::new(NativeState::Stream(state)));
    interp.heap.release_body_bytes(before);
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

/// `handle_set`: marks a `HANDLE:n` stream, `n` an `int`. `HANDLE:0`,
/// `HANDLE:1` and `HANDLE:2` read and write through the standard stream of
/// that number (Deviation 35), and any other descriptor is refused.
pub(super) fn handle_set(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_native_argument(1).into());
    };
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => {
            let text = interp.to_text(value);
            Number::parse_bytes(&text).and_then(|number| number.int64_value(DIGITS64))
        }
    };
    let (min, max) = (i64::from(i32::MIN), i64::from(i32::MAX));
    let Some(descriptor) = number.filter(|number| (min..=max).contains(number)) else {
        // A marshalling raise, reported against the REXX package with no
        // line: measured, oracle, `.Stream~new('HANDLE:x')` is `Error 88
        // running REXX:`.
        let found = interp.string_value_text(value);
        let mut raised =
            Raised::native_argument_outside_range(1, i128::from(min), i128::from(max), &found);
        raised.delivery.lineless = true;
        raised.delivery.internal_package = true;
        return Err(raised.into());
    };
    let which = match descriptor {
        0 => StandardStream::In,
        1 => StandardStream::Out,
        2 => StandardStream::Err,
        _ => return Err(crate::Loud::handle_descriptor(descriptor).into()),
    };
    let state = require_mut(interp, receiver)?;
    state.handle = Some(which);
    state.standard = Some(which);
    Ok(None)
}

/// Whether `receiver` is a `HANDLE:n` stream nothing has opened, which the
/// queries answer as the oracle answers an unopened stream.
fn unopened_handle(interp: &Interp, receiver: ObjRef) -> Result<bool, Failure> {
    let state = require(interp, receiver)?;
    Ok(state.handle.is_some() && state.status == StreamStatus::Unknown)
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
        // when it has an errno to render.
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
        let answer = match state.status {
            StreamStatus::Unknown => Vec::new(),
            _ => state.name.clone(),
        };
        return Ok(Some(interp.text_built(answer)));
    }
    // `StreamInfo::streamExists`: an open stream answers without a lookup, a
    // device by the name as written and anything else by its qualified name.
    if let Some(open) = state.open.as_ref() {
        let answer = if is_device(&open.file) {
            state.name.clone()
        } else {
            state.qualified.clone()
        };
        return Ok(Some(interp.text_built(answer)));
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
    if unopened_handle(interp, receiver)? {
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
pub(super) fn query_handle(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    use std::os::fd::AsRawFd;
    if unopened_handle(interp, receiver)? {
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
                .map(|open| open.file.as_raw_fd().to_string().into_bytes());
            return Ok(Some(interp.text_built(handle.unwrap_or_default())));
        }
    };
    Ok(Some(interp.text(answer)))
}

/// The errno a failed open reports when there is nothing to open. A directory
/// opens on unix and `SysFile::open` closes it again and substitutes this
/// (`common/platform/unix/SysFile.cpp:139`-`:147`), so a directory answers
/// `ERROR:2` rather than `ERROR:21`.
const ENOENT: i32 = 2;

/// Puts the stream in `status` and raises `NOTREADY` for it, answering
/// `answer`: what the entry point returns when no trap transfers control,
/// and the condition's `RESULT` (`StreamInfo::notreadyError`).
#[cold]
#[inline(never)]
fn notready(
    interp: &mut Interp,
    receiver: ObjRef,
    status: StreamStatus,
    answer: Vec<u8>,
) -> Result<Option<ObjRef>, Failure> {
    let state = require_mut(interp, receiver)?;
    state.status = status;
    let name = state.name.clone();
    let answer = interp.text_built(answer);
    interp.roots.activity_mut().push_temp(answer);
    interp.raise_notready(&name, receiver, answer)?;
    Ok(Some(answer))
}

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
    // A `HANDLE:n` stream takes the options a file does and reports its name
    // as written from here on (`StreamNative.cpp`'s `handleOpen`).
    if state.handle.is_some() {
        state.qualified = state.name.clone();
        if parse_open_options(&options).is_err() {
            return Err(Raised::syntax(93, 0, Vec::new()).into());
        }
        state.status = StreamStatus::Ready;
        return Ok(Some(interp.text(b"READY:")));
    }
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
    // WRITE opens the stream for writing only (rexxref funct.xml:4302). The
    // descriptor is still opened for reading where it can be, so a line
    // number given to LINEOUT can be found; the reads refuse by the mode.

    if parsed.read_only && !std::fs::metadata(&path).is_ok_and(|meta| meta.is_file()) {
        let answer = format!("ERROR:{ENOENT}").into_bytes();
        return notready(interp, receiver, StreamStatus::Error(ENOENT), answer);
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
        // An explicit WRITE retries write-only, so a file that grants write
        // and not read opens (`streamOpen`, bug 3274050). The retry is the
        // oracle's `WR_CREAT`, which does not truncate even under REPLACE.
        Err(_) if parsed.write_only => {
            match std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)
            {
                Ok(file) => file,
                Err(error) => return open_failed(interp, receiver, &error),
            }
        }
        Err(error) => return open_failed(interp, receiver, &error),
    };
    // A directory opens on unix and is then rejected as if it had not
    // (`common/platform/unix/SysFile.cpp:139`-`:147`), which is why a
    // directory answers `ERROR:2` and not `ERROR:21`.
    let metadata = opened.metadata().ok();
    if metadata.as_ref().is_none_or(std::fs::Metadata::is_dir) {
        let answer = format!("ERROR:{ENOENT}").into_bytes();
        return notready(interp, receiver, StreamStatus::Error(ENOENT), answer);
    }
    let size = metadata.map_or(0, |meta| meta.len());
    let transient = is_transient(&opened);
    // `checkStreamType`: a BINARY open with no RECLENGTH takes the file's own
    // size as the record, and a size of zero is the bare 93. A transient
    // stream's record is one byte.
    if parsed.record_based && parsed.record_length == 0 {
        if transient {
            parsed.record_length = 1;
        } else if size == 0 {
            return Err(Raised::syntax(93, 0, Vec::new()).into());
        } else {
            parsed.record_length = size;
        }
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
        pending: None,
        transient,
        line_mark: None,
        line_total: None,
        line_mark_digest: None,
        line_total_digest: None,
    });
    state.status = StreamStatus::Ready;
    Ok(Some(interp.text(b"READY:")))
}

/// An explicit open the file system refused: `NOTREADY` answering `ERROR:n`.
fn open_failed(
    interp: &mut Interp,
    receiver: ObjRef,
    error: &std::io::Error,
) -> Result<Option<ObjRef>, Failure> {
    let errno = error.raw_os_error().unwrap_or(ENOENT);
    let answer = format!("ERROR:{errno}").into_bytes();
    notready(interp, receiver, StreamStatus::Error(errno), answer)
}

/// Up to `len` bytes: from `position` (1-based) of a persistent stream, or
/// the next ones of a transient stream, which has no position and waits
/// until `len` arrive or its input ends (rexxref funct.xml:1329-1335). A
/// byte [`transient_has_data`] took comes first.
fn read_at(open: &mut rexx_core::OpenFile, position: u64, len: usize) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    if !open.transient {
        return read_from(&mut open.file, position, len);
    }
    let mut out = Vec::new();
    let mut len = len;
    if len > 0
        && let Some(byte) = open.pending.take()
    {
        out.push(byte);
        len -= 1;
    }
    (&mut open.file)
        .take(u64::try_from(len).unwrap_or(u64::MAX))
        .read_to_end(&mut out)?;
    Ok(out)
}

/// Whether data waits on a transient stream, found without reading it: the
/// byte count `FIONREAD` reports for a pipe, FIFO or terminal (rexxref
/// iostrms.xml:189-193). A device that refuses `FIONREAD` has no way to tell
/// but a read, so a `poll` that finds nothing readable is 0, and otherwise
/// one byte is read and kept for the next read, which returns it first; a
/// read at the end of input finds none, and is 0. A state that cannot be
/// learned answers 1.
fn transient_has_data(open: &mut rexx_core::OpenFile) -> bool {
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    use std::io::Read;
    if open.pending.is_some() {
        return true;
    }
    if let Ok(waiting) = rustix::io::ioctl_fionread(&open.file) {
        return waiting > 0;
    }
    let zero = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let mut polled = [PollFd::new(&open.file, PollFlags::IN)];
    match poll(&mut polled, Some(&zero)) {
        Ok(_) if !polled[0].revents().contains(PollFlags::IN) => return false,
        Ok(_) => {}
        Err(_) => return true,
    }
    let mut byte = [0u8];
    match open.file.read(&mut byte) {
        Ok(0) => false,
        Ok(_) => {
            open.pending = Some(byte[0]);
            true
        }
        Err(_) => true,
    }
}

/// One line from `position`, and the position after it. The line drops its
/// terminating `\n` and the `\r` immediately before that -- a bare `\r` is
/// data (`SysFile::gets`) -- and a final unterminated line is still a line.
/// `None` at the end of the file. A transient stream is read a byte at a
/// time, so nothing after the line end is taken from it.
fn read_line_from(
    open: &mut rexx_core::OpenFile,
    position: u64,
) -> std::io::Result<Option<(Vec<u8>, i64)>> {
    let mut line = Vec::new();
    let mut at = position;
    let want = if open.transient { 1 } else { CHUNK };
    loop {
        let chunk = read_at(open, at, want)?;
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

/// Whether an open descriptor is a transient stream: a character device
/// or a FIFO, which is what `QUERY STREAMTYPE` answers `TRANSIENT` for
/// (`SysFile::getStreamTypeInfo`). A regular file is persistent.
fn is_transient(file: &std::fs::File) -> bool {
    use std::os::unix::fs::FileTypeExt;
    file.metadata()
        .is_ok_and(|meta| meta.file_type().is_char_device() || meta.file_type().is_fifo())
}

/// Whether an open descriptor is a device, as `SysFile::getStreamTypeInfo`
/// decides it: a character or block device.
fn is_device(file: &std::fs::File) -> bool {
    use std::os::unix::fs::FileTypeExt;
    file.metadata()
        .is_ok_and(|meta| meta.file_type().is_char_device() || meta.file_type().is_block_device())
}

/// Opens the stream if nothing has, the way an implicit open does: read-write
/// first, then write-only or read-only depending on what the caller wants
/// (`StreamInfo::implicitOpen`). A read never creates the file. `answer` is
/// what the caller returns when the open fails.
fn ensure_open(
    interp: &mut Interp,
    receiver: ObjRef,
    for_write: bool,
    answer: &[u8],
) -> Result<bool, Failure> {
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
            Err(error) => return fail_open(interp, receiver, &error, answer),
        },
        Err(_) => match std::fs::OpenOptions::new().read(true).open(&path) {
            Ok(file) => (
                file,
                rexx_core::OpenMode {
                    read_only: true,
                    ..rexx_core::OpenMode::default()
                },
            ),
            Err(error) => return fail_open(interp, receiver, &error, answer),
        },
    };
    // `SysFile::open` closes a directory it opened and fails with ENOENT.
    if opened.metadata().is_ok_and(|meta| meta.is_dir()) {
        let error = std::io::Error::from_raw_os_error(ENOENT);
        return fail_open(interp, receiver, &error, answer);
    }
    let size = size_of(&opened);
    let transient = is_transient(&opened);
    // A stream that can write writes at the end, whichever operation opened
    // it. A read-only or transient one keeps `implicitOpen`'s character 1.
    let write_position = if mode.read_only || transient {
        1
    } else {
        size as i64 + 1
    };
    let state = require_mut(interp, receiver)?;
    state.mode = mode;
    state.status = StreamStatus::Ready;
    state.open = Some(rexx_core::OpenFile {
        file: opened,
        read_position: 1,
        write_position,
        pending: None,
        transient,
        line_mark: None,
        line_total: None,
        line_mark_digest: None,
        line_total_digest: None,
    });
    Ok(true)
}

/// Records an implicit open's failure on the state and answers "not open".
fn fail_open(
    interp: &mut Interp,
    receiver: ObjRef,
    error: &std::io::Error,
    answer: &[u8],
) -> Result<bool, Failure> {
    let errno = error.raw_os_error().unwrap_or(ENOENT);
    // Every entry point that opens implicitly raises when the open fails --
    // measured, `lines` and `chars` on a name nothing resolves to each fire a
    // `CALL ON NOTREADY` handler once, and both still answer 0.
    notready(
        interp,
        receiver,
        StreamStatus::Error(errno),
        answer.to_vec(),
    )?;
    Ok(false)
}

/// `StreamInfo::readSetup`: the implicit open, after which the stream is
/// `READY` until the read says otherwise.
fn read_setup(interp: &mut Interp, receiver: ObjRef, answer: &[u8]) -> Result<bool, Failure> {
    if !ensure_open(interp, receiver, false, answer)? {
        return Ok(false);
    }
    require_mut(interp, receiver)?.status = StreamStatus::Ready;
    Ok(true)
}

/// `StreamInfo::writeSetup`: the implicit open for writing, `NOTREADY` with
/// `ERROR:13` on a stream opened read-only, and `READY` otherwise.
fn write_setup(interp: &mut Interp, receiver: ObjRef, answer: &[u8]) -> Result<bool, Failure> {
    if !ensure_open(interp, receiver, true, answer)? {
        return Ok(false);
    }
    if require(interp, receiver)?.mode.read_only {
        notready(
            interp,
            receiver,
            StreamStatus::Error(EACCES),
            answer.to_vec(),
        )?;
        return Ok(false);
    }
    require_mut(interp, receiver)?.status = StreamStatus::Ready;
    Ok(true)
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
            return notready(
                interp,
                receiver,
                StreamStatus::Error(EACCES),
                residual.to_vec(),
            );
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

/// `stream_charin`: `length` characters from `start`, or from the read
/// position. A short read raises `NOTREADY` and answers what it got, and
/// leaves `NOTREADY:EOF` whether it got some or nothing. A `start` past the
/// end is `NOTREADY:EOF` before anything is read.
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
        let start = start.map(positive).transpose()?;
        let StandardStream::In = which else {
            return Ok(Some(interp.text(b"")));
        };
        if let Some(start) = start {
            match interp.input_seek_char(start) {
                None => return Err(Raised::transient_positioning().into()),
                Some(false) => {
                    return notready(interp, receiver, StreamStatus::NotReady, Vec::new());
                }
                Some(true) => {}
            }
        }
        let wanted = usize::try_from(length).unwrap_or(usize::MAX);
        // A zero-length ask reads nothing and leaves the state alone, as it
        // does on the file path: measured, `.stdin~charin( , 0)` answers the
        // null string and `~description` stays `READY:`, where treating it as
        // a read of nothing would leave `NOTREADY:EOF`.
        if wanted == 0 {
            return Ok(Some(interp.text(b"")));
        }
        let read = interp.input_bytes(wanted);
        // Asking for 50 of six bytes answers all six and leaves
        // `NOTREADY:EOF`, asking for exactly six stays `READY:`, and a further
        // ask reads past the end, which is `NOTREADY:EOF` too
        // (rexxref streamclasses.xml:1638-1652).
        let status = match read.len() {
            0 => StreamStatus::NotReady,
            got if got < wanted => StreamStatus::NotReady,
            _ => StreamStatus::Ready,
        };
        if status != StreamStatus::Ready {
            return notready(interp, receiver, status, read);
        }
        require_mut(interp, receiver)?.status = status;
        return Ok(Some(interp.text_built(read)));
    }
    if !read_setup(interp, receiver, b"")? {
        return Ok(Some(interp.text(b"")));
    }
    let state = require_mut(interp, receiver)?;
    let write_only = state.mode.write_only;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"")));
    };
    if let Some(start) = start {
        if open.transient {
            return Err(Raised::transient_positioning().into());
        }
        let start = as_position(positive(start)?);
        if start > size_of(&open.file) as i64 {
            return notready(interp, receiver, StreamStatus::NotReady, Vec::new());
        }
        open.read_position = start;
    }
    let at = open.read_position;
    let wanted = usize::try_from(length).unwrap_or(usize::MAX);
    if wanted == 0 {
        return Ok(Some(interp.text(b"")));
    }
    if write_only {
        return unreadable(interp, receiver);
    }
    // A read that fails reports its errno: measured, `EBADF` on a stream
    // opened write-only is `ERROR:9`.
    let read = match read_at(open, offset_of(at), wanted) {
        Ok(read) => read,
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            return notready(interp, receiver, StreamStatus::Error(errno), Vec::new());
        }
    };
    if read.is_empty() {
        // A read past the end is `NOTREADY:EOF`, as LINEIN's is
        // (rexxref streamclasses.xml:1638-1652).
        return notready(interp, receiver, StreamStatus::NotReady, Vec::new());
    }
    open.read_position = at + read.len() as i64;
    if read.len() < wanted {
        return notready(interp, receiver, StreamStatus::NotReady, read);
    }
    Ok(Some(interp.text_built(read)))
}

/// A read on a stream opened WRITE, which is write-only (rexxref
/// funct.xml:4302): nothing is read, and the stream raises `NOTREADY` in the
/// `ERROR` state with the errno a read of a write-only descriptor gets.
#[cold]
#[inline(never)]
fn unreadable(interp: &mut Interp, receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
    notready(interp, receiver, StreamStatus::Error(EBADF), Vec::new())
}

/// The errno of a read on a descriptor open only for writing.
const EBADF: i32 = 9;

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
        let line = line.map(positive).transpose()?;
        let StandardStream::In = which else {
            return Ok(Some(interp.text(b"")));
        };
        if let Some(line) = line
            && !interp.input_seek_line(line)
        {
            return Err(Raised::transient_positioning().into());
        }
    } else {
        if !read_setup(interp, receiver, b"")? {
            return Ok(Some(interp.text(b"")));
        }
        if let Some(line) = line
            && !write_only(interp, receiver)?
        {
            seek_to_line(interp, receiver, line)?;
        }
    }
    if count == 0 {
        return Ok(Some(interp.text(b"")));
    }
    if write_only(interp, receiver)? {
        return unreadable(interp, receiver);
    }
    if let Some(record_length) = record_length(interp, receiver)? {
        // The rest of the current record: a short one, or nothing at all, is
        // `NOTREADY:EOF` answering what it got.
        let read = read_record(interp, receiver, record_length)?;
        let wanted = record_remainder(read.from, record_length);
        if read.bytes.is_empty() {
            return notready(interp, receiver, past_the_end(read.errno), Vec::new());
        }
        if read.bytes.len() < wanted {
            return notready(interp, receiver, StreamStatus::NotReady, read.bytes);
        }
        return Ok(Some(interp.text_built(read.bytes)));
    }
    Ok(Some(match next_line(interp, receiver)? {
        Some(text) => interp.text_built(text),
        None => interp.text(b""),
    }))
}

/// Whether the stream is open WRITE, and so reads nothing.
fn write_only(interp: &Interp, receiver: ObjRef) -> Result<bool, Failure> {
    let state = require(interp, receiver)?;
    Ok(state.open.is_some() && state.mode.write_only)
}

/// The state a read that got nothing leaves: `NOTREADY` past the end
/// (rexxref streamclasses.xml:1638-1652), `ERROR` for a read that failed.
fn past_the_end(errno: i32) -> StreamStatus {
    if errno == 0 {
        StreamStatus::NotReady
    } else {
        StreamStatus::Error(errno)
    }
}

/// The record length of an open `BINARY` stream, `None` for a line stream.
fn record_length(interp: &Interp, receiver: ObjRef) -> Result<Option<u64>, Failure> {
    let state = require(interp, receiver)?;
    let record_based = state.open.is_some() && state.mode.record_based;
    Ok((record_based && state.mode.record_length > 0).then_some(state.mode.record_length))
}

/// What is left of the record holding `position`: a whole record unless a
/// character operation stopped inside one.
fn record_remainder(position: i64, record_length: u64) -> usize {
    let length = as_position(record_length);
    let left = length - (position - 1).rem_euclid(length);
    usize::try_from(left).unwrap_or(usize::MAX)
}

/// One record read from the read position.
struct RecordRead {
    /// The read position the record started at.
    from: i64,
    bytes: Vec<u8>,
    /// The errno of a read that failed, and 0 otherwise.
    errno: i32,
}

/// Reads the rest of the current record and moves the read position past
/// what it got. A failed read reads nothing and carries its errno.
fn read_record(
    interp: &mut Interp,
    receiver: ObjRef,
    record_length: u64,
) -> Result<RecordRead, Failure> {
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(RecordRead {
            from: 1,
            bytes: Vec::new(),
            errno: 0,
        });
    };
    let from = open.read_position;
    let wanted = record_remainder(from, record_length);
    let (bytes, errno) = match read_at(open, offset_of(from), wanted) {
        Ok(bytes) => (bytes, 0),
        Err(error) => (Vec::new(), error.raw_os_error().unwrap_or(0)),
    };
    open.read_position = from + bytes.len() as i64;
    Ok(RecordRead { from, bytes, errno })
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
                notready(interp, receiver, StreamStatus::NotReady, Vec::new())?;
                Ok(None)
            }
        };
    }
    if !ensure_open(interp, receiver, false, b"")? {
        return Ok(None);
    }
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(None);
    };
    let at = open.read_position;
    match read_line_from(open, offset_of(at)).unwrap_or(None) {
        Some((text, next)) => {
            open.read_position = next;
            Ok(Some(text))
        }
        None => {
            notready(interp, receiver, StreamStatus::NotReady, Vec::new())?;
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
    if standard_of(interp, receiver)?.is_none() && !read_setup(interp, receiver, b"")? {
        return Ok(None);
    }
    if write_only(interp, receiver)? {
        unreadable(interp, receiver)?;
        return Ok(None);
    }
    // A record stream appends a record at a time, a short last one included,
    // and ends on the read that gets nothing.
    if let Some(record_length) = record_length(interp, receiver)? {
        loop {
            let read = read_record(interp, receiver, record_length)?;
            if read.bytes.is_empty() {
                notready(interp, receiver, past_the_end(read.errno), Vec::new())?;
                return Ok(None);
            }
            let item = interp.text_built(read.bytes);
            interp.roots.activity_mut().push_temp(item);
            super::collection::append_slot(interp, array, item)?;
        }
    }
    while let Some(text) = next_line(interp, receiver)? {
        let item = interp.text_built(text);
        // Rooted before the append, which allocates when the array grows.
        interp.roots.activity_mut().push_temp(item);
        super::collection::append_slot(interp, array, item)?;
    }
    Ok(None)
}

/// Moves the read position to the start of `line` (`setLineReadPosition`).
fn seek_to_line(interp: &mut Interp, receiver: ObjRef, line: i64) -> Result<(), Failure> {
    let state = require_mut(interp, receiver)?;
    let record_length = if state.mode.record_based {
        state.mode.record_length
    } else {
        0
    };
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
    let target = as_position(positive(line)?);
    let (_, at) = seek_line(open, target, record_length);
    open.read_position = at;
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
        return close_for_write(interp, _cleared, receiver, b"0");
    }
    if let Some(which) = standard_of(interp, receiver)? {
        refuse_transient_position(interp, which, start.is_some())?;
        start.map(positive).transpose()?;
        let data = data.unwrap_or_default();
        let residual = data.len().to_string().into_bytes();
        return standard_write(interp, receiver, which, &data, &residual);
    }
    let residual = data.as_ref().map_or(0, Vec::len).to_string().into_bytes();
    if !write_setup(interp, receiver, &residual)? {
        return Ok(Some(interp.text_built(residual)));
    }
    let state = require_mut(interp, receiver)?;
    let append = state.mode.append;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    if let Some(start) = start {
        if open.transient {
            return Err(Raised::transient_positioning().into());
        }
        open.write_position = as_position(positive(start)?);
    }
    // A null string writes nothing, so it moves nothing either.
    let Some(data) = data.filter(|data| !data.is_empty()) else {
        return Ok(Some(interp.text(b"0")));
    };
    let at = landing(open, append, open.write_position);
    match write_open(open, at, &data) {
        Ok(()) => {
            open.write_position = at + data.len() as i64;
            Ok(Some(interp.text(b"0")))
        }
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            notready(interp, receiver, StreamStatus::Error(errno), residual)
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
        return close_for_write(interp, _cleared, receiver, b"1");
    }
    if let Some(which) = standard_of(interp, receiver)? {
        refuse_transient_position(interp, which, line.is_some())?;
        line.map(positive).transpose()?;
        let mut bytes = data.unwrap_or_default();
        bytes.push(b'\n');
        return standard_write(interp, receiver, which, &bytes, b"1");
    }
    if !write_setup(interp, receiver, b"1")? {
        return Ok(Some(interp.text(b"1")));
    }
    // Line positioning on a text stream open only for writing is an error
    // (rexxref funct.xml:4454-4455): nothing is written or moved. A record
    // stream's lines are character positions, which a binary stream may name
    // past its end (funct.xml:3203-3205).
    if line.is_some() && write_only(interp, receiver)? && record_length(interp, receiver)?.is_none()
    {
        return notready(interp, receiver, StreamStatus::NotReady, b"1".to_vec());
    }
    // Only a LINEOUT with a string pads its record. Positioning writes
    // nothing (Deviation 46: the oracle pads from the write position here).
    let record = record_length(interp, receiver)?;
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
        if open.transient {
            return Err(Raised::transient_positioning().into());
        }
        let line = as_position(positive(line)?);
        let (_, at) = seek_line(open, line, record_length);
        open.write_position = at;
    }
    let Some(data) = data else {
        return Ok(Some(interp.text(b"0")));
    };
    if let Some(record_length) = record {
        return write_record(interp, receiver, record_length, data);
    }
    // A ctrl-Z is data like any other byte: Unix-like systems use no
    // end-of-file character in text files (rexxref intro.xml:3322-3330).
    let at = landing(open, mode.append, open.write_position);
    let mut bytes = data;
    bytes.push(b'\n');
    match write_open(open, at, &bytes) {
        Ok(()) => {
            open.write_position = at + bytes.len() as i64;
            Ok(Some(interp.text(b"0")))
        }
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            notready(interp, receiver, StreamStatus::Error(errno), b"1".to_vec())
        }
    }
}

/// `LINEOUT` or `CHAROUT` with neither data nor a position: a stream opened
/// read-only is closed first, then the write setup runs -- creating a missing
/// file, or raising `NOTREADY` with `answer` where it cannot -- and the stream
/// is closed (`StreamInfo::lineout`, `StreamInfo::charout`). A close writes
/// nothing (Deviation 46: the oracle's `LINEOUT` pads a record stream's
/// current record from the write position first). A standard stream only
/// closes.
fn close_for_write(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    answer: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    if standard_of(interp, receiver)?.is_none() {
        let state = require_mut(interp, receiver)?;
        if state.open.is_some() && state.mode.read_only {
            state.open = None;
        }
        if !write_setup(interp, receiver, answer)? {
            return Ok(Some(interp.text_built(answer.to_vec())));
        }
    }
    close(interp, cleared, receiver, &[])?;
    Ok(Some(interp.text(b"0")))
}

/// `LINEOUT` on a record stream: `data` padded with blanks to the end of the
/// current record, and the bare 93 when it does not fit what is left of it
/// (`StreamInfo::writeFixedLine`).
fn write_record(
    interp: &mut Interp,
    receiver: ObjRef,
    record_length: u64,
    mut data: Vec<u8>,
) -> Result<Option<ObjRef>, Failure> {
    let state = require_mut(interp, receiver)?;
    let append = state.mode.append;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"1")));
    };
    let at = landing(open, append, open.write_position);
    let room = record_remainder(at, record_length);
    if data.len() > room {
        return Err(Raised::syntax(93, 0, Vec::new()).into());
    }
    data.resize(room, b' ');
    match write_open(open, at, &data) {
        Ok(()) => {
            open.write_position = at + data.len() as i64;
            Ok(Some(interp.text(b"0")))
        }
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            notready(interp, receiver, StreamStatus::Error(errno), b"1".to_vec())
        }
    }
}

/// Where a write from `position` lands: at the end of the file on an APPEND
/// stream, whose write position cannot move into the file as it was opened
/// (rexxref funct.xml:4309-4312), and at `position` otherwise.
fn landing(open: &rexx_core::OpenFile, append: bool, position: i64) -> i64 {
    if append && !open.transient {
        size_of(&open.file) as i64 + 1
    } else {
        position
    }
}

/// Writes `bytes` at `at` (1-based) of an open stream, dropping the line
/// count and any line mark the write lands at or before. The marks rest on
/// rexxref's condition for its positioning optimizations, that nothing else
/// writes the file meanwhile (iostrms.xml:495-500).
fn write_open(open: &mut rexx_core::OpenFile, at: i64, bytes: &[u8]) -> std::io::Result<()> {
    open.line_total = None;
    open.line_total_digest = None;
    if open.line_mark.is_some_and(|(mark, _)| at <= mark) {
        open.line_mark = None;
        open.line_mark_digest = None;
    }
    write_at(&mut open.file, offset_of(at), bytes, open.transient)
}

/// Writes `bytes` at `position` (1-based). A transient stream has no
/// position, so it is written where it stands.
fn write_at(
    file: &mut std::fs::File,
    position: u64,
    bytes: &[u8],
    transient: bool,
) -> std::io::Result<()> {
    use std::io::{Seek, SeekFrom, Write};
    if !transient {
        file.seek(SeekFrom::Start(position.saturating_sub(1)))?;
    }
    file.write_all(bytes)
}

/// `stream_chars`: the characters left from the read position, which is the
/// size less what has been read, floored at zero. A stream that cannot read
/// answers 0. A transient stream answers 1 when data waits and 0 when none
/// does (rexxref iostrms.xml:189-193).
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
    if !ensure_open(interp, receiver, false, b"0")? {
        return Ok(Some(interp.text(b"0")));
    }
    let state = require_mut(interp, receiver)?;
    let readable = state.mode.read_only || state.mode.read_write;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    if !readable {
        return Ok(Some(interp.text(b"0")));
    }
    if open.transient {
        let waiting = transient_has_data(open);
        return Ok(Some(interp.text(if waiting { b"1" } else { b"0" })));
    }
    let left = (size_of(&open.file) as i64 - (open.read_position - 1)).max(0);
    Ok(Some(interp.text_built(left.to_string().into_bytes())))
}

/// `stream_lines`: `Count` -- the method's default -- answers the lines left
/// after the read position, `Normal` answers 1 or 0.
pub(super) fn lines(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // `stream_lines`: the first letter only, and anything but `N` or `C` is
    // 93.915.
    let quick = match args.first().copied().flatten() {
        Some(value) => {
            let text = interp.to_text(value).into_owned();
            match text.first().map(u8::to_ascii_uppercase) {
                Some(b'N') => true,
                Some(b'C') => false,
                _ => return Err(Raised::method_option_not_recognised("CN", &text).into()),
            }
        }
        None => false,
    };
    if let Some(which) = standard_of(interp, receiver)? {
        let StandardStream::In = which else {
            return Ok(Some(interp.text(b"0")));
        };
        let left = interp.input_remaining_lines(!quick).unwrap_or(1);
        let answer = if quick { u64::from(left > 0) } else { left };
        return Ok(Some(interp.text_built(answer.to_string().into_bytes())));
    }
    if !ensure_open(interp, receiver, false, b"0")? {
        return Ok(Some(interp.text(b"0")));
    }
    let record_length = record_length(interp, receiver)?;
    let state = require_mut(interp, receiver)?;
    let readable = state.mode.read_only || state.mode.read_write;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    // A stream that cannot read has no lines (`StreamInfo::lines`), and a
    // transient stream answers whether data waits, as `chars` does.
    if !readable {
        return Ok(Some(interp.text(b"0")));
    }
    if open.transient {
        let waiting = transient_has_data(open);
        return Ok(Some(interp.text(if waiting { b"1" } else { b"0" })));
    }
    let size = size_of(&open.file) as i64;
    if open.read_position > size {
        return Ok(Some(interp.text(b"0")));
    }
    // A record stream counts the records left, a partial last one included,
    // and answers that count for `Normal` too.
    if let Some(record_length) = record_length {
        let length = as_position(record_length);
        let records = size / length + i64::from(size % length != 0);
        let answer = records - (open.read_position - 1) / length;
        return Ok(Some(interp.text_built(answer.to_string().into_bytes())));
    }
    if quick {
        return Ok(Some(interp.text(b"1")));
    }
    // The lines that remain, counted each time (rexxref funct.xml:3306-3307).
    // The count is a read, so it leaves the stream `READY`
    // (`StreamInfo::lines` calls `readSetup` before it scans).
    let answer = lines_after(open, open.read_position).unwrap_or(0);
    state.status = StreamStatus::Ready;
    Ok(Some(interp.text_built(answer.to_string().into_bytes())))
}

/// An optional position argument, marshalled as the native boundary's
/// `int64_t`: anything that is not a whole number in its range is 88.907
/// naming argument `which` (`NativeActivation::int64Value`).
fn optional_position(
    interp: &mut Interp,
    value: Option<ObjRef>,
    which: usize,
) -> Result<Option<i64>, Failure> {
    let Some(value) = value else {
        return Ok(None);
    };
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => {
            let text = interp.to_text(value);
            Number::parse_bytes(&text).and_then(|number| number.int64_value(DIGITS64))
        }
    };
    match number {
        Some(number) => Ok(Some(number)),
        None => {
            let found = interp.string_value_text(value);
            let mut raised = Raised::native_argument_out_of_range(
                &format!("{which}"),
                i64::MIN,
                i64::MAX,
                &found,
            );
            marshalling(&mut raised);
            Err(raised.into())
        }
    }
}

/// A marshalling raise is reported against the REXX package with no line:
/// measured, oracle, `.Stream~new('f.txt')~lineIn(.nil)` is `Error 88 running
/// REXX:`.
fn marshalling(raised: &mut Raised) {
    raised.delivery.lineless = true;
    raised.delivery.internal_package = true;
}

/// A position below 1 is 93.907, which names argument 1 whichever argument
/// carried it (`StreamInfo::setCharReadPosition` and its siblings).
fn positive(position: i64) -> Result<u64, Failure> {
    u64::try_from(position)
        .ok()
        .filter(|position| *position != 0)
        .ok_or_else(|| {
            Raised::method_argument_not_positive(1, position.to_string().as_bytes()).into()
        })
}

/// A whole-number argument that may be zero, marshalled as the native
/// boundary's `size_t` (`NativeActivation::unsignedIntegerValue`).
fn whole_number(interp: &mut Interp, value: ObjRef, which: usize) -> Result<u64, Failure> {
    let number = match value.decode() {
        Decoded::SmallInt(number) => u64::try_from(number).ok(),
        _ => {
            let text = interp.to_text(value);
            Number::parse_bytes(&text).and_then(|number| number.unsigned_int64_value(DIGITS64))
        }
    };
    match number {
        Some(number) => Ok(number),
        None => {
            let found = interp.string_value_text(value);
            let mut raised = Raised::native_argument_out_of_range_unsigned(
                &format!("{which}"),
                0,
                u64::MAX,
                &found,
            );
            marshalling(&mut raised);
            Err(raised.into())
        }
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
            // The fallback token: an offset, once, digits only and within
            // `int64_t` -- which is why `seek 1e1`, `seek =1.5` and
            // `seek 9223372036854775808` are the bare 93.
            match to_number(token).and_then(|number| i64::try_from(number).ok()) {
                Some(number) => parsed.offset = Some(number),
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
/// position and answers where it landed. A position past the end is
/// accepted; one below 1 is not ([`invalid_seek`]).
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
    // it admits both, moving from the read position. An unopened stream
    // admits neither and so takes the both branch -- measured, a seek on one
    // moves both pointers to the target.
    if !parsed.read && !parsed.write {
        if mode.read_only {
            parsed.read = true;
        } else if mode.write_only {
            parsed.write = true;
        } else {
            parsed.read = true;
            parsed.write = true;
        }
    }
    // A READ stream has no write position to move (rexxref funct.xml:4299).
    if parsed.write && mode.read_only {
        return invalid_seek(interp, receiver);
    }
    // The implicit open comes after the flags and never creates the file: a
    // seek naming one that does not exist answers 0 and leaves state ERROR.
    // Asking for the read-shaped open is what selects that, because it is the
    // `operation_nocreate` one whichever pointer is about to move.
    if !ensure_open(interp, receiver, false, b"0")? {
        return Ok(Some(interp.text(b"0")));
    }
    // The pointers were chosen by the mode before the open, and everything
    // below by the mode the open left.
    let mode = require(interp, receiver)?.mode;
    let record_length = if mode.record_based {
        mode.record_length
    } else {
        0
    };
    if !parsed.by_char && !parsed.by_line {
        parsed.by_char = true;
    }
    let offset = if parsed.style == SeekFrom::Backward {
        -offset
    } else {
        offset
    };
    let state = require_mut(interp, receiver)?;
    let Some(open) = state.open.as_mut() else {
        return Ok(Some(interp.text(b"0")));
    };
    let from = if parsed.read {
        open.read_position
    } else {
        open.write_position
    };
    let size = size_of(&open.file) as i64;
    if parsed.by_char {
        // `+` and `-` move from the position the command names
        // (Deviation 49).
        let landed = match parsed.style {
            SeekFrom::Start => offset,
            SeekFrom::End => size - offset + 1,
            SeekFrom::Forward | SeekFrom::Backward => from + offset,
        };
        if landed < 1 {
            return invalid_seek(interp, receiver);
        }
        if parsed.read {
            open.read_position = landed;
        }
        if parsed.write {
            open.write_position = landed;
        }
        return Ok(Some(interp.text_built(landed.to_string().into_bytes())));
    }
    // Line positioning on a text stream open only for writing is an error
    // (rexxref funct.xml:4454-4455); a record stream's lines are character
    // positions.
    if !(mode.read_write || mode.read_only) && record_length == 0 {
        return invalid_seek(interp, receiver);
    }
    let lines = match parsed.style {
        SeekFrom::Start => Ok(offset),
        SeekFrom::End => line_count(open, record_length).map(|count| as_position(count) - offset),
        SeekFrom::Forward | SeekFrom::Backward => {
            line_of(open, from, record_length).map(|current| current + offset)
        }
    };
    let target = match lines {
        Ok(target) => target,
        Err(error) => {
            let errno = error.raw_os_error().unwrap_or(0);
            return notready(interp, receiver, StreamStatus::Error(errno), b"0".to_vec());
        }
    };
    if target < 1 {
        return invalid_seek(interp, receiver);
    }
    let (line, at) = seek_line(open, target, record_length);
    if parsed.read {
        open.read_position = at;
    }
    if parsed.write {
        open.write_position = at;
    }
    Ok(Some(interp.text_built(line.to_string().into_bytes())))
}

/// A SEEK to a position the stream does not have: below 1, the write
/// position of a READ stream, or a line of a stream open only for writing.
/// It answers 0, raises `NOTREADY`, and moves nothing.
#[cold]
#[inline(never)]
fn invalid_seek(interp: &mut Interp, receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
    notready(interp, receiver, StreamStatus::NotReady, b"0".to_vec())
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
    if let Some(object) = interp.heap.get_mut(receiver) {
        let before = object.body.held_bytes();
        if let Body::Instance { native, .. } = &mut object.body {
            *native = None;
        }
        let after = object.body.held_bytes();
        interp.heap.rehold_body_bytes(before, after);
    }
    Ok(None)
}
