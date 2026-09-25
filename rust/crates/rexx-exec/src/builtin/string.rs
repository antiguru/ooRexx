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

//! The string builtins.
//! ```text
//! say substr('abc',0,5,'xx')          40.23  (the pad, not the zero position)
//! say translate('abc',,,'$',0,'q')    40.12 argument 6  (not the zero start)
//! say verify('a','b','X','q')         40.12 argument 4  (not the bad option)
//! ```

use rexx_core::ObjRef;

use super::{
    Args, arg, buffer, count_of, length_of, optional_string, pad_byte, position_of,
    required_render, required_string, whole_number,
};
use crate::Interp;
use crate::error::{Failure, Raised};

/// What `STRIP` strips when no character set is given: blank and horizontal
/// tab (`RexxString::strip`, `classes/StringClassSub.cpp`). Measured, a tab
/// really is in the default set -- `strip('a'||'09'x||'b'||'09'x)` answers
/// `a<tab>b`, with the trailing tab gone.
const DEFAULT_STRIP_SET: &[u8] = b" \t";

/// `LENGTH(string)`: how many bytes the argument renders as.
pub(crate) fn length(interp: &mut Interp, _name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let value = arg(args, 1).expect("check_arity admitted LENGTH's one required argument");
    // The borrow of `interp` ends with this statement, which is what lets the
    // allocation below happen at all.
    let bytes = interp.text_len(value);
    Ok(interp.counted(bytes))
}

/// The upper-cased first letter of an option argument, checked against the
/// set the builtin accepts.
fn option_letter(option: Option<&[u8]>, valid: &str) -> Result<Option<u8>, Failure> {
    let Some(option) = option else {
        return Ok(None);
    };
    let letter = option.first().map(|byte| byte.to_ascii_uppercase());
    match letter {
        Some(letter) if letter == 0 || valid.as_bytes().contains(&letter) => Ok(Some(letter)),
        _ => Err(Raised::invalid_option(valid, option).into()),
    }
}

// ---- building results ----

/// `len` copies of `byte`, appended.
fn push_pad(out: &mut Vec<u8>, byte: u8, len: usize) {
    out.resize(out.len() + len, byte);
}

/// Room for `additional` more bytes in `out`, or the oracle's 5.1.
fn reserve(out: &mut Vec<u8>, additional: usize) -> Result<(), Raised> {
    out.try_reserve(additional)
        .map_err(|_| Raised::system_resources())
}

/// Deletes `range` bytes at 0-based `begin`, or everything from `begin` on
/// for `None`; a `begin` at or past the end deletes nothing.
pub(crate) fn delete_range(bytes: &mut Vec<u8>, begin: usize, range: Option<usize>) {
    if begin >= bytes.len() {
        return;
    }
    let end = match range {
        Some(range) => begin.saturating_add(range).min(bytes.len()),
        None => bytes.len(),
    };
    bytes.drain(begin..end);
}

// ---- the shared search primitives ----

/// The 1-based offset of the first `needle` at or after `start` that
/// `StringUtil::pos` finds within `range` bytes of `start`, or 0 for no match.
pub(crate) fn find_forward(haystack: &[u8], needle: &[u8], start: usize, range: usize) -> usize {
    // `haystack.len() - start` underflows for a start past the end, which is
    // exactly the case the guard below rejects; taking the saturating
    // difference first keeps the two independent.
    let range = range.min(haystack.len().saturating_sub(start));
    if start >= haystack.len() || needle.len() > range || needle.is_empty() {
        return 0;
    }
    let window = &haystack[start..start + range];
    // **The ends are compared before the whole**, and that is what the naive
    // form costs: `candidate == needle` on byte slices is a `memcmp` call, and
    // without a guard it is made at every position the needle could start at.
    // Measured with `perf` on `bench-programs/strings.rex`, whose loop runs
    // `pos` and `changestr` over a 43-byte haystack for a 3-byte needle,
    // `__memcmp_evex_movbe` was 4.98% of samples -- more than `pos` itself.
    let first = needle[0];
    let last = needle.len() - 1;
    // One past the last start at which the whole needle fits. The first byte
    // is found a word at a time ([`find_byte`]) and only a position holding it
    // is tested further.
    let starts = range - last;
    // Whether the scan below reached a first byte at all, which is the
    // question the overrun path asks. Recorded as the scan runs so that path
    // does not walk the same bytes again to answer it.
    let mut saw_first = false;
    let mut at = 0;
    while let Some(offset) = find_byte(&window[at..starts], first) {
        saw_first = true;
        let position = at + offset;
        if window[position + last] == needle[last]
            && window[position..position + needle.len()] == *needle
        {
            return start + position + 1;
        }
        at = position + 1;
    }

    // **The overrun the doc above describes, and the failure path is the only
    // one that pays for it.** The scan just made covered every start at which
    // the whole needle fits; this is the one position past them, and the
    // oracle reaches it only after its own scan has found the first byte among
    // those starts. `saw_first` records exactly that, so asking it is asking
    // whether the oracle's loop would have run at all.
    if !saw_first {
        return 0;
    }
    let over = start + range - last;
    // At most one byte past the haystack, since `range` is capped at what
    // follows `start` -- and that byte is the terminator this declines to
    // invent, so `get` answering `None` is the divergence the doc records and
    // not a bound being papered over.
    debug_assert!(over + needle.len() <= haystack.len() + 1);
    match haystack.get(over..over + needle.len()) {
        Some(candidate) if candidate == needle => over + 1,
        _ => 0,
    }
}

/// The index of the first `byte` in `hay`, or `None`.
pub(crate) fn find_byte(hay: &[u8], byte: u8) -> Option<usize> {
    const LOW: u64 = 0x0101_0101_0101_0101;
    const HIGH: u64 = 0x8080_8080_8080_8080;

    let (words, tail) = hay.as_chunks::<8>();
    let repeated = u64::from(byte) * LOW;
    for (index, word) in words.iter().enumerate() {
        let masked = u64::from_le_bytes(*word) ^ repeated;
        let hits = masked.wrapping_sub(LOW) & !masked & HIGH;
        if hits != 0 {
            return Some(index * 8 + (hits.trailing_zeros() as usize) / 8);
        }
    }
    tail.iter()
        .position(|&candidate| candidate == byte)
        .map(|at| words.len() * 8 + at)
}

/// The 1-based offset of the last `needle` that ends at or before `start`
/// and begins no earlier than `range` bytes before that end, or 0.
pub(crate) fn find_backward(haystack: &[u8], needle: &[u8], start: usize, range: usize) -> usize {
    find_backward_with(haystack, needle, start, range, <[u8]>::eq)
}

/// [`find_backward`] with ASCII case folded away, `StringUtil::caselessLastPos`
/// (`classes/support/StringUtil.cpp:418-478`).
pub(crate) fn caseless_find_backward(
    haystack: &[u8],
    needle: &[u8],
    start: usize,
    range: usize,
) -> usize {
    find_backward_with(haystack, needle, start, range, caseless_eq)
}

/// The backward search both spellings run, `matches` the only difference.
fn find_backward_with(
    haystack: &[u8],
    needle: &[u8],
    start: usize,
    range: usize,
    matches: impl Fn(&[u8], &[u8]) -> bool,
) -> usize {
    if needle.is_empty() || haystack.is_empty() || needle.len() > range {
        return 0;
    }
    let end = start.min(haystack.len());
    let range = range.min(end);
    let window = &haystack[end - range..end];
    if needle.len() > window.len() {
        return 0;
    }
    window
        .windows(needle.len())
        .rposition(|candidate| matches(candidate, needle))
        .map_or(0, |offset| end - range + offset + 1)
}

/// Whether two byte runs are equal with ASCII case folded away.
pub(crate) fn caseless_eq(left: &[u8], right: &[u8]) -> bool {
    left.eq_ignore_ascii_case(right)
}

/// The 1-based offset of the first `needle` at or after `start` and within
/// `range` bytes of it, with ASCII case folded away, or 0.
pub(crate) fn caseless_find_forward(
    haystack: &[u8],
    needle: &[u8],
    start: usize,
    range: usize,
) -> usize {
    let range = range.min(haystack.len().saturating_sub(start));
    if start >= haystack.len() || needle.len() > range || needle.is_empty() {
        return 0;
    }
    let window = &haystack[start..start + range];
    window
        .windows(needle.len())
        .position(|candidate| caseless_eq(candidate, needle))
        .map_or(0, |offset| start + offset + 1)
}

/// How many non-overlapping `needle`s `haystack` holds, stopping at `limit`.
pub(crate) fn count_occurrences(haystack: &[u8], needle: &[u8], limit: usize) -> usize {
    count_with(haystack, needle, limit, find_forward)
}

/// [`count_occurrences`] with ASCII case folded away,
/// `StringUtil::caselessCountStr` (`classes/support/StringUtil.cpp:1220`).
pub(crate) fn caseless_count_occurrences(haystack: &[u8], needle: &[u8], limit: usize) -> usize {
    count_with(haystack, needle, limit, caseless_find_forward)
}

/// The counting loop both spellings run, `find` the only difference.
fn count_with(
    haystack: &[u8],
    needle: &[u8],
    limit: usize,
    find: impl Fn(&[u8], &[u8], usize, usize) -> usize,
) -> usize {
    if needle.is_empty() || needle.len() > haystack.len() || limit == 0 {
        return 0;
    }
    let mut count = 0;
    let mut next = 0;
    while count < limit {
        let found = find(haystack, needle, next, haystack.len());
        if found == 0 {
            break;
        }
        next = found - 1 + needle.len();
        count += 1;
    }
    count
}

/// `StringUtil::makearray`'s default separator (`classes/support/StringUtil.cpp:545`):
/// the lines of `text`.
pub(crate) fn line_slices(text: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.iter().position(|byte| *byte == b'\n') {
        let (line, after) = rest.split_at(at);
        lines.push(line.strip_suffix(b"\r").unwrap_or(line));
        rest = &after[1..];
    }
    if !rest.is_empty() {
        lines.push(rest);
    }
    lines
}

/// `StringUtil::makearray` with an explicit separator
/// (`classes/support/StringUtil.cpp:552`-`:640`): the pieces of `text`
/// between occurrences of `separator`.
pub(crate) fn split_slices<'a>(text: &'a [u8], separator: &[u8]) -> Vec<&'a [u8]> {
    if separator.is_empty() {
        return text.chunks(1).collect();
    }
    // The C++ scans up to `start + length - sepSize + 1`, which for a
    // separator longer than the text lies behind the start and stops the scan
    // before its `memcmp` can read past the end.
    let limit = match text.len().checked_sub(separator.len()) {
        Some(room) => room + 1,
        None => 0,
    };
    let mut pieces = Vec::new();
    let mut start = 0usize;
    while start < limit {
        let Some(at) = (start..limit).find(|at| &text[*at..*at + separator.len()] == separator)
        else {
            break;
        };
        pieces.push(&text[start..at]);
        start = at + separator.len();
    }
    if start < text.len() {
        pieces.push(&text[start..]);
    }
    pieces
}

/// Whether `byte` is one of `set`'s.
fn in_set(byte: u8, set: &[u8]) -> bool {
    set.contains(&byte)
}

// ---- the builtins ----

/// `CENTER(string, length [,pad])`, and `CENTRE` under its other spelling.
pub(crate) fn center(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let width = whole_number(interp, name, args, 2)?.expect("check_arity admitted the width");
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');
    let width = length_of(width)?;
    match center_bytes(interp, &string, width, pad)? {
        Some(out) => Ok(interp.text_built(out)),
        None => Ok(interp.text_built(string)),
    }
}

/// `CENTER`'s answer once its two arguments are checked, shared with
/// `String~center` so the builtin and the method cannot come to disagree.
pub(crate) fn center_bytes(
    interp: &Interp,
    string: &[u8],
    width: usize,
    pad: u8,
) -> Result<Option<Vec<u8>>, Failure> {
    let len = string.len();
    if width == len {
        return Ok(None);
    }
    if width == 0 {
        return Ok(Some(Vec::new()));
    }
    let out = if width > len {
        let left = (width - len) / 2;
        let mut out = buffer(interp, width)?;
        push_pad(&mut out, pad, left);
        out.extend_from_slice(string);
        push_pad(&mut out, pad, width - len - left);
        out
    } else {
        string[(len - width) / 2..][..width].to_vec()
    };
    Ok(Some(out))
}

/// `LEFT(string, length [,pad])`: the leading `length` bytes, padded on the
/// right.
pub(crate) fn left(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let size = whole_number(interp, name, args, 2)?.expect("check_arity admitted the length");
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');
    let size = length_of(size)?;
    let out = left_bytes(interp, &string, size, pad)?;
    Ok(interp.text_built(out))
}

/// `LEFT`'s answer once its two arguments are checked, shared with
/// `String~left`.
pub(crate) fn left_bytes(
    interp: &Interp,
    string: &[u8],
    size: usize,
    pad: u8,
) -> Result<Vec<u8>, Failure> {
    if size == 0 {
        return Ok(Vec::new());
    }
    let kept = string.len().min(size);
    let mut out = buffer(interp, size)?;
    out.extend_from_slice(&string[..kept]);
    push_pad(&mut out, pad, size - kept);
    Ok(out)
}

/// `RIGHT(string, length [,pad])`: the trailing `length` bytes, padded on the
/// left.
pub(crate) fn right(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let size = whole_number(interp, name, args, 2)?.expect("check_arity admitted the length");
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');
    let size = length_of(size)?;
    match right_bytes(interp, &string, size, pad)? {
        Some(out) => Ok(interp.text_built(out)),
        None => Ok(interp.text_built(string)),
    }
}

/// `RIGHT`'s answer once its two arguments are checked, shared with
/// `String~right`.
pub(crate) fn right_bytes(
    interp: &Interp,
    string: &[u8],
    size: usize,
    pad: u8,
) -> Result<Option<Vec<u8>>, Failure> {
    if size == 0 {
        return Ok(Some(Vec::new()));
    }
    if size == string.len() {
        return Ok(None);
    }
    let kept = string.len().min(size);
    let mut out = buffer(interp, size)?;
    push_pad(&mut out, pad, size - kept);
    out.extend_from_slice(&string[string.len() - kept..]);
    Ok(Some(out))
}

/// `SUBSTR`'s body: `length` bytes of `string` from 0-based `start`, padded
/// with `pad`, appended to `out`; `None` is everything from `start` on.
pub(crate) fn substr_bytes(
    out: &mut Vec<u8>,
    string: &[u8],
    start: usize,
    length: Option<usize>,
    pad: u8,
) -> Result<(), Raised> {
    let available = string.len().saturating_sub(start);
    let length = length.unwrap_or(available);
    let kept = length.min(available);
    reserve(out, length)?;
    out.extend_from_slice(&string[start.min(string.len())..][..kept]);
    push_pad(out, pad, length - kept);
    Ok(())
}

/// `SUBSTR(string, n [,length] [,pad])`.
pub(crate) fn substr(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let start = whole_number(interp, name, args, 2)?.expect("check_arity admitted the position");
    let requested = whole_number(interp, name, args, 3)?;
    let pad = pad_byte(interp, name, args, 4)?.unwrap_or(b' ');
    let string = required_render(interp, args, 1);

    let start = position_of(start)? - 1;
    let length = requested.map(length_of).transpose()?;
    let mut out = interp.take_result_buffer();
    substr_bytes(&mut out, string.text(interp), start, length, pad)?;
    Ok(interp.text_built(out))
}

/// `DELSTR(string [,n] [,length])`.
pub(crate) fn delstr(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let mut string = required_string(interp, args, 1);
    let start = whole_number(interp, name, args, 2)?;
    let requested = whole_number(interp, name, args, 3)?;

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    };
    // Range-checked before the out-of-range start is answered, which is what
    // the oracle's own ordering does -- measured, `delstr('abc',9,-1)` is
    // 93.923 rather than the unchanged string.
    let deleted = requested.map(length_of).transpose()?;
    delete_range(&mut string, start - 1, deleted);
    Ok(interp.text_built(string))
}

/// `INSERT`'s body: `new`, padded or cut to `length` bytes, spliced into
/// `target` after its first `start` bytes and appended to `out`; `None` is
/// `new`'s own length.
pub(crate) fn insert_bytes(
    out: &mut Vec<u8>,
    target: &[u8],
    new: &[u8],
    start: usize,
    length: Option<usize>,
    pad: u8,
) -> Result<(), Raised> {
    let insert_len = length.unwrap_or(new.len());
    let (lead_pad, front, back) = if start == 0 {
        (0, 0, target.len())
    } else if start >= target.len() {
        (start - target.len(), target.len(), 0)
    } else {
        (0, start, target.len() - start)
    };
    let copied = new.len().min(insert_len);

    let total = target
        .len()
        .checked_add(insert_len)
        .and_then(|size| size.checked_add(lead_pad))
        .ok_or_else(Raised::system_resources)?;
    reserve(out, total)?;
    out.extend_from_slice(&target[..front]);
    push_pad(out, pad, lead_pad);
    out.extend_from_slice(&new[..copied]);
    push_pad(out, pad, insert_len - copied);
    out.extend_from_slice(&target[front..front + back]);
    Ok(())
}

/// `INSERT(new, target [,n] [,length] [,pad])`.
pub(crate) fn insert(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let new = required_string(interp, args, 1);
    let target = required_string(interp, args, 2);
    let start = whole_number(interp, name, args, 3)?;
    let requested = whole_number(interp, name, args, 4)?;
    let pad = pad_byte(interp, name, args, 5)?.unwrap_or(b' ');

    let start = match start {
        Some(value) => count_of(value, 2)?,
        None => 0,
    };
    let length = requested.map(length_of).transpose()?;
    let mut out = interp.take_result_buffer();
    insert_bytes(&mut out, &target, &new, start, length, pad)?;
    Ok(interp.text_built(out))
}

/// `OVERLAY`'s body: `new`, padded or cut to `length` bytes, written over
/// `target` from 0-based `start`, padding out to `start` first when that is
/// past the end, appended to `out`; `None` is `new`'s own length.
pub(crate) fn overlay_bytes(
    out: &mut Vec<u8>,
    target: &[u8],
    new: &[u8],
    start: usize,
    length: Option<usize>,
    pad: u8,
) -> Result<(), Raised> {
    let overlay_len = length.unwrap_or(new.len());
    let (copied, back_pad) = if overlay_len > new.len() {
        (new.len(), overlay_len - new.len())
    } else {
        (overlay_len, 0)
    };
    let front = start.min(target.len());
    let front_pad = start - front;
    let span_end = start.saturating_add(overlay_len);
    let back = target.len().saturating_sub(span_end);

    reserve(out, front + back + front_pad + overlay_len)?;
    out.extend_from_slice(&target[..front]);
    push_pad(out, pad, front_pad);
    out.extend_from_slice(&new[..copied]);
    push_pad(out, pad, back_pad);
    out.extend_from_slice(&target[target.len() - back..]);
    Ok(())
}

/// `OVERLAY(new, target [,n] [,length] [,pad])`.
pub(crate) fn overlay(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let new = required_string(interp, args, 1);
    let target = required_string(interp, args, 2);
    let start = whole_number(interp, name, args, 3)?;
    let requested = whole_number(interp, name, args, 4)?;
    let pad = pad_byte(interp, name, args, 5)?.unwrap_or(b' ');

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    } - 1;
    let length = requested.map(length_of).transpose()?;
    let mut out = interp.take_result_buffer();
    overlay_bytes(&mut out, &target, &new, start, length, pad)?;
    Ok(interp.text_built(out))
}

/// `POS(needle, haystack [,start] [,range])`.
pub(crate) fn pos(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    // The numeric arguments are converted first so that the two strings can
    // be read through shared borrows afterwards; `required_render`'s own doc
    // carries why moving them is not observable.
    let start = whole_number(interp, name, args, 3)?;
    let requested = whole_number(interp, name, args, 4)?;
    let needle = required_render(interp, args, 1);
    let haystack = required_render(interp, args, 2);
    let needle = needle.text(interp);
    let haystack = haystack.text(interp);

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    };
    let range = match requested {
        Some(value) => length_of(value)?,
        None => haystack.len().saturating_sub(start) + 1,
    };
    let found = find_forward(haystack, needle, start - 1, range);
    Ok(interp.counted(found))
}

/// `LASTPOS(needle, haystack [,start] [,range])`.
pub(crate) fn lastpos(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let needle = required_string(interp, args, 1);
    let haystack = required_string(interp, args, 2);
    let start = whole_number(interp, name, args, 3)?;
    let requested = whole_number(interp, name, args, 4)?;

    let start = match start {
        Some(value) => position_of(value)?,
        None => haystack.len(),
    };
    let range = match requested {
        Some(value) => length_of(value)?,
        None => haystack.len(),
    };
    let found = find_backward(&haystack, &needle, start, range);
    Ok(interp.counted(found))
}

/// `REVERSE(string)`: the bytes back to front.
pub(crate) fn reverse(
    interp: &mut Interp,
    _name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let mut string = required_string(interp, args, 1);
    string.reverse();
    Ok(interp.text_built(string))
}

/// `STRIP(string [,option] [,chars])`.
pub(crate) fn strip(interp: &mut Interp, _name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let option = optional_string(interp, args, 2);
    let set = optional_string(interp, args, 3);

    let option = option_letter(option.as_deref(), "BLT")?.unwrap_or(b'B');
    let kept = strip_bytes(&string, option, set.as_deref());
    Ok(interp.text(kept))
}

/// `STRIP`'s answer once its option and set are read, shared with
/// `String~strip`.
pub(crate) fn strip_bytes<'a>(string: &'a [u8], option: u8, set: Option<&[u8]>) -> &'a [u8] {
    let set = set.unwrap_or(DEFAULT_STRIP_SET);
    let mut kept = string;
    if option == b'L' || option == b'B' {
        while kept.first().is_some_and(|&byte| in_set(byte, set)) {
            kept = &kept[1..];
        }
    }
    if option == b'T' || option == b'B' {
        while kept.last().is_some_and(|&byte| in_set(byte, set)) {
            kept = &kept[..kept.len() - 1];
        }
    }
    kept
}

/// `SPACE`'s body: the words of `string` joined by `gap` copies of `pad`,
/// appended to `out`.
pub(crate) fn space_bytes(
    out: &mut Vec<u8>,
    string: &[u8],
    gap: usize,
    pad: u8,
) -> Result<(), Raised> {
    // The same scan the word builtins use, so `SPACE`'s idea of a word
    // boundary is not a second statement of the rule free to disagree.
    let words = super::word::word_slices(string);
    if words.is_empty() {
        return Ok(());
    }
    let content: usize = words.iter().map(|word| word.len()).sum();
    let total = gap
        .checked_mul(words.len() - 1)
        .and_then(|padding| padding.checked_add(content))
        .ok_or_else(Raised::system_resources)?;
    reserve(out, total)?;
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            push_pad(out, pad, gap);
        }
        out.extend_from_slice(word);
    }
    Ok(())
}

/// `SPACE(string [,n] [,pad])`: the words of `string` rejoined with `n`
/// copies of `pad`.
pub(crate) fn space(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let requested = whole_number(interp, name, args, 2)?;
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');

    let gap = match requested {
        Some(value) => length_of(value)?,
        None => 1,
    };
    let mut out = interp.take_result_buffer();
    space_bytes(&mut out, &string, gap, pad)?;
    Ok(interp.text_built(out))
}

/// `COPIES(string, n)`.
pub(crate) fn copies(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let count = whole_number(interp, name, args, 2)?.expect("check_arity admitted the count");
    let count = count_of(count, 1)?;
    match copies_bytes(interp, &string, count)? {
        Some(out) => Ok(interp.text_built(out)),
        None => Ok(interp.text_built(string)),
    }
}

/// `COPIES`'s answer once its count is checked, shared with `String~copies`.
pub(crate) fn copies_bytes(
    interp: &Interp,
    string: &[u8],
    count: usize,
) -> Result<Option<Vec<u8>>, Failure> {
    if count == 0 || string.is_empty() {
        return Ok(Some(Vec::new()));
    }
    if count == 1 {
        return Ok(None);
    }
    let total = string
        .len()
        .checked_mul(count)
        .ok_or_else(|| Failure::from(Raised::system_resources()))?;
    let mut out = buffer(interp, total)?;
    for _ in 0..count {
        out.extend_from_slice(string);
    }
    Ok(Some(out))
}

/// `ABBREV(information, info [,length])`: whether `info` is a prefix of
/// `information` at least `length` bytes long.
pub(crate) fn abbrev(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let information = required_string(interp, args, 1);
    let info = required_string(interp, args, 2);
    let requested = whole_number(interp, name, args, 3)?;

    let minimum = match requested {
        Some(value) => Some(length_of(value)?),
        None => None,
    };
    let answer = abbrev_holds(&information, &info, minimum);
    Ok(interp.text(if answer { b"1" } else { b"0" }))
}

/// `ABBREV`'s answer once its two strings and its length are read, shared with
/// `String~abbrev`.
pub(crate) fn abbrev_holds(information: &[u8], info: &[u8], minimum: Option<usize>) -> bool {
    abbrev_holds_over(information, info, minimum, false)
}

/// [`abbrev_holds`] ignoring case, shared with `String~caselessAbbrev`.
pub(crate) fn caseless_abbrev_holds(
    information: &[u8],
    info: &[u8],
    minimum: Option<usize>,
) -> bool {
    abbrev_holds_over(information, info, minimum, true)
}

/// The body both abbreviation tests share; `caseless` is the only difference.
fn abbrev_holds_over(
    information: &[u8],
    info: &[u8],
    minimum: Option<usize>,
    caseless: bool,
) -> bool {
    let minimum = minimum.unwrap_or(info.len());
    if minimum == 0 && info.is_empty() {
        return true;
    }
    if information.is_empty() || info.len() < minimum || information.len() < info.len() {
        return false;
    }
    let prefix = &information[..info.len()];
    if caseless {
        caseless_eq(prefix, info)
    } else {
        prefix == info
    }
}

/// `COMPARE(string1, string2 [,pad])`: the 1-based offset of the first byte
/// at which the two differ once the shorter is padded out, or 0.
pub(crate) fn compare(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let first = required_string(interp, args, 1);
    let second = required_string(interp, args, 2);
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');

    Ok(interp.counted(compare_at(&first, &second, pad)))
}

/// `COMPARE`'s answer once its two strings and its pad are read, shared with
/// `String~compare`: the 1-based offset of the first byte that differs, or 0.
pub(crate) fn compare_at(first: &[u8], second: &[u8], pad: u8) -> usize {
    compare_at_over(first, second, pad, false)
}

/// [`compare_at`] ignoring case, shared with `String~caselessCompare`.
pub(crate) fn caseless_compare_at(first: &[u8], second: &[u8], pad: u8) -> usize {
    compare_at_over(first, second, pad, true)
}

/// The body both comparisons share.
fn compare_at_over(first: &[u8], second: &[u8], pad: u8, caseless: bool) -> usize {
    let same = |a: u8, b: u8| {
        if caseless {
            a.eq_ignore_ascii_case(&b)
        } else {
            a == b
        }
    };
    let shared = first.len().min(second.len());
    (0..shared)
        .find(|&index| !same(first[index], second[index]))
        .map(|index| index + 1)
        .or_else(|| {
            // Whichever string is longer supplies the tail; the other is
            // treated as `pad` repeated, so the answer is the same either
            // way round -- measured, `compare('abc','ab')` and
            // `compare('ab','abc')` are both 3.
            let tail = if first.len() > second.len() {
                &first[shared..]
            } else {
                &second[shared..]
            };
            tail.iter()
                .position(|&byte| !same(byte, pad))
                .map(|offset| shared + offset + 1)
        })
        .unwrap_or(0)
}

/// `compareTo`'s three-way comparison of a region of two strings, shared with
/// `String~compareTo` and `String~caselessCompareTo`.
pub(crate) fn compare_to_over(
    first: &[u8],
    second: &[u8],
    start: usize,
    len: usize,
    caseless: bool,
) -> i8 {
    if start > first.len() {
        return if start > second.len() { 0 } else { -1 };
    }
    if start > second.len() {
        return 1;
    }
    let begin = start - 1;
    let mine = len.min(first.len() - begin);
    let theirs = len.min(second.len() - begin);
    let shared = mine.min(theirs);
    let ordering = if caseless {
        let left: Vec<u8> = first[begin..][..shared].to_ascii_uppercase();
        let right: Vec<u8> = second[begin..][..shared].to_ascii_uppercase();
        left.cmp(&right)
    } else {
        first[begin..][..shared].cmp(&second[begin..][..shared])
    };
    match ordering {
        core::cmp::Ordering::Equal => match mine.cmp(&theirs) {
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
            core::cmp::Ordering::Less => -1,
        },
        core::cmp::Ordering::Greater => 1,
        core::cmp::Ordering::Less => -1,
    }
}

/// `COUNTSTR(needle, haystack)`: how many non-overlapping `needle`s
/// `haystack` holds.
pub(crate) fn countstr(
    interp: &mut Interp,
    _name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let needle = required_string(interp, args, 1);
    let haystack = required_string(interp, args, 2);
    let count = count_occurrences(&haystack, &needle, usize::MAX);
    Ok(interp.counted(count))
}

/// `CHANGESTR`'s body: `haystack` with its first `limit` `needle`s replaced
/// by `replacement`, appended to `out`.
pub(crate) fn changestr_bytes(
    out: &mut Vec<u8>,
    haystack: &[u8],
    needle: &[u8],
    replacement: &[u8],
    limit: usize,
) -> Result<(), Raised> {
    changestr_with(out, haystack, needle, replacement, limit, find_forward)
}

/// [`changestr_bytes`] with ASCII case folded away,
/// `MutableBuffer::caselessChangeStr` (`classes/MutableBufferClass.cpp:1136`).
pub(crate) fn caseless_changestr_bytes(
    out: &mut Vec<u8>,
    haystack: &[u8],
    needle: &[u8],
    replacement: &[u8],
    limit: usize,
) -> Result<(), Raised> {
    changestr_with(
        out,
        haystack,
        needle,
        replacement,
        limit,
        caseless_find_forward,
    )
}

/// The rebuild both spellings run, `find` the only difference.
fn changestr_with(
    out: &mut Vec<u8>,
    haystack: &[u8],
    needle: &[u8],
    replacement: &[u8],
    limit: usize,
    find: impl Fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<(), Raised> {
    // **One search, and the answer written as it is found.** Sizing the result
    // before writing it means counting the occurrences first, and counting
    // runs `find_forward` from each occurrence to the next exactly as writing
    // does -- so the haystack is searched twice for one answer. Measured on
    // `bench-programs/strings.rex`, whose loop changes a 43-byte haystack
    // three million times, that second search is 2.4% of the whole program's
    // `instructions:u`.
    reserve(out, haystack.len())?;
    let mut next = 0;
    let mut changes = 0;
    while changes < limit {
        let found = find(haystack, needle, next, haystack.len());
        if found == 0 {
            break;
        }
        let kept = &haystack[next..found - 1];
        // **Every growth stays fallible**, which is what a result sized in one
        // reservation gets for free and this one has to ask for.
        // `extend_from_slice` grows through the infallible path and aborts the
        // process where the oracle raises 5.1, so the room for what is about
        // to be written is taken here and the extends below cannot be what
        // grows the buffer.
        reserve(out, kept.len() + replacement.len())?;
        out.extend_from_slice(kept);
        out.extend_from_slice(replacement);
        next = found - 1 + needle.len();
        changes += 1;
    }
    // **No branch for "nothing changed"**, and none is needed: the loop leaves
    // `next` at 0 when it never ran, so this copies the haystack whole, which
    // is what CHANGESTR answers when the needle is absent, when the needle is
    // the null string and when the requested count is 0.
    let rest = &haystack[next..];
    reserve(out, rest.len())?;
    out.extend_from_slice(rest);
    Ok(())
}

/// `CHANGESTR(needle, haystack, newneedle [,count])`.
pub(crate) fn changestr(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let requested = whole_number(interp, name, args, 4)?;
    let needle = required_render(interp, args, 1);
    let haystack = required_render(interp, args, 2);
    let replacement = required_render(interp, args, 3);

    let limit = match requested {
        Some(value) => count_of(value, 3)?,
        None => usize::MAX,
    };
    // **The lent buffer, like every other sized result in this module.** A
    // fresh `Vec` here was not merely one allocation: `text_built` hands
    // whatever it is given back to the pool, so a fresh one *replaced* the
    // shared buffer with a tighter one on every call, and the next caller
    // wanting a byte more grew it again. Measured on
    // `bench-programs/strings.rex`, that pair was the whole of what the
    // program still allocated.
    let mut out = interp.take_result_buffer();
    changestr_bytes(
        &mut out,
        haystack.text(interp),
        needle.text(interp),
        replacement.text(interp),
        limit,
    )?;
    Ok(interp.text_built(out))
}

/// `TRANSLATE(string [,tableout] [,tablein] [,pad] [,start] [,range])`.
/// ```text
/// zz = ''          ; say '['translate('abcdef','123',zz)']'   ->  [abcdef]
/// zz = left('abc',0) ; say '['translate('abcdef','123',zz)']' ->  [      ]
/// ```
pub(crate) fn translate(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let mut string = required_string(interp, args, 1);
    let out_table = optional_string(interp, args, 2);
    let in_table = optional_string(interp, args, 3);
    let pad = pad_byte(interp, name, args, 4)?;
    let start = whole_number(interp, name, args, 5)?;
    let range = whole_number(interp, name, args, 6)?;

    if out_table.is_none() && in_table.is_none() && pad.is_none() {
        return case_shifted(interp, string, start, range, u8::to_ascii_uppercase);
    }
    let out_table = out_table.unwrap_or_default();
    let pad = pad.unwrap_or(b' ');

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    } - 1;
    let range = range.map(length_of).transpose()?;
    translate_bytes(
        &mut string,
        &out_table,
        in_table.as_deref(),
        pad,
        start,
        range,
    );
    Ok(interp.text_built(string))
}

/// `TRANSLATE`'s body with a table: each byte of `bytes` within `range` of
/// 0-based `start` that `in_table` holds -- or every byte, read as its own
/// index, for `None` -- becomes `out_table`'s byte at that index, or `pad`
/// past its end; a `range` of `None` reaches the end.
pub(crate) fn translate_bytes(
    bytes: &mut [u8],
    out_table: &[u8],
    in_table: Option<&[u8]>,
    pad: u8,
    start: usize,
    range: Option<usize>,
) {
    let Some(window) = window(bytes, start, range) else {
        return;
    };
    for byte in window {
        let index = match in_table {
            Some(table) => table.iter().position(|entry| entry == byte),
            None => Some(usize::from(*byte)),
        };
        if let Some(index) = index {
            *byte = out_table.get(index).copied().unwrap_or(pad);
        }
    }
}

/// The bytes within `range` of 0-based `start`, clipped to the end, or `None`
/// for a `start` at or past it; a `range` of `None` reaches the end.
fn window(bytes: &mut [u8], start: usize, range: Option<usize>) -> Option<&mut [u8]> {
    if start >= bytes.len() {
        return None;
    }
    let available = bytes.len() - start;
    let range = range.map_or(available, |range| range.min(available));
    Some(&mut bytes[start..start + range])
}

/// `VERIFY(string, reference [,option] [,start] [,range])`.
pub(crate) fn verify(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let reference = required_string(interp, args, 2);
    let option = optional_string(interp, args, 3);
    let start = whole_number(interp, name, args, 4)?;
    let range = whole_number(interp, name, args, 5)?;

    let option = option_letter(option.as_deref(), "MN")?.unwrap_or(b'N');
    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    } - 1;
    let range = range.map(length_of).transpose()?;

    // Answered as text rather than through `counted`, unlike every other
    // zero this function answers.
    if start >= string.len() {
        return Ok(interp.text(b"0"));
    }
    let answer = verify_bytes(&string, &reference, option, start, range);
    Ok(interp.counted(answer))
}

/// `VERIFY`'s body: the 1-based offset of the first byte within `range` of
/// 0-based `start` that `reference` does not hold (`option` `N`) or does hold
/// (any other letter), or 0; a `range` of `None` reaches the end.
pub(crate) fn verify_bytes(
    string: &[u8],
    reference: &[u8],
    option: u8,
    start: usize,
    range: Option<usize>,
) -> usize {
    if start >= string.len() {
        return 0;
    }
    let available = string.len() - start;
    let range = range.map_or(available, |range| range.min(available));
    if reference.is_empty() {
        // `if (opt == VERIFY_MATCH) return 0; else return startPos;`
        return if option == b'M' { 0 } else { start + 1 };
    }
    // `if (opt == VERIFY_NOMATCH) ... else ...`, the other letter.
    let matching = option != b'N';
    string[start..start + range]
        .iter()
        .position(|&byte| in_set(byte, reference) == matching)
        .map_or(0, |offset| start + 1 + offset)
}

/// `LOWER(string [,n] [,length])`.
pub(crate) fn lower(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let start = whole_number(interp, name, args, 2)?;
    let range = whole_number(interp, name, args, 3)?;
    case_shifted(interp, string, start, range, u8::to_ascii_lowercase)
}

/// `UPPER(string [,n] [,length])`.
pub(crate) fn upper(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let start = whole_number(interp, name, args, 2)?;
    let range = whole_number(interp, name, args, 3)?;
    case_shifted(interp, string, start, range, u8::to_ascii_uppercase)
}

/// The body `LOWER`, `UPPER` and `TRANSLATE`'s no-table form share.
fn case_shifted(
    interp: &mut Interp,
    mut string: Vec<u8>,
    start: Option<i64>,
    range: Option<i64>,
    shift: fn(&u8) -> u8,
) -> Result<ObjRef, Failure> {
    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    } - 1;
    let range = range.map(length_of).transpose()?;
    case_shift_bytes(&mut string, start, range, shift);
    Ok(interp.text_built(string))
}

/// `UPPER` and `LOWER`'s body: `shift` applied to the bytes within `range` of
/// 0-based `start`; a `range` of `None` reaches the end.
pub(crate) fn case_shift_bytes(
    bytes: &mut [u8],
    start: usize,
    range: Option<usize>,
    shift: fn(&u8) -> u8,
) {
    if let Some(window) = window(bytes, start, range) {
        for byte in window {
            *byte = shift(byte);
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scan_tests {
    use super::find_byte;

    /// [`find_byte`]'s word step against the byte-at-a-time answer it stands
    /// in for, at every length from empty to past the step's own width.
    #[test]
    fn find_byte_agrees_with_position() {
        let alphabet = [0x00u8, 0x01, 0x80, b'a'];
        for length in 0..40usize {
            for seed in 0..500u32 {
                let mut hay = Vec::with_capacity(length);
                let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(length as u32);
                for _ in 0..length {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    hay.push(alphabet[(state >> 16) as usize % alphabet.len()]);
                }
                for &wanted in &alphabet {
                    assert_eq!(
                        find_byte(&hay, wanted),
                        hay.iter().position(|&byte| byte == wanted),
                        "hay {hay:02x?} wanted {wanted:#04x}"
                    );
                }
            }
        }
    }
}
