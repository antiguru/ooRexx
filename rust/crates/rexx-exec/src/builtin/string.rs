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
//!
//! # Two layers of argument checking, and the order between them is observable
//!
//! The oracle validates a builtin's arguments twice, in two places that
//! disagree about both the error family and the exit code, and a program can
//! tell which ran first.
//!
//! * The **call layer** converts each argument to the kind the builtin
//!   declared, walking the argument list left to right. A value that is not a
//!   whole number is `40.12` and a pad that is not one character is `40.23`,
//!   both at rc 216, and both name the routine and the argument's position in
//!   the *call*.
//! * The **operation layer** then range-checks the converted values in the
//!   order the underlying string operation reads them. A negative length is
//!   `93.923`, a non-positive position is `93.924` and a negative count is
//!   `93.906`, all at **rc 163**, and none of them names the routine.
//!
//! Every conversion happens before any range check, measured at each of the
//! three shapes that can tell them apart:
//!
//! ```text
//! say substr('abc',0,5,'xx')          40.23  (the pad, not the zero position)
//! say translate('abc',,,'$',0,'q')    40.12 argument 6  (not the zero start)
//! say verify('a','b','X','q')         40.12 argument 4  (not the bad option)
//! ```
//!
//! So each function below reads its arguments in position order through
//! [`whole_number`] and [`pad_byte`], and only then applies [`length_of`],
//! [`position_of`], [`count_of`] and [`option_letter`].
//!
//! # Results are text, not numbers
//!
//! A builtin answering a count or an offset creates its result as text, for
//! the reason [`length`]'s own comment gives and measured the same way for
//! each: under `numeric digits 1`, `pos('a','bbbbbbbbba')`,
//! `lastpos('a','bbbbbbbbba')`, `compare('bbbbbbbbba','bbbbbbbbbz')`,
//! `countstr('a','aaaaaaaaaa')` and `verify('bbbbbbbbba','b')` all print
//! `10`, where a value carrying `DIGITS 1` as its created pair would render
//! `1E+1`. D15 stays visible from the other side on the same value: bound to
//! `n` under `numeric digits 1`, `say n` is `10` and `say n + 0` is `1E+1`.
//!
//! # Bytes, not characters
//!
//! Every length, position and case conversion here is over bytes.
//! `Utilities::toUpper`/`toLower` (`common/Utilities.hpp`) fold only `A`-`Z`
//! and `a`-`z`, so a byte outside ASCII is left alone -- measured,
//! `upper('e9'x)` and `lower('c9'x)` return their argument unchanged.

use rexx_core::ObjRef;

use crate::Interp;
use crate::error::{Failure, Raised};

/// The precision the oracle converts a builtin's numeric arguments under.
///
/// `Numerics::ARGUMENT_DIGITS` (`runtime/Numerics.hpp`), 18 on a 64-bit
/// build and deliberately not the current `NUMERIC DIGITS` --
/// `Raised::argument_not_whole` carries the pair of measurements that
/// separates the two.
const ARGUMENT_DIGITS: usize = 18;

/// What `STRIP` strips when no character set is given: blank and horizontal
/// tab (`RexxString::strip`, `classes/StringClassSub.cpp`). Measured, a tab
/// really is in the default set -- `strip('a'||'09'x||'b'||'09'x)` answers
/// `a<tab>b`, with the trailing tab gone.
const DEFAULT_STRIP_SET: &[u8] = b" \t";

/// `LENGTH(string)`: how many bytes the argument renders as.
///
/// **The result is a plain integer whose rendering does not depend on
/// `NUMERIC DIGITS`, so it is created as text and not through
/// `Interp::number`.** Measured: `numeric digits 1 ; say
/// length('abcdefghij')` prints `10`, where a value carrying `DIGITS 1` as
/// its created pair would render `1E+1`. It is a *value*, not a *number*
/// whose precision was captured -- and D15 is still visible from the other
/// side, measured on the same value: built under `numeric digits 3` and read
/// back under `numeric digits 1`, `say n` is still `16` while `say n + 0` is
/// `2E+1`, because the addition is a new operation creating a new number
/// under the digits then in force. `set_sigl` (`run.rs`) creates a line
/// number the same way and for the same reason.
///
/// Bytes, not characters: measured, `say length('1.50')` is 4 and `say
/// length('')` is 0. `to_text` is what the oracle's own `REQUIRED_STRING`
/// conversion corresponds to, so a number argument is measured by its
/// rendering -- `say length(1.50)` is 4, not 3.
pub(crate) fn length(
    interp: &mut Interp,
    _name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let value = args[0].expect("check_arity admitted LENGTH's one required argument");
    // The borrow of `interp` ends with this statement, which is what lets the
    // allocation below happen at all.
    let bytes = interp.to_text(value).len();
    Ok(interp.text(bytes.to_string().as_bytes()))
}

// ---- reading arguments ----

/// The argument at 1-based `position`, or `None` if the call did not supply
/// one there.
///
/// The two ways a position can be absent are one answer here on purpose: a
/// list shorter than `position` and an interior `None` mean the same thing to
/// every builtin below, since the oracle's `optional_*` macros test
/// `argcount >= position` and then read a slot that may itself be null.
/// `check_arity` guarantees positions `1..=min` are all `Some`, having
/// turned any omission there into 40.5, so those are the positions the
/// `expect`ing helpers below may be asked about -- and only those.
fn arg(args: &[Option<ObjRef>], position: usize) -> Option<ObjRef> {
    args.get(position - 1).copied().flatten()
}

/// The rendered bytes of the argument at 1-based `position`, which the
/// caller knows is present.
fn required_string(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Vec<u8> {
    let value = arg(args, position).expect("check_arity admitted this required argument");
    interp.to_text(value).into_owned()
}

/// The rendered bytes of an optional argument.
fn optional_string(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Option<Vec<u8>> {
    let value = arg(args, position)?;
    Some(interp.to_text(value).into_owned())
}

/// An argument the builtin declared as an integer, converted the way the
/// oracle's `optional_integer`/`required_integer` macros do.
///
/// `Ok(None)` is "the call supplied nothing here"; the caller then applies
/// that argument's own default, which differs per builtin and is never a
/// single shared value.
fn whole_number(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<i64>, Failure> {
    let Some(value) = arg(args, position) else {
        return Ok(None);
    };
    // `to_number` hands back an owned `Number`, so the borrow of `interp` is
    // over before `to_text` below needs its own.
    if let Ok(number) = interp.to_number(value)
        && let Some(whole) = number.whole_value(ARGUMENT_DIGITS)
    {
        return Ok(Some(whole));
    }
    let found = interp.to_text(value).into_owned();
    Err(Raised::argument_not_whole(name, position, &found).into())
}

/// An argument the builtin declared as a pad, which must be exactly one
/// byte.
fn pad_byte(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<u8>, Failure> {
    let Some(value) = arg(args, position) else {
        return Ok(None);
    };
    let found = interp.to_text(value).into_owned();
    match found.as_slice() {
        [byte] => Ok(Some(*byte)),
        _ => Err(Raised::argument_not_a_pad(name, position, &found).into()),
    }
}

// ---- range-checking converted arguments ----

/// A converted argument used as a length: zero or positive.
fn length_of(value: i64) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| Raised::invalid_length(value.to_string().as_bytes()).into())
}

/// A converted argument used as a position: strictly positive.
fn position_of(value: i64) -> Result<usize, Failure> {
    match usize::try_from(value) {
        Ok(position) if position > 0 => Ok(position),
        _ => Err(Raised::invalid_position(value.to_string().as_bytes()).into()),
    }
}

/// A converted argument used as a repetition or replacement count: zero or
/// positive. `method_position` is the position the oracle's message names,
/// which is the *operation's* own numbering rather than the call's.
fn count_of(value: i64, method_position: usize) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| {
        Raised::argument_not_non_negative(method_position, value.to_string().as_bytes()).into()
    })
}

/// The upper-cased first letter of an option argument, checked against the
/// set the builtin accepts.
///
/// `None` for an omitted argument, so the caller supplies the default; an
/// argument that is present but empty is an error rather than a default,
/// which is measured (`strip('ab','')` is 93.915).
///
/// **A first byte of `0x00` is accepted and is not any of the letters**,
/// which is the oracle's own answer and not a kindness of this one. The
/// check there is `strchr(validOptions, option) == NULL` over an ASCII-Z
/// string (`optionArgument`, `classes/StringClassUtil.cpp`), and `strchr`
/// finds the terminating NUL, so a NUL option passes a test written to
/// reject anything outside the set. Measured, and the callers below then
/// answer whatever their own "none of the letters" branch says:
/// `strip('  ab  ','00'x)` is `  ab  ` unstripped, `verify('abcde','abc','00'x)`
/// is 1 -- the answer `'M'` gives, not `'N'`'s. Only the first byte decides:
/// `strip('  ab  ','00'x||'L')` is unstripped and `strip('  ab  ','L'||'00'x)`
/// strips leading. Every other control byte is refused as usual.
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

/// A result buffer of exactly `len` bytes' capacity, or the condition the
/// oracle raises when the allocator refuses.
///
/// See `Raised::system_resources` for why the refusal is asked of the
/// allocator rather than of a size limit.
fn buffer(len: usize) -> Result<Vec<u8>, Failure> {
    let mut out = Vec::new();
    out.try_reserve_exact(len)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    Ok(out)
}

/// `len` copies of `byte`, appended.
fn push_pad(out: &mut Vec<u8>, byte: u8, len: usize) {
    out.resize(out.len() + len, byte);
}

// ---- the shared search primitives ----

/// The 1-based offset of the first `needle` inside `haystack[start..]` that
/// lies wholly within `range` bytes of `start`, or 0 for no match.
///
/// `start` is 0-based here, as `StringUtil::pos`'s own parameter is.
/// A null needle never matches, measured: `pos('','banana')` is 0.
fn find_forward(haystack: &[u8], needle: &[u8], start: usize, range: usize) -> usize {
    // `haystack.len() - start` underflows for a start past the end, which is
    // exactly the case the guard below rejects; taking the saturating
    // difference first keeps the two independent.
    let range = range.min(haystack.len().saturating_sub(start));
    if start >= haystack.len() || needle.len() > range || needle.is_empty() {
        return 0;
    }
    let window = &haystack[start..start + range];
    window
        .windows(needle.len())
        .position(|candidate| candidate == needle)
        .map_or(0, |offset| start + offset + 1)
}

/// The 1-based offset of the last `needle` that ends at or before `start`
/// and begins no earlier than `range` bytes before that end, or 0.
///
/// **The match has to end within the window, not merely begin there**, which
/// is the rule a one-character needle cannot show: measured,
/// `lastpos('abc','xxabc',5)` is 3 while `lastpos('abc','xxabc',4)` and
/// `lastpos('abc','xxabc',3)` are both 0.
fn find_backward(haystack: &[u8], needle: &[u8], start: usize, range: usize) -> usize {
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
        .rposition(|candidate| candidate == needle)
        .map_or(0, |offset| end - range + offset + 1)
}

/// How many non-overlapping `needle`s `haystack` holds, stopping at `limit`.
///
/// Non-overlapping is measured: `countstr('aa','aaaa')` is 2, not 3.
fn count_occurrences(haystack: &[u8], needle: &[u8], limit: usize) -> usize {
    if needle.is_empty() || needle.len() > haystack.len() || limit == 0 {
        return 0;
    }
    let mut count = 0;
    let mut next = 0;
    while count < limit {
        let found = find_forward(haystack, needle, next, haystack.len());
        if found == 0 {
            break;
        }
        next = found - 1 + needle.len();
        count += 1;
    }
    count
}

/// Whether `byte` is one of `set`'s.
fn in_set(byte: u8, set: &[u8]) -> bool {
    set.contains(&byte)
}

/// The word boundaries `SPACE` uses: blank and horizontal tab only
/// (`RexxString::WordIterator::skipBlanks`). Measured, a newline is *not* a
/// separator -- `space('a'||'0a'x||'b')` keeps the newline inside one word.
fn words(text: &[u8]) -> Vec<&[u8]> {
    text.split(|&byte| byte == b' ' || byte == b'\t')
        .filter(|word| !word.is_empty())
        .collect()
}

// ---- the builtins ----

/// `CENTER(string, length [,pad])`, and `CENTRE` under its other spelling.
///
/// One implementation for both names, reached from two [`super::IMPLEMENTED`]
/// rows: the oracle's two `BUILTIN` bodies are identical bar the name they
/// report, and the name arrives as a parameter so the messages differ without
/// the code doing. Measured, they do differ -- `centre('ab',6,'--')` names
/// `CENTRE` where `center('ab',6,'--')` names `CENTER`.
///
/// **When the padding or the truncation does not divide evenly, the odd
/// character goes to the right**, and both halves of that are measured:
/// `center('ab',5,'-')` is `-ab--`, one pad left and two right, and
/// `center('abcdef',5)` is `abcde`, nothing dropped from the left and one
/// byte from the right. A wider truncation drops from both --
/// `center('abcdef',3)` is `bcd`.
pub(crate) fn center(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let width = whole_number(interp, name, args, 2)?.expect("check_arity admitted the width");
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');
    let width = length_of(width)?;

    let len = string.len();
    if width == len {
        return Ok(interp.text_owned(string));
    }
    if width == 0 {
        return Ok(interp.text(b""));
    }
    let out = if width > len {
        let left = (width - len) / 2;
        let mut out = buffer(width)?;
        push_pad(&mut out, pad, left);
        out.extend_from_slice(&string);
        push_pad(&mut out, pad, width - len - left);
        out
    } else {
        string[(len - width) / 2..][..width].to_vec()
    };
    Ok(interp.text_owned(out))
}

/// `LEFT(string, length [,pad])`: the leading `length` bytes, padded on the
/// right.
pub(crate) fn left(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let size = whole_number(interp, name, args, 2)?.expect("check_arity admitted the length");
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');
    let size = length_of(size)?;

    if size == 0 {
        return Ok(interp.text(b""));
    }
    let kept = string.len().min(size);
    let mut out = buffer(size)?;
    out.extend_from_slice(&string[..kept]);
    push_pad(&mut out, pad, size - kept);
    Ok(interp.text_owned(out))
}

/// `RIGHT(string, length [,pad])`: the trailing `length` bytes, padded on the
/// left.
pub(crate) fn right(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let size = whole_number(interp, name, args, 2)?.expect("check_arity admitted the length");
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');
    let size = length_of(size)?;

    if size == 0 {
        return Ok(interp.text(b""));
    }
    let kept = string.len().min(size);
    let mut out = buffer(size)?;
    push_pad(&mut out, pad, size - kept);
    out.extend_from_slice(&string[string.len() - kept..]);
    Ok(interp.text_owned(out))
}

/// `SUBSTR(string, n [,length] [,pad])`.
///
/// **A start past the end is not an error**, unlike a start of zero:
/// measured, `substr('abcdef',7)` is the null string and
/// `substr('abcdef',7,3,'.')` is `...`, where `substr('abc',0)` is 93.924.
pub(crate) fn substr(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let start = whole_number(interp, name, args, 2)?.expect("check_arity admitted the position");
    let requested = whole_number(interp, name, args, 3)?;
    let pad = pad_byte(interp, name, args, 4)?.unwrap_or(b' ');

    let start = position_of(start)? - 1;
    // The default is everything from the start position on, which is nothing
    // at all once the position is past the end.
    let length = match requested {
        Some(value) => length_of(value)?,
        None => string.len().saturating_sub(start),
    };
    if length == 0 {
        return Ok(interp.text(b""));
    }
    let kept = length.min(string.len().saturating_sub(start));
    let mut out = buffer(length)?;
    out.extend_from_slice(&string[start.min(string.len())..][..kept]);
    push_pad(&mut out, pad, length - kept);
    Ok(interp.text_owned(out))
}

/// `DELSTR(string [,n] [,length])`.
///
/// **Both numeric arguments are optional and `n` defaults to 1**, so a
/// one-argument call deletes the whole string: measured,
/// `delstr('abcdef')` is the null string and `delstr('abcdef',,2)` is
/// `cdef`.
pub(crate) fn delstr(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let start = whole_number(interp, name, args, 2)?;
    let requested = whole_number(interp, name, args, 3)?;

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    };
    // Range-checked before the out-of-range start is answered, which is what
    // the oracle's own ordering does -- measured, `delstr('abc',9,-1)` is
    // 93.923 rather than the unchanged string.
    let deleted = match requested {
        Some(value) => length_of(value)?,
        None => string.len().saturating_sub(start) + 1,
    };

    if start > string.len() {
        return Ok(interp.text_owned(string));
    }
    if start == 1 && deleted >= string.len() {
        return Ok(interp.text(b""));
    }
    let start = start - 1;
    let tail = start.saturating_add(deleted);
    let mut out = string[..start].to_vec();
    if tail < string.len() {
        out.extend_from_slice(&string[tail..]);
    }
    Ok(interp.text_owned(out))
}

/// `INSERT(new, target [,n] [,length] [,pad])`.
///
/// **`n` is a count of characters to skip, not a position**, which is why
/// zero is legal here and an error in `OVERLAY`: measured,
/// `insert('-','abc',0)` is `-abc` while `overlay('XY','abcdef',0)` is
/// 93.924. A negative `n` is 93.906, not 93.924.
pub(crate) fn insert(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let new = required_string(interp, args, 1);
    let target = required_string(interp, args, 2);
    let start = whole_number(interp, name, args, 3)?;
    let requested = whole_number(interp, name, args, 4)?;
    let pad = pad_byte(interp, name, args, 5)?.unwrap_or(b' ');

    let start = match start {
        Some(value) => count_of(value, 2)?,
        None => 0,
    };
    let insert_len = match requested {
        Some(value) => length_of(value)?,
        None => new.len(),
    };

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
        .ok_or_else(|| Failure::from(Raised::system_resources()))?;
    let mut out = buffer(total)?;
    out.extend_from_slice(&target[..front]);
    push_pad(&mut out, pad, lead_pad);
    out.extend_from_slice(&new[..copied]);
    push_pad(&mut out, pad, insert_len - copied);
    out.extend_from_slice(&target[front..front + back]);
    Ok(interp.text_owned(out))
}

/// `OVERLAY(new, target [,n] [,length] [,pad])`.
///
/// **A zero length still extends the target out to `n`**, which is the shape
/// that looks like a no-op and is not: measured, `overlay('XY','abc',5,0)` is
/// `abc` followed by one blank and `overlay('','abc',6,1)` is `abc` followed
/// by three, while `overlay('XY','abc',3,0)` and `overlay('XY','abc',4,0)`
/// are both `abc` unchanged, since neither reaches past the end.
pub(crate) fn overlay(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let new = required_string(interp, args, 1);
    let target = required_string(interp, args, 2);
    let start = whole_number(interp, name, args, 3)?;
    let requested = whole_number(interp, name, args, 4)?;
    let pad = pad_byte(interp, name, args, 5)?.unwrap_or(b' ');

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    };
    let overlay_len = match requested {
        Some(value) => length_of(value)?,
        None => new.len(),
    };

    let (copied, back_pad) = if overlay_len > new.len() {
        (new.len(), overlay_len - new.len())
    } else {
        (overlay_len, 0)
    };

    let mut front_pad = 0;
    let mut front = start - 1;
    // The tail begins one past the overlaid span and is empty whenever that
    // span reaches the end, which is also the only case in which the
    // subtraction would have gone negative.
    let span_end = start.saturating_add(overlay_len).saturating_sub(1);
    let mut back = target.len().saturating_sub(span_end);
    if start > target.len() {
        front_pad = start - target.len() - 1;
        front = target.len();
    }
    if span_end.saturating_add(1) > target.len() {
        back = 0;
    }

    let total = front + back + front_pad + overlay_len;
    let mut out = buffer(total)?;
    out.extend_from_slice(&target[..front]);
    push_pad(&mut out, pad, front_pad);
    out.extend_from_slice(&new[..copied]);
    push_pad(&mut out, pad, back_pad);
    if back > 0 {
        // `span_end` can be past the end of the target -- an overlay
        // starting or finishing beyond it -- and that is exactly when there
        // is no tail, so the index is only ever formed when it is in range.
        out.extend_from_slice(&target[span_end..span_end + back]);
    }
    Ok(interp.text_owned(out))
}

/// `POS(needle, haystack [,start] [,range])`.
///
/// The two trailing arguments are ooRexx's own extension; `range` counts
/// bytes from `start`, and a match has to fit inside it -- measured,
/// `pos('an','banana',1,3)` is 2 and `pos('an','banana',1,2)` is 0.
pub(crate) fn pos(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let needle = required_string(interp, args, 1);
    let haystack = required_string(interp, args, 2);
    let start = whole_number(interp, name, args, 3)?;
    let requested = whole_number(interp, name, args, 4)?;

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    };
    let range = match requested {
        Some(value) => length_of(value)?,
        None => haystack.len().saturating_sub(start) + 1,
    };
    let found = find_forward(&haystack, &needle, start - 1, range);
    Ok(interp.text(found.to_string().as_bytes()))
}

/// `LASTPOS(needle, haystack [,start] [,range])`.
///
/// `start` defaults to the end of the haystack and `range` to the haystack's
/// **whole length**, not to what is left of it from `start` -- the two
/// builtins do not mirror each other, since `LASTPOS`'s range extends
/// *backwards*. The distinction takes a needle before the start position to
/// see at all: measured, `lastpos('b','banana',5)` is 1, where a range
/// defaulting to `len - start + 1` would have searched only the two bytes
/// before position 5 and answered 0 -- which is what
/// `lastpos('b','banana',5,2)` does answer.
pub(crate) fn lastpos(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
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
    Ok(interp.text(found.to_string().as_bytes()))
}

/// `REVERSE(string)`: the bytes back to front.
pub(crate) fn reverse(
    interp: &mut Interp,
    _name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let mut string = required_string(interp, args, 1);
    string.reverse();
    Ok(interp.text_owned(string))
}

/// `STRIP(string [,option] [,chars])`.
///
/// **`chars` is a set, not a pad**, so it takes any number of characters and
/// an empty one strips nothing at all: measured,
/// `strip('+-+-a-+b-+-+',,'-+')` is `a-+b` and `strip('abc','B','')` is
/// `abc`.
pub(crate) fn strip(
    interp: &mut Interp,
    _name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let option = optional_string(interp, args, 2);
    let set = optional_string(interp, args, 3);

    let option = option_letter(option.as_deref(), "BLT")?.unwrap_or(b'B');
    let set = set.as_deref().unwrap_or(DEFAULT_STRIP_SET);

    let mut kept = string.as_slice();
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
    Ok(interp.text(kept))
}

/// `SPACE(string [,n] [,pad])`: the words of `string` rejoined with `n`
/// copies of `pad`.
pub(crate) fn space(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let requested = whole_number(interp, name, args, 2)?;
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');

    let gap = match requested {
        Some(value) => length_of(value)?,
        None => 1,
    };
    let words = words(&string);
    if words.is_empty() {
        return Ok(interp.text(b""));
    }
    let content: usize = words.iter().map(|word| word.len()).sum();
    let total = gap
        .checked_mul(words.len() - 1)
        .and_then(|padding| padding.checked_add(content))
        .ok_or_else(|| Failure::from(Raised::system_resources()))?;
    let mut out = buffer(total)?;
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            push_pad(&mut out, pad, gap);
        }
        out.extend_from_slice(word);
    }
    Ok(interp.text_owned(out))
}

/// `COPIES(string, n)`.
pub(crate) fn copies(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let count = whole_number(interp, name, args, 2)?.expect("check_arity admitted the count");
    let count = count_of(count, 1)?;

    if count == 0 || string.is_empty() {
        return Ok(interp.text(b""));
    }
    let total = string
        .len()
        .checked_mul(count)
        .ok_or_else(|| Failure::from(Raised::system_resources()))?;
    let mut out = buffer(total)?;
    for _ in 0..count {
        out.extend_from_slice(&string);
    }
    Ok(interp.text_owned(out))
}

/// `ABBREV(information, info [,length])`: whether `info` is a prefix of
/// `information` at least `length` bytes long.
///
/// Case-sensitive, measured: `abbrev('Print','PRI')` is 0. The result is the
/// text `1` or `0` rather than a boolean object -- measured,
/// `datatype(abbrev('a','a'))` is `NUM` and `abbrev('a','a') + 1` is 2.
pub(crate) fn abbrev(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let information = required_string(interp, args, 1);
    let info = required_string(interp, args, 2);
    let requested = whole_number(interp, name, args, 3)?;

    let minimum = match requested {
        Some(value) => length_of(value)?,
        None => info.len(),
    };
    // An empty `info` with a zero minimum abbreviates anything, including
    // the empty string -- measured, `abbrev('','')` is 1 where
    // `abbrev('','x')` is 0.
    let answer = if minimum == 0 && info.is_empty() {
        true
    } else if information.is_empty() || info.len() < minimum || information.len() < info.len() {
        false
    } else {
        information[..info.len()] == info[..]
    };
    Ok(interp.text(if answer { b"1" } else { b"0" }))
}

/// `COMPARE(string1, string2 [,pad])`: the 1-based offset of the first byte
/// at which the two differ once the shorter is padded out, or 0.
pub(crate) fn compare(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let first = required_string(interp, args, 1);
    let second = required_string(interp, args, 2);
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(b' ');

    let shared = first.len().min(second.len());
    let mismatch = (0..shared)
        .find(|&index| first[index] != second[index])
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
                .position(|&byte| byte != pad)
                .map(|offset| shared + offset + 1)
        })
        .unwrap_or(0);
    Ok(interp.text(mismatch.to_string().as_bytes()))
}

/// `COUNTSTR(needle, haystack)`: how many non-overlapping `needle`s
/// `haystack` holds.
pub(crate) fn countstr(
    interp: &mut Interp,
    _name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let needle = required_string(interp, args, 1);
    let haystack = required_string(interp, args, 2);
    let count = count_occurrences(&haystack, &needle, usize::MAX);
    Ok(interp.text(count.to_string().as_bytes()))
}

/// `CHANGESTR(needle, haystack, newneedle [,count])`.
pub(crate) fn changestr(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let needle = required_string(interp, args, 1);
    let haystack = required_string(interp, args, 2);
    let replacement = required_string(interp, args, 3);
    let requested = whole_number(interp, name, args, 4)?;

    let limit = match requested {
        Some(value) => count_of(value, 3)?,
        None => usize::MAX,
    };
    let changes = count_occurrences(&haystack, &needle, limit);
    if changes == 0 {
        return Ok(interp.text_owned(haystack));
    }
    let mut out = Vec::new();
    let mut next = 0;
    for _ in 0..changes {
        let found = find_forward(&haystack, &needle, next, haystack.len());
        out.extend_from_slice(&haystack[next..found - 1]);
        out.extend_from_slice(&replacement);
        next = found - 1 + needle.len();
    }
    out.extend_from_slice(&haystack[next..]);
    Ok(interp.text_owned(out))
}

/// `TRANSLATE(string [,tableout] [,tablein] [,pad] [,start] [,range])`.
///
/// **With all three of `tableout`, `tablein` and `pad` omitted this is
/// `UPPER`**, start and range included: measured, `translate('abcdef')` is
/// `ABCDEF` and `translate('abcdef',,,,2,3)` is `aBCDef`.
///
/// **An omitted `tablein` means "the byte is its own index"; a supplied one
/// means "look the byte up"**, and the two differ even when the supplied
/// table is empty: measured, `translate('abcdef','123')` is six blanks
/// (every byte's index is past the end of a three-byte `tableout`, so every
/// byte becomes the pad) while `translate('abcdef','','')` is `abcdef`
/// unchanged (no byte is found in an empty table, so none is translated).
///
/// **A known divergence lives in exactly that distinction.** The oracle does
/// not ask whether `tablein` was supplied; it compares the argument's
/// *address* against its own null-string singleton
/// (`tablei != GlobalNames::NULLSTRING` in `RexxString::translate`), and
/// several builtins return that singleton rather than a fresh empty string.
/// So a null string that came from one of them takes the omitted path, and a
/// null string written as a literal does not. Measured, on one line each:
///
/// ```text
/// zz = ''          ; say '['translate('abcdef','123',zz)']'   ->  [abcdef]
/// zz = left('abc',0) ; say '['translate('abcdef','123',zz)']' ->  [      ]
/// ```
///
/// This crate answers `[abcdef]` for both, because its value model has no
/// null-string singleton to be identical to and no rule that would give one
/// meaning. Reproducing it would mean giving a string's *identity*
/// observable meaning, which nothing else in Rexx has.
pub(crate) fn translate(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let out_table = optional_string(interp, args, 2);
    let in_table = optional_string(interp, args, 3);
    let pad = pad_byte(interp, name, args, 4)?;
    let start = whole_number(interp, name, args, 5)?;
    let range = whole_number(interp, name, args, 6)?;

    if out_table.is_none() && in_table.is_none() && pad.is_none() {
        return case_shifted(interp, &string, start, range, u8::to_ascii_uppercase);
    }
    let out_table = out_table.unwrap_or_default();
    let pad = pad.unwrap_or(b' ');

    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    };
    let range = match range {
        Some(value) => length_of(value)?,
        None => string.len().saturating_sub(start) + 1,
    };
    if start > string.len() || range == 0 {
        return Ok(interp.text_owned(string));
    }
    let range = range.min(string.len() - start + 1);

    let mut result = string.clone();
    for byte in &mut result[start - 1..start - 1 + range] {
        let index = match &in_table {
            Some(table) => table.iter().position(|entry| entry == byte),
            None => Some(usize::from(*byte)),
        };
        if let Some(index) = index {
            *byte = out_table.get(index).copied().unwrap_or(pad);
        }
    }
    Ok(interp.text_owned(result))
}

/// `VERIFY(string, reference [,option] [,start] [,range])`.
///
/// `N` (the default) answers where the first byte *outside* `reference` is,
/// `M` where the first byte inside it is.
///
/// **An empty `reference` under `N` answers the start position whatever the
/// range**, which is the one place a zero range does not mean "no answer":
/// measured, `verify('abc','',,,0)` is 1 and `verify('abc','',,2,0)` is 2,
/// where the same calls with a non-empty reference are 0.
///
/// **The two branches test different letters, and collapsing them into one
/// flag is wrong.** An empty reference asks `opt == VERIFY_MATCH` and a
/// non-empty one asks `opt == VERIFY_NOMATCH` (`StringUtil::verify`), so an
/// option that is neither -- the `0x00` byte [`option_letter`] admits -- takes
/// the *second* arm of both. Measured, and this pair is what separates them:
/// `verify('abcde','','00'x)` is 1, the answer `'N'` gives, while
/// `verify('abcde','abc','00'x)` is 1, the answer `'M'` gives.
pub(crate) fn verify(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let reference = required_string(interp, args, 2);
    let option = optional_string(interp, args, 3);
    let start = whole_number(interp, name, args, 4)?;
    let range = whole_number(interp, name, args, 5)?;

    let option = option_letter(option.as_deref(), "MN")?.unwrap_or(b'N');
    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    };
    let range = match range {
        Some(value) => length_of(value)?,
        None => string.len().saturating_sub(start) + 1,
    };

    if start > string.len() {
        return Ok(interp.text(b"0"));
    }
    let range = range.min(string.len() - start + 1);
    let answer = if reference.is_empty() {
        // `if (opt == VERIFY_MATCH) return 0; else return startPos;`
        if option == b'M' { 0 } else { start }
    } else {
        // `if (opt == VERIFY_NOMATCH) ... else ...`, the other letter.
        let matching = option != b'N';
        string[start - 1..start - 1 + range]
            .iter()
            .position(|&byte| in_set(byte, &reference) == matching)
            .map_or(0, |offset| start + offset)
    };
    Ok(interp.text(answer.to_string().as_bytes()))
}

/// `LOWER(string [,n] [,length])`.
pub(crate) fn lower(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let start = whole_number(interp, name, args, 2)?;
    let range = whole_number(interp, name, args, 3)?;
    case_shifted(interp, &string, start, range, u8::to_ascii_lowercase)
}

/// `UPPER(string [,n] [,length])`.
pub(crate) fn upper(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let start = whole_number(interp, name, args, 2)?;
    let range = whole_number(interp, name, args, 3)?;
    case_shifted(interp, &string, start, range, u8::to_ascii_uppercase)
}

/// The body `LOWER`, `UPPER` and `TRANSLATE`'s no-table form share.
///
/// It is one function because the oracle makes it one: `RexxString::translate`
/// forwards to `upperRexx` when neither table and no pad was given, so the
/// defaults, the range capping and the two range checks are the same code
/// there as well.
/// `start` and `range` arrive already converted but not yet range-checked, so
/// this is where 93.924 and 93.923 come from for all three.
///
/// **A start past the end and a zero range are both no-ops, not errors**:
/// measured, `lower('ABCDEF',9)` and `lower('ABCDEF',3,0)` both answer their
/// argument unchanged.
fn case_shifted(
    interp: &mut Interp,
    string: &[u8],
    start: Option<i64>,
    range: Option<i64>,
    shift: fn(&u8) -> u8,
) -> Result<ObjRef, Failure> {
    let start = match start {
        Some(value) => position_of(value)?,
        None => 1,
    } - 1;
    let range = match range {
        Some(value) => length_of(value)?,
        None => string.len(),
    };
    if start >= string.len() {
        return Ok(interp.text(string));
    }
    let range = range.min(string.len() - start);
    if range == 0 {
        return Ok(interp.text(string));
    }
    let mut result = string.to_vec();
    for byte in &mut result[start..start + range] {
        *byte = shift(byte);
    }
    Ok(interp.text_owned(result))
}

#[cfg(test)]
mod tests {
    use super::super::dispatch;
    use crate::error::Failure;
    use crate::{Interp, error::Raised};

    /// Runs `name` over `arguments`, each `None` standing for an omitted
    /// interior position, and answers the result's own bytes.
    ///
    /// Goes through [`dispatch`] rather than calling the implementation
    /// directly, so every case here also exercises the arity check and the
    /// name lookup that a real call would.
    fn call(name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
        let mut interp = Interp::new();
        let args: Vec<_> = arguments
            .iter()
            .map(|argument| argument.map(|bytes| interp.text(bytes)))
            .collect();
        let result = dispatch(&mut interp, name, &args).expect("a builtin name")?;
        Ok(interp.to_text(result).into_owned())
    }

    /// `call`, for the cases whose answer is the bytes and nothing else.
    fn answer(name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
        call(name, &arguments).expect("this call succeeds")
    }

    /// The `(major, sub)` and substitutions of the condition `name` raises.
    fn raised(name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
        let failure = call(name, arguments).expect_err("this call raises");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        (raised.number, raised.sub, raised.additional)
    }

    /// The oracle transcripts each of these lines came from are in the
    /// function's own doc comment; this is the same table run through
    /// `dispatch`.
    #[test]
    fn the_padding_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"CENTER", &[b"ab", b"6", b"-"]), b"--ab--");
        assert_eq!(answer(b"CENTRE", &[b"ab", b"6", b"-"]), b"--ab--");
        assert_eq!(answer(b"CENTER", &[b"ab", b"6"]), b"  ab  ");
        assert_eq!(answer(b"CENTER", &[b"abcdef", b"3"]), b"bcd");
        assert_eq!(answer(b"CENTER", &[b"abcdef", b"2"]), b"cd");
        // The odd pad and the odd truncation both go right, which an
        // even-width case cannot tell from their going left.
        assert_eq!(answer(b"CENTER", &[b"ab", b"5", b"-"]), b"-ab--");
        assert_eq!(answer(b"CENTRE", &[b"ab", b"5", b"-"]), b"-ab--");
        assert_eq!(answer(b"CENTER", &[b"abc", b"6", b"-"]), b"-abc--");
        assert_eq!(answer(b"CENTER", &[b"abcdef", b"5"]), b"abcde");
        assert_eq!(answer(b"CENTER", &[b"abcde", b"2"]), b"bc");
        assert_eq!(answer(b"CENTER", &[b"The blue sky", b"7"]), b"e blue ");
        assert_eq!(answer(b"CENTER", &[b"abc", b"0"]), b"");
        assert_eq!(answer(b"CENTER", &[b"abc", b"3"]), b"abc");
        assert_eq!(answer(b"CENTER", &[b"", b"4", b"*"]), b"****");

        assert_eq!(answer(b"LEFT", &[b"ab", b"5", b"."]), b"ab...");
        assert_eq!(answer(b"LEFT", &[b"ab", b"5"]), b"ab   ");
        assert_eq!(answer(b"LEFT", &[b"abcdef", b"3"]), b"abc");
        assert_eq!(answer(b"LEFT", &[b"abc", b"0"]), b"");

        assert_eq!(answer(b"RIGHT", &[b"ab", b"5", b"."]), b"...ab");
        assert_eq!(answer(b"RIGHT", &[b"abcdef", b"3"]), b"def");
        assert_eq!(answer(b"RIGHT", &[b"abc", b"0"]), b"");
    }

    #[test]
    fn the_extraction_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2", b"3"]), b"bcd");
        assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2"]), b"bcdef");
        assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2", b"8"]), b"bcdef   ");
        assert_eq!(
            answer(b"SUBSTR", &[b"abcdef", b"2", b"8", b"."]),
            b"bcdef..."
        );
        assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"7"]), b"");
        assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"7", b"3", b"."]), b"...");
        assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2", b"0"]), b"");
        assert_eq!(
            call(b"SUBSTR", &[Some(b"abcdef"), Some(b"3"), None, Some(b".")])
                .expect("an omitted length is legal past the minimum"),
            b"cdef"
        );

        assert_eq!(answer(b"DELSTR", &[b"abcdef", b"3", b"2"]), b"abef");
        assert_eq!(answer(b"DELSTR", &[b"abcdef", b"3"]), b"ab");
        assert_eq!(answer(b"DELSTR", &[b"abcdef", b"1", b"6"]), b"");
        assert_eq!(answer(b"DELSTR", &[b"abcdef", b"9", b"2"]), b"abcdef");
        assert_eq!(answer(b"DELSTR", &[b"abcdef"]), b"");
        assert_eq!(answer(b"DELSTR", &[b"abcdef", b"3", b"0"]), b"abcdef");
        assert_eq!(
            call(b"DELSTR", &[Some(b"abcdef"), None, Some(b"2")]).expect("n defaults to 1"),
            b"cdef"
        );

        assert_eq!(answer(b"REVERSE", &[b"abcdef"]), b"fedcba");
        assert_eq!(answer(b"REVERSE", &[b""]), b"");
        assert_eq!(answer(b"REVERSE", &[b"a"]), b"a");
    }

    #[test]
    fn the_splicing_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"INSERT", &[b"-", b"abc", b"1"]), b"a-bc");
        assert_eq!(answer(b"INSERT", &[b"-", b"abc"]), b"-abc");
        assert_eq!(answer(b"INSERT", &[b"-", b"abc", b"0"]), b"-abc");
        assert_eq!(answer(b"INSERT", &[b"-", b"abc", b"5"]), b"abc  -");
        assert_eq!(answer(b"INSERT", &[b"XY", b"abc", b"1", b"4"]), b"aXY  bc");
        assert_eq!(
            answer(b"INSERT", &[b"XY", b"abc", b"1", b"4", b"."]),
            b"aXY..bc"
        );
        assert_eq!(answer(b"INSERT", &[b"XY", b"abc", b"1", b"1"]), b"aXbc");
        assert_eq!(answer(b"INSERT", &[b"XY", b"abc", b"1", b"0"]), b"abc");
        assert_eq!(answer(b"INSERT", &[b"", b"abc", b"2"]), b"abc");
        assert_eq!(
            answer(b"INSERT", &[b"123", b"abc", b"5", b"6", b"+"]),
            b"abc++123+++"
        );
        assert_eq!(answer(b"INSERT", &[b"", b"", b"3", b"0"]), b"   ");
        assert_eq!(answer(b"INSERT", &[b"", b"", b"3", b"1"]), b"    ");

        assert_eq!(answer(b"OVERLAY", &[b"XY", b"abcdef", b"3"]), b"abXYef");
        assert_eq!(answer(b"OVERLAY", &[b"XY", b"abcdef"]), b"XYcdef");
        assert_eq!(
            answer(b"OVERLAY", &[b"XY", b"abcdef", b"3", b"4"]),
            b"abXY  "
        );
        assert_eq!(
            answer(b"OVERLAY", &[b"XY", b"abcdef", b"3", b"4", b"."]),
            b"abXY.."
        );
        assert_eq!(
            answer(b"OVERLAY", &[b"XY", b"abcdef", b"3", b"1"]),
            b"abXdef"
        );
        assert_eq!(answer(b"OVERLAY", &[b"XY", b"abcdef", b"8"]), b"abcdef XY");
        assert_eq!(answer(b"OVERLAY", &[b"XY", b"abc", b"3", b"0"]), b"abc");
        assert_eq!(answer(b"OVERLAY", &[b"", b"abcdef", b"3"]), b"abcdef");
        assert_eq!(answer(b"OVERLAY", &[b"", b"abc", b"6", b"1"]), b"abc   ");
        assert_eq!(answer(b"OVERLAY", &[b"", b"abc", b"3", b"1"]), b"ab ");
        assert_eq!(answer(b"OVERLAY", &[b"XY", b"abc", b"4", b"0"]), b"abc");
        assert_eq!(answer(b"OVERLAY", &[b"XY", b"abc", b"5", b"0"]), b"abc ");
        assert_eq!(answer(b"OVERLAY", &[b"qq", b"abcd", b"4"]), b"abcqq");
    }

    #[test]
    fn the_searching_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"POS", &[b"an", b"banana"]), b"2");
        assert_eq!(answer(b"POS", &[b"an", b"banana", b"3"]), b"4");
        assert_eq!(answer(b"POS", &[b"an", b"banana", b"5"]), b"0");
        assert_eq!(answer(b"POS", &[b"an", b"banana", b"1", b"3"]), b"2");
        assert_eq!(answer(b"POS", &[b"an", b"banana", b"1", b"2"]), b"0");
        assert_eq!(answer(b"POS", &[b"", b"banana"]), b"0");
        assert_eq!(answer(b"POS", &[b"a", b""]), b"0");
        assert_eq!(answer(b"POS", &[b"a", b"banana", b"9"]), b"0");
        assert_eq!(answer(b"POS", &[b"a", b"BANANA"]), b"0");
        assert_eq!(answer(b"POS", &[b"e", b"abcdeeeeef", b"2", b"4"]), b"5");
        assert_eq!(answer(b"POS", &[b"eee", b"abcdeeeeef", b"5", b"2"]), b"0");

        assert_eq!(answer(b"LASTPOS", &[b"an", b"banana"]), b"4");
        assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"4"]), b"2");
        assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"3"]), b"2");
        assert_eq!(answer(b"LASTPOS", &[b"a", b"banana", b"3"]), b"2");
        assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"6", b"2"]), b"0");
        assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"6", b"3"]), b"4");
        assert_eq!(answer(b"LASTPOS", &[b"", b"banana"]), b"0");
        assert_eq!(answer(b"LASTPOS", &[b"a", b"banana", b"99"]), b"6");
        assert_eq!(answer(b"LASTPOS", &[b"abc", b"xxabc", b"5"]), b"3");
        assert_eq!(answer(b"LASTPOS", &[b"abc", b"xxabc", b"4"]), b"0");
        assert_eq!(answer(b"LASTPOS", &[b"a", b"aaaaabcdef", b"9", b"5"]), b"5");
        assert_eq!(
            answer(b"LASTPOS", &[b"a", b"aaaaabcdef", b"10", b"5"]),
            b"0"
        );
        // The range default is the whole haystack and not what is left of it
        // from the start; a needle before the start position is the only way
        // to see the difference, and the explicit range is the other half.
        assert_eq!(answer(b"LASTPOS", &[b"b", b"banana", b"5"]), b"1");
        assert_eq!(answer(b"LASTPOS", &[b"b", b"banana", b"5", b"2"]), b"0");

        assert_eq!(answer(b"COUNTSTR", &[b"a", b"banana"]), b"3");
        assert_eq!(answer(b"COUNTSTR", &[b"an", b"banana"]), b"2");
        assert_eq!(answer(b"COUNTSTR", &[b"aa", b"aaaa"]), b"2");
        assert_eq!(answer(b"COUNTSTR", &[b"", b"abc"]), b"0");
        assert_eq!(answer(b"COUNTSTR", &[b"abc", b""]), b"0");

        assert_eq!(answer(b"COMPARE", &[b"abcde", b"abcde"]), b"0");
        assert_eq!(answer(b"COMPARE", &[b"abcde", b"abXde"]), b"3");
        assert_eq!(answer(b"COMPARE", &[b"abc", b"abc "]), b"0");
        assert_eq!(answer(b"COMPARE", &[b"abc", b"abc."]), b"4");
        assert_eq!(answer(b"COMPARE", &[b"abc", b"abc.", b"."]), b"0");
        assert_eq!(answer(b"COMPARE", &[b"", b"a"]), b"1");
        assert_eq!(answer(b"COMPARE", &[b"abc", b"ab"]), b"3");
        assert_eq!(answer(b"COMPARE", &[b"ab", b"abc"]), b"3");
        assert_eq!(answer(b"COMPARE", &[b"abc", b"ab", b"c"]), b"0");

        assert_eq!(answer(b"ABBREV", &[b"Print", b"Pri"]), b"1");
        assert_eq!(answer(b"ABBREV", &[b"Print", b"Pro"]), b"0");
        assert_eq!(answer(b"ABBREV", &[b"Print", b""]), b"1");
        assert_eq!(answer(b"ABBREV", &[b"Print", b"", b"1"]), b"0");
        assert_eq!(answer(b"ABBREV", &[b"Print", b"Pri", b"4"]), b"0");
        assert_eq!(answer(b"ABBREV", &[b"Print", b"Pri", b"3"]), b"1");
        assert_eq!(answer(b"ABBREV", &[b"", b""]), b"1");
        assert_eq!(answer(b"ABBREV", &[b"", b"x"]), b"0");
        assert_eq!(answer(b"ABBREV", &[b"Print", b"PRI"]), b"0");
        assert_eq!(answer(b"ABBREV", &[b"Print", b"Printer"]), b"0");
    }

    #[test]
    fn the_rewriting_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"CHANGESTR", &[b"a", b"banana", b"X"]), b"bXnXnX");
        assert_eq!(
            answer(b"CHANGESTR", &[b"a", b"banana", b"X", b"2"]),
            b"bXnXna"
        );
        assert_eq!(
            answer(b"CHANGESTR", &[b"a", b"banana", b"X", b"0"]),
            b"banana"
        );
        assert_eq!(answer(b"CHANGESTR", &[b"a", b"banana", b""]), b"bnn");
        assert_eq!(answer(b"CHANGESTR", &[b"", b"banana", b"X"]), b"banana");
        assert_eq!(
            answer(b"CHANGESTR", &[b"an", b"banana", b"ANA"]),
            b"bANAANAa"
        );
        assert_eq!(answer(b"CHANGESTR", &[b"z", b"banana", b"X"]), b"banana");
        assert_eq!(answer(b"CHANGESTR", &[b"a", b"", b"X"]), b"");

        assert_eq!(answer(b"COPIES", &[b"ab", b"3"]), b"ababab");
        assert_eq!(answer(b"COPIES", &[b"ab", b"0"]), b"");
        assert_eq!(answer(b"COPIES", &[b"", b"5"]), b"");
        assert_eq!(answer(b"COPIES", &[b"ab", b"1"]), b"ab");
        assert_eq!(answer(b"COPIES", &[b"ab", b" -0 "]), b"");

        assert_eq!(answer(b"SPACE", &[b"a   b  c"]), b"a b c");
        assert_eq!(answer(b"SPACE", &[b"a   b  c", b"2"]), b"a  b  c");
        assert_eq!(answer(b"SPACE", &[b"a   b  c", b"0"]), b"abc");
        assert_eq!(answer(b"SPACE", &[b"a   b  c", b"2", b"-"]), b"a--b--c");
        assert_eq!(answer(b"SPACE", &[b"   "]), b"");
        assert_eq!(answer(b"SPACE", &[b""]), b"");
        assert_eq!(answer(b"SPACE", &[b"  a  "]), b"a");
        assert_eq!(answer(b"SPACE", &[b"a\tb"]), b"a b");
        assert_eq!(answer(b"SPACE", &[b"a\nb"]), b"a\nb");
    }

    #[test]
    fn the_option_taking_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"STRIP", &[b"  ab  "]), b"ab");
        assert_eq!(answer(b"STRIP", &[b"  ab  ", b"L"]), b"ab  ");
        assert_eq!(answer(b"STRIP", &[b"  ab  ", b"T"]), b"  ab");
        assert_eq!(answer(b"STRIP", &[b"  ab  ", b"B"]), b"ab");
        assert_eq!(answer(b"STRIP", &[b"  ab  ", b"l"]), b"ab  ");
        assert_eq!(answer(b"STRIP", &[b"  ab  ", b"Leading"]), b"ab  ");
        assert_eq!(answer(b"STRIP", &[b"xyabyx", b"B", b"xy"]), b"ab");
        assert_eq!(answer(b"STRIP", &[b"   "]), b"");
        assert_eq!(answer(b"STRIP", &[b"a\tb\t"]), b"a\tb");
        assert_eq!(answer(b"STRIP", &[b"abc", b"B", b""]), b"abc");
        assert_eq!(
            answer(b"STRIP", &[b"+-+-a-+b-+-+", b"L", b"-+"]),
            b"a-+b-+-+"
        );
        assert_eq!(
            call(b"STRIP", &[Some(b"xxabxx"), None, Some(b"x")]).expect("the option defaults to B"),
            b"ab"
        );

        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc"]), b"4");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abcde"]), b"0");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"M"]), b"1");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"xyz", b"M"]), b"0");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"m"]), b"1");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"Nope"]), b"4");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b""]), b"1");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"", b"M"]), b"0");
        assert_eq!(answer(b"VERIFY", &[b"", b"abc"]), b"0");
        assert_eq!(answer(b"VERIFY", &[b"abc", b"", b"N", b"2", b"0"]), b"2");
        assert_eq!(
            answer(b"VERIFY", &[b"abcde", b"ab", b"N", b"1", b"2"]),
            b"0"
        );
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"N", b"9"]), b"0");
        assert_eq!(
            answer(b"VERIFY", &[b"ABCDEF", b"ABC", b"N", b"2", b"3"]),
            b"4"
        );
        assert_eq!(
            answer(b"VERIFY", &[b"ABCDEF", b"ADEF", b"M", b"2", b"3"]),
            b"4"
        );
    }

    #[test]
    fn the_case_shifting_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"LOWER", &[b"ABCdef"]), b"abcdef");
        assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3"]), b"ABcdef");
        assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3", b"2"]), b"ABcdEF");
        assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"9"]), b"ABCDEF");
        assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3", b"0"]), b"ABCDEF");
        assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3", b"99"]), b"ABcdef");
        assert_eq!(answer(b"UPPER", &[b"abcDEF"]), b"ABCDEF");
        assert_eq!(answer(b"UPPER", &[b"abcdef", b"3", b"2"]), b"abCDef");
        // Only ASCII folds; measured, `upper('e9'x)` is its own argument.
        assert_eq!(answer(b"UPPER", &[&[0xe9]]), &[0xe9]);
        assert_eq!(answer(b"LOWER", &[&[0xc9]]), &[0xc9]);

        assert_eq!(answer(b"TRANSLATE", &[b"abcdef"]), b"ABCDEF");
        assert_eq!(
            answer(b"TRANSLATE", &[b"abcdef", b"123", b"abc"]),
            b"123def"
        );
        assert_eq!(answer(b"TRANSLATE", &[b"abcdef", b"123"]), b"      ");
        assert_eq!(
            answer(b"TRANSLATE", &[b"abcdef", b"12", b"abcd"]),
            b"12  ef"
        );
        assert_eq!(
            answer(b"TRANSLATE", &[b"abcdef", b"1234", b"ab"]),
            b"12cdef"
        );
        assert_eq!(
            answer(b"TRANSLATE", &[b"abcdef", b"12", b"abcd", b"."]),
            b"12..ef"
        );
        assert_eq!(answer(b"TRANSLATE", &[b"abcdef", b"", b""]), b"abcdef");
        assert_eq!(
            answer(b"TRANSLATE", &[b"abcABC", b"123", b"abc"]),
            b"123ABC"
        );
        assert_eq!(answer(b"TRANSLATE", &[b"aXbXc", b"12", b"XX"]), b"a1b1c");
        assert_eq!(answer(b"TRANSLATE", &[b"4123", b"abcd", b"1234"]), b"dabc");
        assert_eq!(
            answer(
                b"TRANSLATE",
                &[b"abcdef", b"123456", b"aaabbbcc", b".", b"2", b"3"]
            ),
            b"a4.def"
        );
        assert_eq!(
            call(
                b"TRANSLATE",
                &[Some(b"abcdef"), None, None, None, Some(b"2"), Some(b"3")]
            )
            .expect("with no tables and no pad this is UPPER"),
            b"aBCDef"
        );
        assert_eq!(
            call(b"TRANSLATE", &[Some(b"abcdef"), None, Some(b"abc")])
                .expect("an omitted tableout is the null string"),
            b"   def"
        );
        assert_eq!(
            call(b"TRANSLATE", &[Some(b"abcdef"), None, None, Some(b".")])
                .expect("a pad alone still translates"),
            b"......"
        );
    }

    /// The 40.x call-layer family, with the routine name and the *call's* own
    /// argument position substituted.
    #[test]
    fn a_bad_argument_kind_names_the_routine_and_the_call_position() {
        assert_eq!(
            raised(b"LEFT", &[Some(b"ab"), Some(b"q")]),
            (40, 12, vec![b"LEFT".to_vec(), b"2".to_vec(), b"q".to_vec()])
        );
        assert_eq!(
            raised(
                b"SUBSTR",
                &[Some(b"abc"), Some(b"2"), Some(b"3"), Some(b"pq")]
            ),
            (
                40,
                23,
                vec![b"SUBSTR".to_vec(), b"4".to_vec(), b"pq".to_vec()]
            )
        );
        assert_eq!(
            raised(
                b"TRANSLATE",
                &[Some(b"abc"), None, None, Some(b"$"), Some(b"1"), Some(b"q")]
            ),
            (
                40,
                12,
                vec![b"TRANSLATE".to_vec(), b"6".to_vec(), b"q".to_vec()]
            )
        );
        assert_eq!(
            raised(
                b"INSERT",
                &[
                    Some(b"-"),
                    Some(b"abc"),
                    Some(b"1"),
                    Some(b"2"),
                    Some(b"pq")
                ]
            ),
            (
                40,
                23,
                vec![b"INSERT".to_vec(), b"5".to_vec(), b"pq".to_vec()]
            )
        );
        // A pad is refused whether or not it could be used, and a value
        // needing more than ARGUMENT_DIGITS is not a whole number.
        assert_eq!(
            raised(b"LEFT", &[Some(b""), Some(b"0"), Some(b"xx")]),
            (
                40,
                23,
                vec![b"LEFT".to_vec(), b"3".to_vec(), b"xx".to_vec()]
            )
        );
        assert_eq!(
            raised(b"LEFT", &[Some(b"ab"), Some(b"1E18")]),
            (
                40,
                12,
                vec![b"LEFT".to_vec(), b"2".to_vec(), b"1E18".to_vec()]
            )
        );
        assert_eq!(
            raised(b"COPIES", &[Some(b"ab"), Some(b"")]),
            (40, 12, vec![b"COPIES".to_vec(), b"2".to_vec(), Vec::new()])
        );

        // `CENTRE` is `CENTER`'s implementation under its own name, and the
        // message is where the two are told apart.
        assert_eq!(
            raised(b"CENTRE", &[Some(b"ab"), Some(b"6"), Some(b"--")]),
            (
                40,
                23,
                vec![b"CENTRE".to_vec(), b"3".to_vec(), b"--".to_vec()]
            )
        );
        assert_eq!(
            raised(b"CENTER", &[Some(b"ab"), Some(b"6"), Some(b"--")]),
            (
                40,
                23,
                vec![b"CENTER".to_vec(), b"3".to_vec(), b"--".to_vec()]
            )
        );
    }

    /// The 93.9xx operation-layer family, which names neither the routine nor
    /// the position and reports the *converted* value.
    #[test]
    fn a_bad_argument_range_reports_the_converted_value() {
        assert_eq!(
            raised(b"LEFT", &[Some(b"ab"), Some(b"-1.0")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"SUBSTR", &[Some(b"abc"), Some(b"0.0")]),
            (93, 924, vec![b"0".to_vec()])
        );
        assert_eq!(
            raised(b"SUBSTR", &[Some(b"abc"), Some(b"2"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"COPIES", &[Some(b"ab"), Some(b"-1.0")]),
            (93, 906, vec![b"1".to_vec(), b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"INSERT", &[Some(b"a"), Some(b"b"), Some(b"-1")]),
            (93, 906, vec![b"2".to_vec(), b"-1".to_vec()])
        );
        assert_eq!(
            raised(
                b"CHANGESTR",
                &[Some(b"a"), Some(b"b"), Some(b"c"), Some(b"-1")]
            ),
            (93, 906, vec![b"3".to_vec(), b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"STRIP", &[Some(b"ab"), Some(b"Xyz")]),
            (93, 915, vec![b"BLT".to_vec(), b"Xyz".to_vec()])
        );
        assert_eq!(
            raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(b"")]),
            (93, 915, vec![b"MN".to_vec(), Vec::new()])
        );
        // A range error inside a builtin whose start is out of range is
        // still raised, which is the ordering `delstr`'s own comment names.
        assert_eq!(
            raised(b"DELSTR", &[Some(b"abc"), Some(b"9"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
    }

    /// Every conversion runs before every range check, at the three shapes
    /// that can tell the two layers apart.
    ///
    /// The adjacent success matters as much as the refusal: with the bad pad
    /// removed, each of these calls reaches the range error it was hiding.
    #[test]
    fn the_call_layer_is_checked_before_the_operation_layer() {
        assert_eq!(
            raised(
                b"SUBSTR",
                &[Some(b"abc"), Some(b"0"), Some(b"5"), Some(b"xx")]
            ),
            (
                40,
                23,
                vec![b"SUBSTR".to_vec(), b"4".to_vec(), b"xx".to_vec()]
            )
        );
        assert_eq!(
            raised(
                b"SUBSTR",
                &[Some(b"abc"), Some(b"0"), Some(b"5"), Some(b"x")]
            ),
            (93, 924, vec![b"0".to_vec()])
        );

        assert_eq!(
            raised(
                b"TRANSLATE",
                &[Some(b"abc"), None, None, Some(b"$"), Some(b"0"), Some(b"q")]
            ),
            (
                40,
                12,
                vec![b"TRANSLATE".to_vec(), b"6".to_vec(), b"q".to_vec()]
            )
        );
        assert_eq!(
            raised(
                b"TRANSLATE",
                &[Some(b"abc"), None, None, Some(b"$"), Some(b"0"), Some(b"1")]
            ),
            (93, 924, vec![b"0".to_vec()])
        );

        assert_eq!(
            raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(b"X"), Some(b"q")]),
            (
                40,
                12,
                vec![b"VERIFY".to_vec(), b"4".to_vec(), b"q".to_vec()]
            )
        );
        assert_eq!(
            raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(b"X"), Some(b"1")]),
            (93, 915, vec![b"MN".to_vec(), b"X".to_vec()])
        );
    }

    /// A result too large to allocate is the oracle's own Error 5, not a
    /// process abort.
    ///
    /// `999999999999999999` is the largest whole number `ARGUMENT_DIGITS`
    /// admits, so this is the pair either side of the boundary: one more
    /// digit is a 40.12 at the call layer and never reaches the allocator.
    #[test]
    fn a_result_too_large_to_allocate_is_the_oracles_own_error_5() {
        let failure =
            call(b"LEFT", &[Some(b"ab"), Some(b"999999999999999999")]).expect_err("too large");
        let Failure::Raised(exhausted) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((exhausted.number, exhausted.sub), (5, 0));
        assert_eq!(exhausted.exit_code(), 251);
        assert!(exhausted.additional.is_empty());

        assert_eq!(
            raised(b"LEFT", &[Some(b"ab"), Some(b"1234567890123456789")]),
            (
                40,
                12,
                vec!["LEFT".into(), "2".into(), "1234567890123456789".into()]
            )
        );

        for name in [b"COPIES".as_slice(), b"CENTER", b"SPACE", b"RIGHT"] {
            let subject: &[u8] = if name == b"SPACE" { b"a b" } else { b"ab" };
            let failure =
                call(name, &[Some(subject), Some(b"123456789012345678")]).expect_err("too large");
            let Failure::Raised(exhausted) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!(
                (exhausted.number, exhausted.sub),
                (5, 0),
                "{} did not report the allocation failure",
                String::from_utf8_lossy(name)
            );
        }
    }

    /// The whole-number conversion runs at `ARGUMENT_DIGITS`, not at the
    /// caller's `NUMERIC DIGITS`, and the two disagree in both directions.
    ///
    /// Measured on the oracle: `numeric digits 2 ; left('ab','1.0000001')`
    /// is 40.12 where a two-digit conversion would have rounded it whole,
    /// and `left('ab','1.0000000000000000000004')` is `a` because rounding
    /// *that* to eighteen digits leaves exactly 1.
    #[test]
    fn the_whole_number_conversion_uses_the_argument_precision() {
        assert_eq!(
            raised(b"LEFT", &[Some(b"ab"), Some(b"1.0000001")]),
            (
                40,
                12,
                vec![b"LEFT".to_vec(), b"2".to_vec(), b"1.0000001".to_vec()]
            )
        );
        assert_eq!(answer(b"LEFT", &[b"ab", b"1.0000000000000000000004"]), b"a");
        // The generous spellings the oracle accepts, each measured.
        assert_eq!(answer(b"LEFT", &[b"ab", b" 5 "]), b"ab   ");
        assert_eq!(answer(b"LEFT", &[b"ab", b"+5"]), b"ab   ");
        assert_eq!(answer(b"LEFT", &[b"ab", b"1e1"]), b"ab        ");
        assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2e0"]), b"bcdef");
    }

    /// A count or offset is created as text, so `NUMERIC DIGITS` cannot
    /// reach it.
    ///
    /// The builtins here are the ones that answer a number rather than a
    /// slice of their argument; the mutation this catches is building the
    /// result through `Interp::number` under the settings in force.
    #[test]
    fn a_counting_builtin_answers_text_that_no_digits_setting_reshapes() {
        let mut interp = Interp::new();
        for (name, arguments) in [
            (b"POS".as_slice(), [b"a".as_slice(), b"bbbbbbbbba"]),
            (b"LASTPOS", [b"a", b"bbbbbbbbba"]),
            (b"COMPARE", [b"bbbbbbbbba", b"bbbbbbbbbz"]),
            (b"COUNTSTR", [b"a", b"aaaaaaaaaa"]),
            (b"VERIFY", [b"bbbbbbbbba", b"b"]),
            (b"LENGTH", [b"bbbbbbbbba", b"bbbbbbbbba"]),
        ] {
            let take = if name == b"LENGTH" { 1 } else { 2 };
            let args: Vec<_> = arguments[..take]
                .iter()
                .map(|bytes| Some(interp.text(bytes)))
                .collect();
            let result = dispatch(&mut interp, name, &args)
                .expect("a builtin name")
                .expect("this call succeeds");
            assert_eq!(
                interp.to_text(result).into_owned(),
                b"10",
                "{} did not answer plain text",
                String::from_utf8_lossy(name)
            );
        }
    }

    /// The condition every 93.9xx here carries is rc 163, not the 40.x
    /// family's 216, which is the observable that made the two families
    /// worth separating.
    #[test]
    fn the_two_argument_families_exit_differently() {
        let failure = call(b"SUBSTR", &[Some(b"abc"), Some(b"2"), Some(b"-1")])
            .expect_err("a negative length");
        let Failure::Raised(range) = failure else {
            panic!("expected Raised");
        };
        assert_eq!(range.exit_code(), 163);

        let failure =
            call(b"SUBSTR", &[Some(b"abc"), Some(b"x")]).expect_err("a non-numeric position");
        let Failure::Raised(kind) = failure else {
            panic!("expected Raised");
        };
        assert_eq!(kind.exit_code(), 216);
    }

    /// A substitution carries the argument's own bytes, and the report
    /// applies the oracle's display rule to them.
    ///
    /// The two halves are separate defects and each needs its own witness. A
    /// byte at or above `0x80` reaches the message intact -- a `String`-shaped
    /// substitution would turn `FF` into U+FFFD's three bytes -- while a
    /// control byte reaches it as `?`. Both measured, and both live in the
    /// committed ooTest groups: `COPIES` test095/231/372/538, `DELSTR`
    /// test17, `INSERT` test035/066 and `SUBSTR` test019/048 pass a
    /// high-byte argument where a whole number is required.
    #[test]
    fn a_substitution_carries_bytes_and_the_report_makes_them_displayable() {
        assert_eq!(
            raised(b"COPIES", &[Some(b"ab"), Some(&[0xff])]),
            (40, 12, vec![b"COPIES".to_vec(), b"2".to_vec(), vec![0xff]])
        );
        assert_eq!(
            raised(b"LEFT", &[Some(b"ab"), Some(b"5"), Some(&[0x00, 0x12])]),
            (
                40,
                23,
                vec![b"LEFT".to_vec(), b"3".to_vec(), vec![0x00, 0x12]]
            )
        );

        // The rule is applied by `Raised::report`, so the check is on the
        // rendered line rather than on the stored substitution.
        let site = crate::error::ClauseSite {
            sites: &[],
            path: "/p.rex",
        };
        let high = Raised::argument_not_whole(b"COPIES", 2, &[0xff]);
        assert!(
            high.report(&site)
                .windows(4)
                .any(|w| w == [b'"', 0xff, b'"', b'.']),
            "a byte at or above 0x80 must reach the report unchanged"
        );
        let control = Raised::argument_not_a_pad(b"LEFT", 3, &[0x00, 0x12]);
        assert!(
            control.report(&site).windows(5).any(|w| w == *b"\"??\"."),
            "a control byte must reach the report as a question mark"
        );
        // Tab, carriage return and line feed are the three that stay, and
        // without them this rule would be "every byte below 0x20".
        let kept = Raised::argument_not_a_pad(b"LEFT", 3, b"\t\r\n");
        assert!(
            kept.report(&site).windows(5).any(|w| w == *b"\"\t\r\n\""),
            "tab, carriage return and line feed are not sanitised"
        );
    }

    /// A `0x00` first byte is an accepted option letter that is none of the
    /// letters, and each caller answers with its own "neither" branch.
    ///
    /// See [`option_letter`] for the `strchr` reading this comes from. The
    /// adjacent refusal is what pins it to the NUL rather than to control
    /// bytes generally.
    #[test]
    fn a_null_option_byte_is_accepted_and_matches_no_letter() {
        assert_eq!(answer(b"STRIP", &[b"  ab  ", &[0x00]]), b"  ab  ");
        assert_eq!(answer(b"STRIP", &[b"  ab  ", &[0x00, b'L']]), b"  ab  ");
        assert_eq!(answer(b"STRIP", &[b"  ab  ", &[b'L', 0x00]]), b"ab  ");
        // `VERIFY`'s two branches test different letters, so the option that
        // is neither takes the second arm of each -- and the two arms give
        // *opposite* letters' answers. With a non-empty reference the test is
        // for `N`, so `0x00` answers as `M` does; with an empty one the test
        // is for `M`, so `0x00` answers as `N` does. A single flag for both
        // gets exactly one of these two lines wrong.
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", &[0x00]]), b"1");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"", &[0x00]]), b"1");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"", &[0x00], b"3"]), b"3");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"xyz", &[0x00]]), b"0");
        // The four letter cases either side of them, which is what pins each
        // arm to the letter it actually tests.
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"M"]), b"1");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"N"]), b"4");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"", b"M"]), b"0");
        assert_eq!(answer(b"VERIFY", &[b"abcde", b"", b"N"]), b"1");

        assert_eq!(
            raised(b"STRIP", &[Some(b"ab"), Some(&[0x01])]),
            (93, 915, vec![b"BLT".to_vec(), vec![0x01]])
        );
        assert_eq!(
            raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(&[0x01])]),
            (93, 915, vec![b"MN".to_vec(), vec![0x01]])
        );
    }

    /// A raiser this module owns renders through the catalogue, so a wrong
    /// sub-number shows up as a message that is not the oracle's.
    #[test]
    fn the_new_raisers_render_the_oracles_own_message_text() {
        let cases = [
            (
                Raised::argument_not_whole(b"LEFT", 2, b"q"),
                "LEFT argument 2 must be a whole number; found \"q\".",
            ),
            (
                Raised::argument_not_a_pad(b"LEFT", 3, b"xx"),
                "LEFT argument 3 must be a single character; found \"xx\".",
            ),
            (
                Raised::invalid_length(b"-1"),
                "Invalid length argument specified; found \"-1\".",
            ),
            (
                Raised::invalid_position(b"0"),
                "Invalid position argument specified; found \"0\".",
            ),
            (
                Raised::argument_not_non_negative(1, b"-1"),
                "Method argument 1 must be zero or a positive whole number; found \"-1\".",
            ),
            (
                Raised::invalid_option("BLT", b"Xyz"),
                "Method option must be one of \"BLT\"; found \"Xyz\".",
            ),
            (Raised::system_resources(), "System resources exhausted."),
        ];
        for (raised, expected) in cases {
            let rendered = String::from_utf8(raised.report(&crate::error::ClauseSite {
                sites: &[],
                path: "/p.rex",
            }))
            .expect("the catalogue is UTF-8");
            assert!(
                rendered.contains(expected),
                "{rendered:?} does not carry {expected:?}"
            );
        }
    }
}
