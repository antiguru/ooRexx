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

//! The `REXXUTIL` message routines: `getErrorMessage` and the `SysGetMessage`
//! pair in `interpreter/platform/unix/SysRexxUtil.cpp`, `formatMessage` in
//! `runtime/RexxUtilCommon.cpp`, and `SysGetErrorText`. A catalog other than
//! the interpreter's own is read as glibc 2.43's `catopen` and `catgets` read
//! it (`catgets/catgets.c`, `catgets/open_catalog.c`).

use std::io::Read as _;

use rexx_core::ObjRef;

use super::stems::positive;
use super::{
    c_string, check_arity, int_argument, marshalled, optional_text, required, required_c_string,
};
use crate::error::Raised;
use crate::{Failure, Interp};

/// The catalog name that means the interpreter's own messages.
const REXX_CATALOG: &[u8] = b"rexx.cat";

const MESSAGE_NOT_FOUND: &[u8] = b"Error: Message not found";

const CATALOG_NOT_FOUND: &[u8] = b"Error: Message catalog not found";

/// The most substitution values `formatMessage` takes.
const SUBSTITUTION_LIMIT: usize = 9;

/// The `Message_map_table` bound: messages whose major code is below this
/// have a message number `RexxGetErrorMessageByNumber` finds.
const MAPPED_MAJOR_LIMIT: u16 = 102;

/// `SysGetMessage(number [, catalog] [, substitution...])`: message `number`
/// of set 1.
pub(crate) fn get_message(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let number = positive(interp, args, 1)?;
    let catalog = optional_text(interp, args, 2)?;
    let message = error_message(interp, catalog.as_deref(), 1, native_int(number));
    format_message(interp, &message, Substitutions::Arguments(args), 3)
}

/// `SysGetMessageX(set, number [, catalog] [, substitution...])`.
pub(crate) fn get_message_x(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let set = positive(interp, args, 1)?;
    let number = positive(interp, args, 2)?;
    let catalog = optional_text(interp, args, 3)?;
    let message = error_message(
        interp,
        catalog.as_deref(),
        native_int(set),
        native_int(number),
    );
    format_message(interp, &message, Substitutions::Arguments(args), 4)
}

/// `SysFormatMessage(message, substitutions)`: `&1` to `&9` in `message`
/// replaced by the array's items.
pub(crate) fn format(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 2)?;
    let message = required_c_string(interp, args, 1)?;
    let array = substitution_array(interp, required(args, 2)?)?;
    format_message(interp, &message, Substitutions::Array(array), 1)
}

/// `SysGetErrorText(errno)`: `strerror`'s text.
pub(crate) fn error_text(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let number = int_argument(interp, required(args, 1)?, 1)?;
    let number = i32::try_from(number).expect("int_argument answers an int");
    Ok(interp.text_built(strerror(number)))
}

/// glibc's `strerror` text for `number`, as the standard library's error
/// display renders it before its own ` (os error N)` suffix.
fn strerror(number: i32) -> Vec<u8> {
    let rendered = std::io::Error::from_raw_os_error(number).to_string();
    let suffix = format!(" (os error {number})");
    rendered
        .strip_suffix(&suffix)
        .unwrap_or(&rendered)
        .as_bytes()
        .to_vec()
}

/// A positive whole-number argument as the C++ passes it on, to an `int`.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the oracle truncates: SysGetMessage(4294967301) is message 5"
)]
fn native_int(value: i64) -> i32 {
    value as i32
}

/// `getErrorMessage`: the interpreter's own message where `catalog` is
/// omitted or is `rexx.cat`, else the catalog's.
fn error_message(interp: &Interp, catalog: Option<&[u8]>, set: i32, number: i32) -> Vec<u8> {
    match catalog.map(c_string) {
        None | Some(REXX_CATALOG) => {
            let found = if set == 1 { rexx_message(number) } else { None };
            found.map_or(MESSAGE_NOT_FOUND, str::as_bytes).to_vec()
        }
        Some(name) => catalog_message(interp, name, set, number),
    }
}

/// `RexxGetErrorMessageByNumber`: the text of the message whose message
/// number is `number`.
fn rexx_message(number: i32) -> Option<&'static str> {
    rexx_inventory::errors::MESSAGES
        .iter()
        .find(|message| message.major < MAPPED_MAJOR_LIMIT && i32::from(message.number) == number)
        .map(|message| message.text)
}

/// Where `formatMessage` reads its substitution values.
enum Substitutions<'a> {
    /// The routine's own arguments.
    Arguments(&'a [Option<ObjRef>]),
    /// An array, read item by item as each is converted, since a conversion
    /// can change it.
    Array(ObjRef),
}

impl Substitutions<'_> {
    /// `ArraySize`.
    fn size(&self, interp: &Interp) -> usize {
        match self {
            Substitutions::Arguments(args) => args.len(),
            Substitutions::Array(array) => interp.array_slots(*array).map_or(0, <[_]>::len),
        }
    }

    /// `ArrayAt(index)`, one-based.
    fn at(&self, interp: &Interp, index: usize) -> Option<ObjRef> {
        match self {
            Substitutions::Arguments(args) => args.get(index - 1).copied().flatten(),
            Substitutions::Array(array) => interp
                .array_slots(*array)
                .and_then(|slots| slots.get(index - 1).copied().flatten()),
        }
    }
}

/// `formatMessage`: `message` with `&1` to `&9` replaced by the
/// substitution values from position `first` on, a reference past them
/// replaced by nothing. 40.1 where there are more than nine values.
///
/// Every value is converted before the message is read, an omitted one as
/// the null string. A conversion that raises does not stop the others, and
/// the last raise is the routine's answer. The oracle reads a raising value
/// that the message refers to as a null pointer and dies
/// (`corpus/oracle-crashes.txt` entry 41).
fn format_message(
    interp: &mut Interp,
    message: &[u8],
    substitutions: Substitutions<'_>,
    first: usize,
) -> Result<ObjRef, Failure> {
    let size = substitutions.size(interp);
    let count = (size + 1).saturating_sub(first);
    if count > SUBSTITUTION_LIMIT {
        return Err(Raised::routine_failed(b"SysFormatMessage").into());
    }
    let mut values = Vec::with_capacity(count);
    let mut raised = None;
    for index in first..=size {
        let text = match substitutions.at(interp, index) {
            None => Ok(Vec::new()),
            Some(value) => substitution_text(interp, value),
        };
        values.push(text.unwrap_or_else(|failure| {
            raised = Some(failure);
            Vec::new()
        }));
    }
    if let Some(failure) = raised {
        return Err(failure);
    }
    Ok(interp.text_built(substitute(message, &values)))
}

/// `message` with each `&` and digit from `1` to `9` replaced by that
/// value, or by nothing where there is none.
fn substitute(message: &[u8], values: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::with_capacity(message.len());
    let mut at = 0;
    while at < message.len() {
        match (message[at], message.get(at + 1)) {
            (b'&', Some(digit @ b'1'..=b'9')) => {
                if let Some(value) = values.get(usize::from(digit - b'1')) {
                    out.extend_from_slice(value);
                }
                at += 2;
            }
            (byte, _) => {
                out.push(byte);
                at += 1;
            }
        }
    }
    out
}

/// `ObjectToStringValue`: the value's requested string, as the C string the
/// C++ measures with `strlen`. A string request answering `.nil` is read as
/// the null string, measured.
fn substitution_text(interp: &mut Interp, value: ObjRef) -> Result<Vec<u8>, Failure> {
    let string = interp.required_string_or_nil(value)?;
    if string == ObjRef::NIL {
        return Ok(Vec::new());
    }
    Ok(c_string(&interp.to_text(string)).to_vec())
}

/// A `RexxArrayObject` argument: the array itself, or what `requestArray`
/// answers, rooted. 98.913 where that is not a single-dimensional array.
fn substitution_array(interp: &mut Interp, value: ObjRef) -> Result<ObjRef, Failure> {
    let array = if interp.array_slots(value).is_some() {
        Some(value)
    } else {
        interp.request_array_value(value)?
    };
    if let Some(array) = array {
        interp.roots.activity_mut().push_temp(array);
        if matches!(
            interp.array_body(array),
            Some((_, dimensions)) if dimensions.is_none_or(|shape| shape.len() == 1)
        ) {
            return Ok(array);
        }
    }
    let found = interp.string_value_text(value);
    Err(marshalled(Raised::native_argument_not_an_array(&found)))
}

/// glibc's catalog search path where `NLSPATH` is unset or empty, and what
/// it appends to one that is set (`catgets/Makefile`, `localedir`
/// `/usr/share/locale`).
const DEFAULT_NLSPATH: &[u8] = b"/usr/share/locale/%L/%N:/usr/share/locale/%L/LC_MESSAGES/%N:\
/usr/share/locale/%l/%N:/usr/share/locale/%l/LC_MESSAGES/%N:";

/// `CATGETS_MAGIC`.
const CATALOG_MAGIC: u32 = 0x9604_08de;

/// `sizeof (struct catalog_obj)`: the magic and the table's two dimensions.
const HEADER_BYTES: usize = 12;

/// `catopen(name, 0)` and `catgets(set, number)`: the message, or the
/// oracle's text for a catalog that does not open or a message it does not
/// hold. An empty message reads as one that is not there.
///
/// Where `catgets` answers its default the C++ reads `errno` and answers
/// "Error: Invalid message catalog" for `EBADF`. glibc's `catgets` sets
/// `ENOMSG` or nothing, so that text answers only to a stale `errno`, and
/// here it is never the answer.
fn catalog_message(interp: &Interp, name: &[u8], set: i32, number: i32) -> Vec<u8> {
    let Some(catalog) = open_catalog(interp, name).and_then(Catalog::new) else {
        return CATALOG_NOT_FOUND.to_vec();
    };
    match catalog.message(set, number) {
        Some(text) if !text.is_empty() => text,
        _ => MESSAGE_NOT_FOUND.to_vec(),
    }
}

/// An environment variable as `getenv` answers it.
fn env_c_string<'a>(interp: &'a Interp, name: &[u8]) -> Option<&'a [u8]> {
    interp.env_get(name).map(c_string)
}

/// `catopen`'s file: `name` itself where it holds a `/`, else the first
/// element of the search path that opens. The file's bytes, or `None` where
/// nothing opens or what opens is no catalog file of at least a header's
/// size.
fn open_catalog(interp: &Interp, name: &[u8]) -> Option<CatalogImage> {
    let file = if name.contains(&b'/') {
        open(interp, name)?
    } else {
        let locale = env_c_string(interp, b"LANG")
            .filter(|value| !value.is_empty())
            .unwrap_or(b"C");
        let search = match env_c_string(interp, b"NLSPATH") {
            Some(path) if !path.is_empty() => [path, b":", DEFAULT_NLSPATH].concat(),
            _ => DEFAULT_NLSPATH.to_vec(),
        };
        search_path(interp, &search, name, locale)?
    };
    CatalogImage::read(file)
}

/// `open(path, O_RDONLY)`, a relative path resolved against the program's
/// directory.
fn open(interp: &Interp, path: &[u8]) -> Option<std::fs::File> {
    if path.is_empty() {
        return None;
    }
    let full = if path[0] == b'/' {
        path.to_vec()
    } else {
        [interp.cwd_bytes(), b"/", path].concat()
    };
    std::fs::File::open(super::os_path(&full)).ok()
}

/// `__open_catalog`'s walk of the search path `path`: the first candidate
/// that opens is the file, whatever it is.
fn search_path(interp: &Interp, path: &[u8], name: &[u8], locale: &[u8]) -> Option<std::fs::File> {
    search_candidates(path, name, locale)
        .iter()
        .find_map(|candidate| open(interp, c_string(candidate)))
}

/// The search path's elements expanded, in order, an element that expands to
/// nothing left out: `%N` the name, `%L` the locale, `%l`, `%t` and `%c` its
/// language, territory and code set, `%%` a `%`, any other `%` code dropping
/// the element, an empty element the name alone. A territory or code set
/// that ends at its separator expands to a NUL, which ends the path `open`
/// sees.
fn search_candidates(path: &[u8], name: &[u8], locale: &[u8]) -> Vec<Vec<u8>> {
    let byte = |at: usize| path.get(at).copied().unwrap_or(0);
    let locale_byte = |at: usize| locale.get(at).copied().unwrap_or(0);
    let mut candidates = Vec::new();
    let mut at = 0;
    while byte(at) != 0 {
        let mut candidate = Vec::new();
        if byte(at) == b':' {
            candidate.extend_from_slice(name);
        } else {
            while byte(at) != b':' && byte(at) != 0 {
                if byte(at) != b'%' {
                    candidate.push(byte(at));
                    at += 1;
                    continue;
                }
                let code = byte(at + 1);
                at += 2;
                match code {
                    b'N' => candidate.extend_from_slice(name),
                    b'L' => candidate.extend_from_slice(locale),
                    b'l' => {
                        let mut from = 0;
                        loop {
                            candidate.push(locale_byte(from));
                            from += 1;
                            if matches!(locale_byte(from), 0 | b'_' | b'.') {
                                break;
                            }
                        }
                    }
                    b't' => {
                        let mut from = 1;
                        while !matches!(locale_byte(from), 0 | b'_' | b'.') {
                            from += 1;
                        }
                        if locale_byte(from) == b'_' {
                            loop {
                                from += 1;
                                candidate.push(locale_byte(from));
                                if matches!(locale_byte(from + 1), 0 | b'.') {
                                    break;
                                }
                            }
                        }
                    }
                    b'c' => {
                        let mut from = 1;
                        while !matches!(locale_byte(from), 0 | b'.') {
                            from += 1;
                        }
                        if locale_byte(from) == b'.' {
                            loop {
                                from += 1;
                                candidate.push(locale_byte(from));
                                if locale_byte(from + 1) == 0 {
                                    break;
                                }
                            }
                        }
                    }
                    b'%' => candidate.push(b'%'),
                    _ => {
                        candidate.clear();
                        while byte(at) != 0 && byte(at) != b':' {
                            at += 1;
                        }
                    }
                }
            }
        }
        if !candidate.is_empty() {
            candidates.push(candidate);
        }
        at += 1;
    }
    candidates
}

/// A catalog file as the oracle maps it: its bytes, then zeros to the end of
/// its last page. Past that the oracle faults, and a read there answers
/// `None`.
struct CatalogImage {
    bytes: Vec<u8>,
    mapped: usize,
}

impl CatalogImage {
    /// The file's contents, or `None` where it is not a regular file of at
    /// least [`HEADER_BYTES`] or does not begin with either byte order of
    /// [`CATALOG_MAGIC`], which glibc rejects before anything else.
    fn read(mut file: std::fs::File) -> Option<CatalogImage> {
        let metadata = file.metadata().ok()?;
        let size = usize::try_from(metadata.len()).ok()?;
        if !metadata.is_file() || size < HEADER_BYTES {
            return None;
        }
        let mut magic = [0; 4];
        file.read_exact(&mut magic).ok()?;
        let magic = u32::from_ne_bytes(magic);
        if magic != CATALOG_MAGIC && magic != CATALOG_MAGIC.swap_bytes() {
            return None;
        }
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&magic.to_ne_bytes());
        file.read_to_end(&mut bytes).ok()?;
        bytes.truncate(size);
        let page = rustix::param::page_size();
        let mapped = size.div_ceil(page) * page;
        Some(CatalogImage { bytes, mapped })
    }

    fn byte(&self, at: usize) -> Option<u8> {
        match self.bytes.get(at) {
            Some(byte) => Some(*byte),
            None => (at < self.mapped).then_some(0),
        }
    }

    /// The host-order word at byte `at`.
    fn word(&self, at: usize) -> Option<u32> {
        let mut word = [0; 4];
        for (offset, byte) in word.iter_mut().enumerate() {
            *byte = self.byte(at.checked_add(offset)?)?;
        }
        Some(u32::from_ne_bytes(word))
    }

    /// The bytes from `at` to the next NUL.
    fn c_string(&self, mut at: usize) -> Option<Vec<u8>> {
        let mut text = Vec::new();
        loop {
            match self.byte(at)? {
                0 => return Some(text),
                byte => text.push(byte),
            }
            at = at.checked_add(1)?;
        }
    }
}

/// An opened catalog: a hash table of `plane_size` buckets `plane_depth`
/// deep, each entry a set number plus one, a message number, and the offset
/// of the message from `strings`.
struct Catalog {
    image: CatalogImage,
    plane_size: usize,
    plane_depth: usize,
    /// Byte offset of the table in host order.
    table: usize,
    /// Byte offset of the first message.
    strings: usize,
}

impl Catalog {
    /// `__open_catalog`'s checks after the magic, its arithmetic kept: the
    /// size test counts the tables' words as bytes, and the scan for the
    /// last message's NUL may run into the zeros past the file's end. `None`
    /// is a file glibc rejects, and also a table whose bounds overflow or
    /// that reaches past the mapping, where the oracle faults.
    fn new(image: CatalogImage) -> Option<Catalog> {
        let swapped = image.word(0)? != CATALOG_MAGIC;
        let header = |at| {
            image
                .word(at)
                .map(|word| if swapped { word.swap_bytes() } else { word })
        };
        let plane_size = usize::try_from(header(4)?).ok()?;
        let plane_depth = usize::try_from(header(8)?).ok()?;
        let words = plane_size.checked_mul(plane_depth)?.checked_mul(3)?;
        let table_bytes = words.checked_mul(4)?;
        let table = if cfg!(target_endian = "little") {
            HEADER_BYTES
        } else {
            HEADER_BYTES.checked_add(table_bytes)?
        };
        let strings = HEADER_BYTES.checked_add(table_bytes.checked_mul(2)?)?;
        let mut max_offset = 0;
        for entry in (2..words).step_by(3) {
            let offset = usize::try_from(image.word(table + entry * 4)?).ok()?;
            max_offset = max_offset.max(offset);
        }
        let size = image.bytes.len();
        let claimed = HEADER_BYTES
            .checked_add(words.checked_mul(2)?)?
            .checked_add(max_offset)?;
        if size <= claimed {
            return None;
        }
        let mut last = strings.checked_add(max_offset)?;
        let mut remaining = (size - HEADER_BYTES)
            .checked_add(words * 2)?
            .checked_add(max_offset)?;
        while image.byte(last)? != 0 {
            remaining -= 1;
            if remaining == 0 {
                return None;
            }
            last = last.checked_add(1)?;
        }
        Some(Catalog {
            image,
            plane_size,
            plane_depth,
            table,
            strings,
        })
    }

    /// `catgets`: the message, or `None` where the catalog holds none for
    /// `set` and `number`. `None` too where the oracle faults: a table of no
    /// buckets (`corpus/oracle-crashes.txt` entry 42) or a read past the
    /// mapping.
    fn message(&self, set: i32, number: i32) -> Option<Vec<u8>> {
        let set = set.wrapping_add(1);
        if set <= 0 || number < 0 || self.plane_size == 0 {
            return None;
        }
        let hash = isize::try_from(set.wrapping_mul(number))
            .ok()?
            .cast_unsigned();
        let stride = self.plane_size.checked_mul(3)?;
        let mut entry = (hash % self.plane_size) * 3;
        let mut depth = 0;
        loop {
            let word = |offset: usize| {
                let at = entry.checked_add(offset)?.checked_mul(4)?;
                self.image.word(self.table.checked_add(at)?)
            };
            if word(0)? == set.cast_unsigned() && word(1)? == number.cast_unsigned() {
                let offset = usize::try_from(word(2)?).ok()?;
                return self.image.c_string(self.strings.checked_add(offset)?);
            }
            entry = entry.checked_add(stride)?;
            depth += 1;
            if depth >= self.plane_depth {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests;
