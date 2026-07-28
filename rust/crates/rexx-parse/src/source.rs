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

//! Retained program source and physical line lookup (`SOURCELINE`).
//!
//! Ported from `BufferProgramSource::buildDescriptors`
//! (`ProgramSource.cpp:348`). A Rexx source file is bytes, not text: a
//! literal may hold arbitrary bytes that are not valid UTF-8 (measured: a
//! literal containing a raw `FF FE` runs fine, `c2x` reports `FFFE`), so the
//! retained source is `Vec<u8>` and lines are `&[u8]`, never `String`/`&str`.

use std::ops::Range;

/// The retained text of one Rexx program, indexed by physical line.
///
/// Built once at construction; `line` and `line_of` are read-only lookups
/// over that index, never a re-scan, because error reporting calls
/// `line_of` on every diagnostic (see later tasks) and parse throughput is
/// measured (Task 3.10).
pub struct ProgramSource {
    /// The program text. Truncated at the first Ctrl-Z (0x1A) byte, if any --
    /// see `new` for why.
    text: Vec<u8>,
    /// Byte range of each physical line's content, in order, with any line
    /// terminator excluded. Index 0 holds line 1 (`SOURCELINE` is 1-based).
    /// Starts are strictly increasing, which is what makes `line_of`'s
    /// binary search valid.
    lines: Vec<(usize, usize)>,
}

impl ProgramSource {
    /// Builds the line index for `text`.
    ///
    /// A line ends at a `\r`, a `\n`, or end of input, whichever comes
    /// first; a `\r` immediately followed by `\n` is one terminator (CRLF),
    /// not two. This means a bare `\r` (no `\n`) ends a line on its own, and
    /// a `\n` immediately followed by `\r` is two terminators, producing an
    /// empty line between them -- both verified against `build/bin/rexx`
    /// (`ProgramSource.cpp:387`-`441` scans for either byte and only
    /// special-cases `\r` followed by `\n`).
    ///
    /// The interpreter also treats a Ctrl-Z (0x1A) byte as an end-of-file
    /// mark, a legacy DOS/CP-M artifact: everything at and after the first
    /// occurrence is discarded before line scanning even starts
    /// (`ProgramSource.cpp:373`-`377`), including a partial line up to that
    /// byte. Verified: `build/bin/rexx` on a file with a mid-line 0x1A
    /// truncates the line's text at that byte and reports a shorter
    /// `sourceline()` count; a comment left unclosed by the truncation
    /// raises the ordinary unmatched-comment error, confirming the bytes
    /// after 0x1A are never parsed at all.
    pub fn new(mut text: Vec<u8>) -> Self {
        let scan_len = text.iter().position(|&b| b == 0x1a).unwrap_or(text.len());
        text.truncate(scan_len);
        let len = text.len();

        let mut lines = Vec::new();
        let mut pos = 0;
        while pos < len {
            let start = pos;
            match text[pos..].iter().position(|&b| b == b'\r' || b == b'\n') {
                None => {
                    // No terminator: the remainder of the file is the last line.
                    lines.push((start, len));
                    pos = len;
                }
                Some(rel) => {
                    let delim = pos + rel;
                    lines.push((start, delim));
                    if text[delim] == b'\r' {
                        let mut next = delim + 1;
                        // Pair a CR with an immediately following LF into one
                        // terminator; a lone CR (or a CR followed by anything
                        // else) ends the line by itself.
                        if next < len && text[next] == b'\n' {
                            next += 1;
                        }
                        pos = next;
                    } else {
                        pos = delim + 1;
                    }
                }
            }
        }

        ProgramSource { text, lines }
    }

    /// The number of physical lines, as `SOURCELINE()` with no argument
    /// reports it. A completely empty program has zero lines (verified via
    /// `ProgramSource.cpp:387`'s `while (bufferLength != 0)`, which never
    /// runs for an empty buffer, so `lineCount` stays 0).
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// The text of physical line `n` (1-based), with its terminator
    /// excluded, or `None` if `n` is 0 or past `line_count()`.
    ///
    /// The interpreter does not return an empty string out of range:
    /// `sourceline(0)` raises error 40.14 and `sourceline(99)` past the end
    /// raises 40.34 (both verified against `build/bin/rexx`). `None` is how
    /// this crate reports that; a later task turns it into the right error
    /// number. Do not let it render as `""`.
    pub fn line(&self, n: usize) -> Option<&[u8]> {
        let index = n.checked_sub(1)?;
        let &(start, end) = self.lines.get(index)?;
        Some(&self.text[start..end])
    }

    /// The line's content range in the retained text, terminator excluded,
    /// or `None` if `n` is 0 or past `line_count()`. `line(n)` returns
    /// exactly `&text[line_span(n)?]`.
    ///
    /// This is what lets the scanner work on a line at a time and still
    /// report absolute byte offsets: it adds the line's start to every
    /// in-line offset. The terminator rules stay here, in one place, rather
    /// than being re-derived from the source bytes by a second scanner.
    pub fn line_span(&self, n: usize) -> Option<Range<usize>> {
        let index = n.checked_sub(1)?;
        let &(start, end) = self.lines.get(index)?;
        Some(start..end)
    }

    /// The 1-based physical line containing byte offset `byte`.
    ///
    /// A byte that sits on a line terminator belongs to the line that
    /// terminator ends. A byte at or past the end of the retained text
    /// clamps to the last line rather than failing, so a caller never has to
    /// bounds-check an offset before reporting a diagnostic. An empty source
    /// answers 1, which is the only case where the answer names a line that
    /// `line` will not return.
    ///
    /// Total by construction, so it returns `usize` rather than `Option`.
    pub fn line_of(&self, byte: usize) -> usize {
        // `partition_point` counts the starts that are `<= byte`; because
        // starts are 0-based and strictly increasing, that count is exactly
        // the 1-based line number containing `byte` (and clamps to the last
        // line for a byte past the end of the source). `.max(1)` only
        // matters for a source with zero lines, which no real byte offset
        // from a token can point into.
        self.lines
            .partition_point(|&(start, _)| start <= byte)
            .max(1)
    }
}
