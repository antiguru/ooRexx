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

use std::borrow::Cow;
use std::ops::Range;

/// Where a `ProgramSource`'s text came from, which decides how it is divided
/// into lines.
///
/// Every one of `ProgramSource::new`'s behaviours is program-only: splitting
/// on CR and LF, truncating at a Ctrl-Z, and the `#!` first line the scanner
/// skips. So this is a property of the source, fixed at construction, and not
/// of a particular parse. Expressing it in one place is what makes it
/// impossible to build a source one way and read it the other.
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
}

/// The retained text of one Rexx program, indexed by physical line.
///
/// Built once at construction; `line` and `line_of` are read-only lookups
/// over that index, never a re-scan, because error reporting calls
/// `line_of` on every diagnostic (see later tasks) and parse throughput is
/// measured (Task 3.10).
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
    ///
    /// For a `SourceKind::Program`, a line ends at a `\r`, a `\n`, or end of
    /// input, whichever comes first; a `\r` immediately followed by `\n` is
    /// one terminator (CRLF), not two. This means a bare `\r` (no `\n`) ends a
    /// line on its own, and a `\n` immediately followed by `\r` is two
    /// terminators, producing an empty line between them -- both verified
    /// against `build/bin/rexx` (`ProgramSource.cpp:387`-`441` scans for
    /// either byte and only special-cases `\r` followed by `\n`).
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
    ///
    /// A `SourceKind::Interpret` gets neither rule. Its text is one line from
    /// end to end, because `ArrayProgramSource` holds it as a single array
    /// element, so a `\n`, a `\r` or a `0x1A` inside it is just a byte on that
    /// line and the scanner rejects it as error 13.1 the way it rejects any
    /// other character that cannot appear in a program. Measured, all five:
    /// `interpret "say 1" || '0a'x || "say 2"` is error 13.1, and so are the
    /// same with `'0d'x`, with `'0d0a'x`, with `'1a'x` in the middle and with
    /// `'1a'x` at the very end. For contrast `interpret "say 1; say 2"` prints
    /// 1 and 2, so a `;` still separates clauses, and
    /// `interpret "say c2x('" || '1a'x || "')"` prints `1A`, so a Ctrl-Z
    /// inside a literal survives as data.
    ///
    /// Empty text is one empty line under `Interpret` and no lines at all
    /// under `Program`, because the interpreter's array always has its one
    /// element. Both scan to no tokens, and measured, `interpret ""` is
    /// accepted and the program runs on.
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

    /// The bytes a token or clause span covers, or `None` if the span runs
    /// past the end of the retained text or backwards.
    ///
    /// Spans are absolute offsets into the whole text, not offsets within a
    /// line, so a caller cannot slice them out of `line`: a span may sit on
    /// any line, and a clause's span crosses lines whenever the clause is
    /// continued. This is the accessor that turns one back into bytes, and it
    /// is deliberately the *only* way out: a whole-text getter would let a
    /// caller re-derive line boundaries from the bytes, which is exactly what
    /// the terminator rules above exist to prevent.
    ///
    /// Every span the scanner produces is in range. `None` is for a span that
    /// came from somewhere else.
    pub fn span_bytes(&self, span: Range<usize>) -> Option<&[u8]> {
        self.text.get(span)
    }

    /// The bytes a clause span covers with the line terminators inside the
    /// span removed, which is the text `TRACE` prints on a `*-*` line: a
    /// continued clause's fragments are joined by dropping the terminator
    /// between them and keeping every other byte, including the continuation
    /// line's leading blanks and a terminating `;`.
    ///
    /// Ported from `ProgramSource::extract` (`ProgramSource.cpp:153`), whose
    /// multi-line branch concatenates `getStringLine` results, and those are
    /// line content with terminators excluded. Measured under `trace r`:
    /// `say "x",` continued as `    "y"` traces as `say "x",    "y"`, and the
    /// CRLF spelling of the same file traces the identical text.
    ///
    /// Borrowed exactly when the span contains no terminator byte, which is
    /// every uncontinued clause's span and every empty span. Owned exactly
    /// when the join dropped something. `None` exactly when `span_bytes`
    /// answers `None`: a span that runs past the end of the retained text or
    /// backwards.
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

impl std::fmt::Debug for ProgramSource {
    /// Reports the shape rather than the text.
    ///
    /// A derived `Debug` would dump the whole program into every failing
    /// assertion that mentions a `ProgramSource`, which is what made the type
    /// undebuggable in practice rather than merely underived.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgramSource")
            .field("kind", &self.kind)
            .field("lines", &self.lines.len())
            .field("bytes", &self.text.len())
            .finish()
    }
}
