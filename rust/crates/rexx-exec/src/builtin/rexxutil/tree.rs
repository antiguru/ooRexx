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

//! `SysFileTree`: `TreeFinder` in `interpreter/runtime/RexxUtilCommon.cpp`
//! and its Unix half in `platform/unix/SysRexxUtil.cpp`.

use rexx_core::ObjRef;
use rustix::fs::{FileType, Stat};

use super::files::{CivilTime, Found, OPTION_MESSAGE_LIMIT, local_time, program_zone};
use super::{answer, c_string, check_arity, optional_text, os_path, required, required_c_string};
use crate::builtin::platform::{expand_tilde, qualify_name};
use crate::error::Raised;
use crate::{Failure, Interp};

/// What one call looks for and how it reports what it finds, from the option
/// letters.
#[derive(Clone, Copy)]
struct TreeOptions {
    files: bool,
    directories: bool,
    recurse: bool,
    /// `O`: the name alone, with no date, size or attributes.
    names_only: bool,
    /// `T`: the date as `YY/MM/DD/HH/MM`.
    editable_time: bool,
    /// `L`: the date as `YYYY-MM-DD HH:MM:SS`, over `T`.
    long_time: bool,
    /// `H`: the size twenty wide and never capped.
    long_size: bool,
    /// `I`: the final name matched without case.
    caseless: bool,
}

impl TreeOptions {
    /// `TreeFinder::goodOpts`: the letters in any case, each of `F`, `D` and
    /// `B` replacing what an earlier one chose. `None` for any other letter.
    fn parse(text: &[u8]) -> Option<TreeOptions> {
        let mut options = TreeOptions {
            files: true,
            directories: true,
            recurse: false,
            names_only: false,
            editable_time: false,
            long_time: false,
            long_size: false,
            caseless: false,
        };
        for letter in text {
            match letter.to_ascii_uppercase() {
                b'F' => (options.files, options.directories) = (true, false),
                b'D' => (options.files, options.directories) = (false, true),
                b'B' => (options.files, options.directories) = (true, true),
                b'S' => options.recurse = true,
                b'O' => options.names_only = true,
                b'T' => options.editable_time = true,
                b'L' => options.long_time = true,
                b'H' => options.long_size = true,
                b'I' => options.caseless = true,
                _ => return None,
            }
        }
        Some(options)
    }
}

/// A 40.900 whose message `snprintf` formats into 256 bytes.
fn user_defined(mut text: Vec<u8>) -> Failure {
    text.truncate(OPTION_MESSAGE_LIMIT);
    Raised::routine_user_defined(&text).into()
}

/// `TreeFinder::parseMask`: an attribute mask, cut at a NUL, is empty or up
/// to five of `+`, `-` and `*`. Unix applies neither mask.
fn check_mask(mask: Option<&[u8]>, position: usize) -> Result<(), Failure> {
    let mask = mask.map_or(&[][..], c_string);
    if mask.len() > 5 || mask.iter().any(|byte| !matches!(byte, b'+' | b'-' | b'*')) {
        let mut text = format!(
            "SysFileTree argument {position} must be 5 characters or less in length \
             containing only '+', '-', or '*'; found \""
        )
        .into_bytes();
        text.extend_from_slice(mask);
        text.push(b'"');
        return Err(user_defined(text));
    }
    Ok(())
}

/// `SysFileSystem::canonicalizeName` over a name starting with `~`: the
/// tilde resolved where it can be, and the result normalised where it is
/// absolute. What does not resolve or is not absolute stays as it is.
fn canonicalize_tilde(interp: &Interp, name: Vec<u8>) -> Vec<u8> {
    let expanded = expand_tilde(interp, &name).unwrap_or(name);
    if expanded.first() == Some(&b'/') {
        crate::paths::normalize_bytes(&expanded, b"/")
    } else {
        expanded
    }
}

/// `TreeFinder::adjustDirectory`: `*` becomes `./*`, a spec ending in `/`,
/// `/.` or `/..` gains a wildcard, and a leading `~` is resolved.
fn adjust_directory(interp: &Interp, spec: &[u8]) -> Vec<u8> {
    let mut spec = spec.to_vec();
    if spec == b"*" {
        spec = b"./*".to_vec();
    } else if spec.ends_with(b"/") {
        spec.push(b'*');
    } else if spec.ends_with(b"/.") || spec.ends_with(b"/..") {
        spec.extend_from_slice(b"/*");
    }
    if spec.first() == Some(&b'~') {
        spec = canonicalize_tilde(interp, spec);
    }
    spec
}

/// `TreeFinder::getFullPath`: the directory to search, ending in `/`, and
/// the pattern for the names in it.
///
/// A spec with no `/` is searched for in the current directory, unqualified,
/// and `..` searches the directory named by the current one's name with `..`
/// appended. Measured, `SysFileTree('..', s.)` run from `/` lists `/../bin`.
/// The directory part of any other spec is qualified, and where that fails
/// the search is in `/`.
fn full_path(interp: &Interp, spec: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (mut directory, pattern) = match spec.iter().rposition(|byte| *byte == b'/') {
        None => {
            let mut directory = interp.cwd_bytes().to_vec();
            let pattern = match spec {
                b"." => b"*".to_vec(),
                b".." => {
                    directory.extend_from_slice(b"..");
                    b"*".to_vec()
                }
                _ => spec.to_vec(),
            };
            (directory, pattern)
        }
        Some(slash) => {
            let pattern = if spec.ends_with(b"/") {
                b"*".to_vec()
            } else {
                spec[slash + 1..].to_vec()
            };
            (qualify_name(interp, &spec[..=slash]), pattern)
        }
    };
    if directory.last() != Some(&b'/') {
        directory.push(b'/');
    }
    (directory, pattern)
}

/// `SysFileTree(filespec, stem [, options [, tattrib [, nattrib]]])`: the
/// names matching `filespec`, into a stem or an array. The directory part of
/// the spec is literal and only its final name may hold wildcards, matched as
/// `fnmatch(3)` matches them.
///
/// Errors after the stem is resolved still set its tail `0` to `0`, as
/// `StemHandler`'s destructor does. The order entries are found in is the
/// directory's own, which is not specified.
pub(crate) fn file_tree(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 5)?;
    let spec = required_c_string(interp, args, 1)?;
    let target = required(args, 2)?;
    let options = optional_text(interp, args, 3)?;
    let target_mask = optional_text(interp, args, 4)?;
    let new_mask = optional_text(interp, args, 5)?;
    let found = Found::resolve(interp, target, 2)?;

    let options = match validate(
        &spec,
        options.as_deref(),
        target_mask.as_deref(),
        new_mask.as_deref(),
    ) {
        Ok(options) => options,
        Err(failure) => {
            found.complete(interp, 0);
            return Err(failure);
        }
    };
    let spec = adjust_directory(interp, &spec);
    let (directory, pattern) = full_path(interp, &spec);
    let zone = program_zone(interp);
    let mut lines = Vec::new();
    search(&directory, &pattern, options, &zone, &mut lines);
    let count = lines.len();
    for (index, line) in lines.into_iter().enumerate() {
        found.add(interp, index + 1, line)?;
    }
    found.complete(interp, count);
    Ok(answer(interp, 0))
}

/// The arguments' checks in the order `TreeFinder`'s constructor makes them:
/// the spec, the options, then the two masks.
fn validate(
    spec: &[u8],
    options: Option<&[u8]>,
    target_mask: Option<&[u8]>,
    new_mask: Option<&[u8]>,
) -> Result<TreeOptions, Failure> {
    if spec.is_empty() {
        return Err(Raised::routine_null_argument("SysFileTree", 1).into());
    }
    let options = match options.map(c_string) {
        None => TreeOptions::parse(b"").expect("no letters is valid"),
        Some([]) => return Err(Raised::routine_null_argument("SysFileTree", 3).into()),
        Some(letters) => TreeOptions::parse(letters).ok_or_else(|| {
            let mut text = b"SysFileTree options argument must be a combination of \
                             F, D, B, S, T, L, I, O, or H; found \""
                .to_vec();
            text.extend_from_slice(letters);
            text.push(b'"');
            user_defined(text)
        })?,
    };
    check_mask(target_mask, 4)?;
    check_mask(new_mask, 5)?;
    Ok(options)
}

/// `TreeFinder::recursiveFindFile`: the entries of `directory` whose names
/// match `pattern`, then, under `S`, every subdirectory's the same way. A
/// directory that does not open contributes nothing.
fn search(
    directory: &[u8],
    pattern: &[u8],
    options: TreeOptions,
    zone: &tz::TimeZone,
    out: &mut Vec<Vec<u8>>,
) {
    for (name, stat) in entries(directory, |name| matches(pattern, name, options.caseless)) {
        let mut path = directory.to_vec();
        path.extend_from_slice(&name);
        if let Some(line) = entry_line(options, zone, path, &stat) {
            out.push(line);
        }
    }
    if options.recurse {
        for (name, stat) in entries(directory, |_| true) {
            if FileType::from_raw_mode(stat.st_mode) != FileType::Directory {
                continue;
            }
            let mut path = directory.to_vec();
            path.extend_from_slice(&name);
            path.push(b'/');
            search(&path, pattern, options, zone, out);
        }
    }
}

/// The names in `directory` other than `.` and `..` that `wanted` accepts,
/// each with what `stat` answers for it, following a symbolic link. Only
/// those names are stat'ed, as `SysFileIterator` does. Where `stat` fails (a
/// dangling link, a chain of links past the kernel's limit) it is `lstat`'s
/// answer: the oracle reports whatever its buffer last held
/// (`corpus/oracle-crashes.txt` entry 40). A name both fail on, removed since
/// it was read, is left out.
fn entries(directory: &[u8], wanted: impl Fn(&[u8]) -> bool) -> Vec<(Vec<u8>, Stat)> {
    use std::os::unix::ffi::OsStrExt as _;
    let Ok(listing) = std::fs::read_dir(os_path(directory)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in listing.flatten() {
        let name = entry.file_name().as_bytes().to_vec();
        if !wanted(&name) {
            continue;
        }
        let mut path = directory.to_vec();
        path.extend_from_slice(&name);
        let stat =
            rustix::fs::stat(path.as_slice()).or_else(|_| rustix::fs::lstat(path.as_slice()));
        if let Ok(stat) = stat {
            out.push((name, stat));
        }
    }
    out
}

/// The reading for a time the calendar cannot hold.
const EPOCH: CivilTime = CivilTime {
    year: 1970,
    month0: 0,
    day: 1,
    hour: 0,
    minute: 0,
    second: 0,
};

/// `TreeFinder::checkFile` and `formatFileAttributes`: the line for one
/// entry, or `None` where `F` or `D` leaves its kind out. The qualified name
/// alone under `O`, otherwise the date, the size, the mode string and the
/// name, two blanks after each of the first three.
fn entry_line(
    options: TreeOptions,
    zone: &tz::TimeZone,
    path: Vec<u8>,
    stat: &Stat,
) -> Option<Vec<u8>> {
    let kind = FileType::from_raw_mode(stat.st_mode);
    let wanted = if kind == FileType::Directory {
        options.directories
    } else {
        options.files
    };
    if !wanted {
        return None;
    }
    if options.names_only {
        return Some(path);
    }
    let when = local_time(zone, stat.st_mtime).unwrap_or(EPOCH);
    let month = when.month0 + 1;
    let two_digit_year = (when.year - 1900 + 10_000) % 100;
    let mut out = if options.long_time {
        format!(
            "{:4}-{month:02}-{:02} {:02}:{:02}:{:02}  ",
            when.year, when.day, when.hour, when.minute, when.second
        )
    } else if options.editable_time {
        format!(
            "{two_digit_year:02}/{month:02}/{:02}/{:02}/{:02}  ",
            when.day, when.hour, when.minute
        )
    } else {
        let hour = if when.hour < 13 && when.hour != 0 {
            when.hour
        } else {
            (when.hour - 12).abs()
        };
        let meridiem = if when.hour < 12 { 'a' } else { 'p' };
        format!(
            "{month:2}/{:02}/{two_digit_year:02}  {hour:2}:{:02}{meridiem}  ",
            when.day, when.minute
        )
    };
    if options.long_size {
        out.push_str(&format!("{:20}  ", stat.st_size));
    } else {
        out.push_str(&format!("{:10}  ", stat.st_size.min(9_999_999_999)));
    }
    let mut out = out.into_bytes();
    out.extend_from_slice(&mode_string(kind, stat.st_mode));
    out.extend_from_slice(b"  ");
    out.extend_from_slice(&path);
    Some(out)
}

/// The ten-character mode string `ls -l` shows: the kind, then each class's
/// permissions with set-user-ID, set-group-ID and sticky folded into the
/// execute letters.
fn mode_string(kind: FileType, mode: u32) -> [u8; 10] {
    let letter = match kind {
        FileType::Symlink => b'l',
        FileType::BlockDevice => b'b',
        FileType::CharacterDevice => b'c',
        FileType::Directory => b'd',
        FileType::Fifo => b'p',
        FileType::Socket => b's',
        FileType::RegularFile | FileType::Unknown => b'-',
    };
    let bit = |mask: u32, set: u8| if mode & mask == 0 { b'-' } else { set };
    let execute = |special: u32, execute: u32, both: u8, special_only: u8| match (
        mode & special != 0,
        mode & execute != 0,
    ) {
        (true, true) => both,
        (true, false) => special_only,
        (false, true) => b'x',
        (false, false) => b'-',
    };
    [
        letter,
        bit(0o400, b'r'),
        bit(0o200, b'w'),
        execute(0o4000, 0o100, b's', b'S'),
        bit(0o040, b'r'),
        bit(0o020, b'w'),
        execute(0o2000, 0o010, b's', b'S'),
        bit(0o004, b'r'),
        bit(0o002, b'w'),
        execute(0o1000, 0o001, b't', b'T'),
    ]
}

/// `fnmatch(3)` with `FNM_PATHNAME | FNM_NOESCAPE` in the C locale, over one
/// name, folding ASCII case under `caseless` as `FNM_CASEFOLD` does: `*`,
/// `?`, and a bracket expression with ranges, `[:class:]`, `[.c.]` and
/// `[=c=]` forms and a leading `!` or `^`. A backslash is an ordinary byte, and a leading `.` needs no
/// literal match.
fn matches(pattern: &[u8], name: &[u8], caseless: bool) -> bool {
    let fold = |byte: u8| {
        if caseless {
            byte.to_ascii_lowercase()
        } else {
            byte
        }
    };
    // The classic backtracking walk: `star` remembers where the last `*` was
    // so a failed tail can resume one byte later.
    let (mut pattern_at, mut name_at) = (0, 0);
    let (mut star, mut resume) = (None, 0);
    while name_at < name.len() {
        let step = match pattern.get(pattern_at) {
            Some(b'*') => {
                star = Some(pattern_at);
                resume = name_at;
                pattern_at += 1;
                continue;
            }
            Some(b'?') => Some(pattern_at + 1),
            Some(b'[') => bracket(pattern, pattern_at, name[name_at], caseless),
            Some(byte) if fold(*byte) == fold(name[name_at]) => Some(pattern_at + 1),
            _ => None,
        };
        match (step, star) {
            (Some(next), _) => {
                pattern_at = next;
                name_at += 1;
            }
            (None, Some(at)) => {
                pattern_at = at + 1;
                resume += 1;
                name_at = resume;
            }
            (None, None) => return false,
        }
    }
    pattern[pattern_at..].iter().all(|byte| *byte == b'*')
}

/// One bracket expression opening at `open` against one byte: the index
/// just past its `]` when it matches, `None` when it does not. A bracket with
/// no closing `]` is a literal `[`.
///
/// Measured against glibc through the oracle: a `[.c.]` or `[=c=]` naming one
/// byte is that byte, compared without folding case, and `[.c.]` may end a
/// range. A `[.` with no `.]`, a collating symbol of any other length, and
/// `[=c=]` ending a range make the whole pattern fail. A `[=` not closed by
/// `=]` two bytes on is a literal `[`, and `[=c=]` followed by `-` starts no
/// range.
fn bracket(pattern: &[u8], open: usize, byte: u8, caseless: bool) -> Option<usize> {
    let fold = |value: u8| {
        if caseless {
            value.to_ascii_lowercase()
        } else {
            value
        }
    };
    let mut at = open + 1;
    if matches!(pattern.get(at), Some(b'!' | b'^')) {
        at += 1;
    }
    let negated = at > open + 1;
    let mut hit = false;
    let mut first = true;
    while at < pattern.len() {
        // A `]` in the first position is the literal character, not the close.
        if pattern[at] == b']' && !first {
            return (hit != negated).then_some(at + 1);
        }
        first = false;
        if pattern[at] == b'['
            && pattern.get(at + 1) == Some(&b':')
            && let Some(length) = pattern[at + 2..].windows(2).position(|pair| pair == b":]")
        {
            let class = &pattern[at + 2..at + 2 + length];
            if !in_class(class, byte) {
                // An unknown class name makes the whole pattern fail.
                if !is_class(class) {
                    return None;
                }
            } else {
                hit = true;
            }
            at += length + 4;
            continue;
        }
        if let Some(equivalent) = equivalence(pattern, at) {
            if equivalent == byte {
                hit = true;
            }
            at += 5;
            continue;
        }
        let (low, low_end, symbol) = match pattern.get(at + 1) {
            Some(b'.') if pattern[at] == b'[' => {
                let (low, end) = collating(pattern, at)?;
                (low, end, true)
            }
            _ => (pattern[at], at + 1, false),
        };
        if pattern.get(low_end) == Some(&b'-')
            && pattern.get(low_end + 1).is_some_and(|end| *end != b']')
        {
            let (high, high_end) = match (pattern[low_end + 1], pattern.get(low_end + 2)) {
                (b'[', Some(b'.')) => collating(pattern, low_end + 1)?,
                (b'[', Some(b'=')) if equivalence(pattern, low_end + 1).is_some() => return None,
                (high, _) => (high, low_end + 2),
            };
            if (fold(low)..=fold(high)).contains(&fold(byte)) {
                hit = true;
            }
            at = high_end;
        } else {
            let matched = if symbol {
                low == byte
            } else {
                fold(low) == fold(byte)
            };
            if matched {
                hit = true;
            }
            at = low_end;
        }
    }
    (byte == b'[').then_some(open + 1)
}

/// The byte a `[.c.]` at `at` names and the index past it, `None` where it
/// does not close or names other than one byte.
fn collating(pattern: &[u8], at: usize) -> Option<(u8, usize)> {
    let length = pattern
        .get(at + 2..)?
        .windows(2)
        .position(|pair| pair == b".]")?;
    (length == 1).then(|| (pattern[at + 2], at + length + 4))
}

/// The byte a well-formed `[=c=]` at `at` names.
fn equivalence(pattern: &[u8], at: usize) -> Option<u8> {
    match pattern.get(at..at + 5)? {
        [b'[', b'=', c, b'=', b']'] => Some(*c),
        _ => None,
    }
}

/// Whether `name` is a character class the C locale defines.
fn is_class(name: &[u8]) -> bool {
    matches!(
        name,
        b"alnum"
            | b"alpha"
            | b"blank"
            | b"cntrl"
            | b"digit"
            | b"graph"
            | b"lower"
            | b"print"
            | b"punct"
            | b"space"
            | b"upper"
            | b"xdigit"
    )
}

/// Whether `byte` is in the C locale's character class `name`.
fn in_class(name: &[u8], byte: u8) -> bool {
    match name {
        b"alnum" => byte.is_ascii_alphanumeric(),
        b"alpha" => byte.is_ascii_alphabetic(),
        b"blank" => byte == b' ' || byte == b'\t',
        b"cntrl" => byte.is_ascii_control(),
        b"digit" => byte.is_ascii_digit(),
        b"graph" => byte.is_ascii_graphic(),
        b"lower" => byte.is_ascii_lowercase(),
        b"print" => byte.is_ascii_graphic() || byte == b' ',
        b"punct" => byte.is_ascii_punctuation(),
        b"space" => matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'),
        b"upper" => byte.is_ascii_uppercase(),
        b"xdigit" => byte.is_ascii_hexdigit(),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
