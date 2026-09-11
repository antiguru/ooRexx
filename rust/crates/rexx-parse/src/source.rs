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

use std::borrow::Cow;
use std::ops::Range;

/// Where a `ProgramSource`'s text came from, which decides how it is divided
/// into lines.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SourceKind {
    /// A whole program, from a file or a buffer
    /// (`BufferProgramSource`).
    Program,
    /// The string an `INTERPRET` is about to run. Exactly one physical line:
    /// the interpreter wraps the string in a one-element array
    /// (`LanguageParser.cpp:450`, `new ArrayProgramSource(new_array(
    /// interpretString), lineNumber)`), so nothing inside it can start a
    /// second line.
    Interpret,
    /// Source handed over as an array of lines, one element per physical
    /// line -- `ArrayProgramSource` with no interpret adjustment, which is
    /// what `MethodClass::newMethodObject` compiles a method body from
    /// (`classes/MethodClass.cpp:463`, `:482`).
    Lines,
}

/// The retained text of one Rexx program, indexed by physical line.
pub struct ProgramSource {
    /// The program text. For a `Program`, truncated at the first Ctrl-Z (0x1A)
    /// byte, if any -- see `new` for why.
    text: Vec<u8>,
    /// Byte range of each physical line's content, in order, with any line
    /// terminator excluded. Index 0 holds line 1 (`SOURCELINE` is 1-based).
    /// Starts are strictly increasing, which is what makes `line_of`'s
    /// binary search valid.
    lines: Vec<(usize, usize)>,
    kind: SourceKind,
}

impl ProgramSource {
    /// Builds the line index for `text`.
    pub fn new(mut text: Vec<u8>, kind: SourceKind) -> Self {
        if kind == SourceKind::Interpret {
            let lines = vec![(0, text.len())];
            return ProgramSource { text, lines, kind };
        }

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

        ProgramSource { text, lines, kind }
    }

    /// A [`SourceKind::Lines`] source: the line index is given rather than
    /// scanned, so nothing inside an element can divide it.
    pub fn from_lines(lines: &[&[u8]]) -> Self {
        let mut text = Vec::new();
        let mut index = Vec::with_capacity(lines.len());
        for (position, line) in lines.iter().enumerate() {
            // Between elements and never before the first, which is what
            // keeps every line's start strictly greater than the one before
            // it -- `line_of`'s binary search rests on that, and two empty
            // elements in a row would otherwise share a start.
            if position > 0 {
                text.push(b'\n');
            }
            let start = text.len();
            text.extend_from_slice(line);
            index.push((start, text.len()));
        }
        ProgramSource {
            text,
            lines: index,
            kind: SourceKind::Lines,
        }
    }

    /// What this source holds, which the scanner needs because a `#!` first
    /// line is skipped in a program and is an invalid character in an
    /// `INTERPRET`.
    pub fn kind(&self) -> SourceKind {
        self.kind
    }

    /// The number of physical lines, as `SOURCELINE()` with no argument
    /// reports it. A completely empty program has zero lines (verified via
    /// `ProgramSource.cpp:387`'s `while (bufferLength != 0)`, which never
    /// runs for an empty buffer, so `lineCount` stays 0). Empty `INTERPRET`
    /// text still has its one line.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// The text of physical line `n` (1-based), with its terminator
    /// excluded, or `None` if `n` is 0 or past `line_count()`.
    pub fn line(&self, n: usize) -> Option<&[u8]> {
        let index = n.checked_sub(1)?;
        let &(start, end) = self.lines.get(index)?;
        Some(&self.text[start..end])
    }

    /// The bytes a token or clause span covers, or `None` if the span runs
    /// past the end of the retained text or backwards.
    pub fn span_bytes(&self, span: Range<usize>) -> Option<&[u8]> {
        self.text.get(span)
    }

    /// The bytes a clause span covers with the line terminators inside the
    /// span removed, which is the text `TRACE` prints on a `*-*` line: a
    /// continued clause's fragments are joined by dropping the terminator
    /// between them and keeping every other byte, including the continuation
    /// line's leading blanks and a terminating `;`.
    pub fn join_span(&self, span: Range<usize>) -> Option<Cow<'_, [u8]>> {
        let bytes = self.text.get(span.clone())?;
        // An empty span has nothing to drop. It is also the only span a
        // zero-line source can produce, and there the line walk below has no
        // line to visit and would answer an owned empty, breaking the
        // borrowed/owned contract above.
        if bytes.is_empty() {
            return Some(Cow::Borrowed(bytes));
        }
        if let Some(line) = self.line_span(self.line_of(span.start))
            && line.start <= span.start
            && span.end <= line.end
        {
            return Some(Cow::Borrowed(bytes));
        }
        // A trim cannot do this: the bytes to drop are the terminators in the
        // MIDDLE of the span, one per continuation, and everything around
        // them stays. So the join walks the line index and keeps each line's
        // intersection with the span, which is also what keeps the terminator
        // rules (CRLF is one terminator, LF-CR is two) in this module instead
        // of re-derived from the bytes by a second scanner.
        let mut joined = Vec::new();
        let mut n = self.line_of(span.start);
        while let Some(line) = self.line_span(n) {
            if line.start >= span.end {
                break;
            }
            let start = line.start.max(span.start);
            let end = line.end.min(span.end);
            if start < end {
                joined.extend_from_slice(&self.text[start..end]);
            }
            n += 1;
        }
        Some(Cow::Owned(joined))
    }

    /// The line's content range in the retained text, terminator excluded,
    /// or `None` if `n` is 0 or past `line_count()`. `line(n)` returns
    /// exactly `&text[line_span(n)?]`.
    pub fn line_span(&self, n: usize) -> Option<Range<usize>> {
        let index = n.checked_sub(1)?;
        let &(start, end) = self.lines.get(index)?;
        Some(start..end)
    }

    /// The 1-based physical line containing byte offset `byte`.
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

impl std::fmt::Debug for ProgramSource {
    /// Reports the shape rather than the text.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgramSource")
            .field("kind", &self.kind)
            .field("lines", &self.lines.len())
            .field("bytes", &self.text.len())
            .finish()
    }
}
