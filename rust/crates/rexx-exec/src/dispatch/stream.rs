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
        return Ok(Some(
            interp.text_built(format!("ERROR:{ENOENT}").into_bytes()),
        ));
    }
    let size = metadata.map_or(0, |meta| meta.len());
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
    let write_position = if parsed.read_only { 0 } else { size + 1 };

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
        transient: false,
    });
    state.status = StreamStatus::Ready;
    Ok(Some(interp.text(b"READY:")))
}

/// `stream_close`: `READY:` for a stream that was open, and the null string
/// for one that never was. Either way the state goes back to `UNKNOWN`.
pub(super) fn close(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = require_mut(interp, receiver)?;
    let was_open = state.open.take().is_some();
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
