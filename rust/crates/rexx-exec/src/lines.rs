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

//! Line positions of a text file: the line walkers and the line mark and
//! count on an [`rexx_core::OpenFile`], shared by the stream class and the
//! file console.

/// How much of a file one read asks the system for.
pub(crate) const CHUNK: usize = 4096;

/// A validated positive argument as a character position. The validator
/// answers `u64` because it rejects zero and everything below it; the
/// positions it feeds are signed because a later seek can drive them
/// negative.
pub(crate) fn as_position(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// A logical character position as a file offset. A position below 1 is not
/// a legal offset -- a `SEEK` can drive one negative and the oracle's `lseek`
/// fails there -- so it clamps, and the read or write then finds nothing.
pub(crate) fn offset_of(position: i64) -> u64 {
    u64::try_from(position).unwrap_or(0)
}

/// Bytes from `position` (1-based), stopping at the end of the file.
pub(crate) fn read_from(
    file: &mut std::fs::File,
    position: u64,
    len: usize,
) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    file.seek(SeekFrom::Start(position.saturating_sub(1)))?;
    // Grown as bytes arrive, so a count past the end allocates no more than
    // the file holds.
    let mut out = Vec::new();
    file.take(u64::try_from(len).unwrap_or(u64::MAX))
        .read_to_end(&mut out)?;
    Ok(out)
}

/// The lines from `position` to the end, counted the way `SysFile::countLines`
/// counts them: an unterminated final line counts.
pub(crate) fn count_lines_from(file: &mut std::fs::File, position: u64) -> std::io::Result<u64> {
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

/// The file's size, or 0 when it cannot be stat'd.
pub(crate) fn size_of(file: &std::fs::File) -> u64 {
    file.metadata().map_or(0, |meta| meta.len())
}

/// The lines a stream holds, for a `<n` line seek: an unterminated last line
/// counts, and on a record stream a partial last record does.
pub(crate) fn line_count(
    open: &mut rexx_core::OpenFile,
    record_length: u64,
) -> std::io::Result<u64> {
    let size = size_of(&open.file);
    if let Some(whole) = size.checked_div(record_length) {
        return Ok(whole + u64::from(!size.is_multiple_of(record_length)));
    }
    if let Some(total) = open.line_total {
        if cfg!(debug_assertions) && total_intact(open) {
            assert_eq!(
                Some(total),
                count_lines_from(&mut open.file, 1).ok(),
                "the cached line count"
            );
        }
        return Ok(total);
    }
    let total = count_lines_from(&mut open.file, 1)?;
    open.line_total = Some(total);
    if cfg!(debug_assertions) {
        open.line_total_digest = digest_before(&mut open.file, size as i64 + 1);
    }
    Ok(total)
}

/// Debug builds: a digest of the bytes before character `upto`.
pub(crate) fn digest_before(file: &mut std::fs::File, upto: i64) -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let bytes = read_from(file, 1, usize::try_from(upto - 1).ok()?).ok()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    Some(hasher.finish())
}

/// Debug builds: whether the bytes the line mark was counted over are as
/// they were.
pub(crate) fn mark_intact(open: &mut rexx_core::OpenFile) -> bool {
    match (open.line_mark, open.line_mark_digest) {
        (Some((mark, _)), Some(digest)) => digest_before(&mut open.file, mark) == Some(digest),
        _ => false,
    }
}

/// Debug builds: whether the file the line count was counted over is as it
/// was.
pub(crate) fn total_intact(open: &mut rexx_core::OpenFile) -> bool {
    let end = size_of(&open.file) as i64 + 1;
    open.line_total_digest.is_some() && digest_before(&mut open.file, end) == open.line_total_digest
}

/// Sets the line mark. In a debug build its digest is kept only when the
/// line was counted from bytes known unchanged, so a mark built on a stale
/// one is never checked.
pub(crate) fn set_mark(open: &mut rexx_core::OpenFile, position: i64, line: i64, trusted: bool) {
    open.line_mark = Some((position, line));
    open.line_mark_digest = if cfg!(debug_assertions) && trusted {
        digest_before(&mut open.file, position)
    } else {
        None
    };
}

/// The lines of a text stream from `position` on, which is not past its end:
/// the stream's lines less those before the one `position` lies in.
pub(crate) fn lines_after(open: &mut rexx_core::OpenFile, position: i64) -> std::io::Result<u64> {
    let total = line_count(open, 0)?;
    let before = u64::try_from(line_of(open, position, 0)? - 1).unwrap_or(0);
    let answer = total.saturating_sub(before);
    if cfg!(debug_assertions) && total_intact(open) && mark_intact(open) {
        assert_eq!(
            Some(answer),
            count_lines_from(&mut open.file, offset_of(position)).ok(),
            "the lines after a position, from the cached count"
        );
    }
    Ok(answer)
}

/// The start of line `target` and the line reached: past the last line end
/// of a text stream, the end of the stream on the line after the last line
/// end (`SysFile::seekForwardLines`). Line ends are counted from the start
/// every time (rexxref iostrms.xml:476-490). A record stream's line n is
/// character `(n - 1) * reclength + 1` (iostrms.xml:465-472).
pub(crate) fn seek_line(
    open: &mut rexx_core::OpenFile,
    target: i64,
    record_length: u64,
) -> (u64, i64) {
    if target <= 1 {
        return (1, 1);
    }
    let line = u64::try_from(target).unwrap_or(1);
    if record_length > 0 {
        return (line, as_position(record_length) * (target - 1) + 1);
    }
    // Line ends are passed from the mark when it lies on an earlier line.
    let (from, from_line) = match open.line_mark {
        Some((mark, mark_line)) if mark_line < target => {
            (mark, u64::try_from(mark_line).unwrap_or(1))
        }
        _ => (1, 1),
    };
    let trusted = cfg!(debug_assertions) && (from == 1 || mark_intact(open));
    let (moved, at) =
        seek_forward_lines(&mut open.file, from, line - from_line).unwrap_or((0, from));
    let reached = from_line + moved;
    if trusted {
        assert_eq!(
            Some((reached - 1, at)),
            seek_forward_lines(&mut open.file, 1, line - 1).ok(),
            "a line walk from the mark"
        );
    }
    set_mark(open, at, as_position(reached), trusted);
    (reached, at)
}

/// Moves forward over up to `count` line ends from `position`, answering how
/// many it passed and the position after the last one, or the end of the file
/// when it ran out.
pub(crate) fn seek_forward_lines(
    file: &mut std::fs::File,
    position: i64,
    count: u64,
) -> std::io::Result<(u64, i64)> {
    let mut moved = 0;
    let mut at = position;
    while moved < count {
        let chunk = read_from(file, offset_of(at), CHUNK)?;
        if chunk.is_empty() {
            break;
        }
        for (index, byte) in chunk.iter().enumerate() {
            if *byte == b'\n' {
                moved += 1;
                if moved == count {
                    return Ok((moved, at + index as i64 + 1));
                }
            }
        }
        at += chunk.len() as i64;
    }
    Ok((moved, at))
}

/// The line character `position` lies in: one more than the line ends before
/// it (rexxref iostrms.xml:476-483), or on a record stream the record holding
/// it.
pub(crate) fn line_of(
    open: &mut rexx_core::OpenFile,
    position: i64,
    record_length: u64,
) -> std::io::Result<i64> {
    if record_length > 0 {
        return Ok((position.max(1) - 1) / as_position(record_length) + 1);
    }
    let position = position.clamp(1, size_of(&open.file) as i64 + 1);
    let (from, from_line) = match open.line_mark {
        Some((mark, mark_line)) if mark <= position => (mark, mark_line),
        _ => (1, 1),
    };
    let trusted = cfg!(debug_assertions) && (from == 1 || mark_intact(open));
    let line = from_line + line_ends_between(&mut open.file, from, position)?;
    if trusted {
        assert_eq!(
            line,
            1 + line_ends_between(&mut open.file, 1, position)?,
            "a line counted from the mark"
        );
    }
    set_mark(open, position, line, trusted);
    Ok(line)
}

/// The line ends at character positions `from` up to, not including, `to`.
pub(crate) fn line_ends_between(
    file: &mut std::fs::File,
    from: i64,
    to: i64,
) -> std::io::Result<i64> {
    let end = offset_of(to);
    let mut count = 0;
    let mut at = offset_of(from);
    while at < end {
        let want = usize::try_from((end - at).min(CHUNK as u64)).unwrap_or(CHUNK);
        let chunk = read_from(file, at, want)?;
        if chunk.is_empty() {
            break;
        }
        count += chunk.iter().filter(|byte| **byte == b'\n').count() as i64;
        at += chunk.len() as u64;
    }
    Ok(count)
}
