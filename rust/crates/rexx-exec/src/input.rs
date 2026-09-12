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

//! `.input`: one line position, shared by every construct that reads a line.
//! ```text
//! 1=<line-A> 2=<line-B> 3=<line-C> 4=<line-D>
//! ```
//! ```text
//! 1=<qentry-one> 2=<line-A> 3=<qentry-two> 4=<line-B>
//! ```
//! ```text
//! " pad \n"          ->  " pad "   (5)   leading and trailing blanks kept
//! "\n"               ->  ""        (0)
//! "last-no-newline"  ->  the whole 15 bytes, at end of file
//! "crlf\r\n"         ->  "crlf"    (4)   the CR is not data
//! "a\rb\n"           ->  "a\rb"    (3)   a CR elsewhere IS data
//! "x\r\r\n"          ->  "x\r"     (2)   exactly one CR removed, not both
//! "y\r"              ->  "y\r"     (2)   at end of file, no pair to collapse
//! "a\x00b\n"         ->  "a\x00b"  (3)   NUL is data
//! ```

use std::io::{BufRead, Cursor, Read};

use crate::Interp;
use crate::invocation::ProgramInput;

/// `.input`'s position: the one line cursor every input construct advances.
pub(crate) struct Input {
    source: Source,
    /// Whether a read has already found the end of the input. **What parts a
    /// live source from a drained one**, which is a distinction the oracle
    /// makes and seekability alone does not: measured, `.stdin~chars` over a
    /// pipe answers `1` before anything is read and `0` once it is drained.
    exhausted: bool,
    /// Whether the line count has been asked for already. Measured on an
    /// unmeasurable input: `.stdin~lines` answers `1` the first time and `0`
    /// every time after, with no read in between, where `.stdin~chars` keeps
    /// answering `1`. `LINES('N')` is that first ask too.
    lines_asked: bool,
}

enum Source {
    /// Nothing to read, ever. Distinct from `Bytes` over an empty buffer only
    /// in costing no allocation; both answer `None` on the first read.
    Nothing,
    /// `std::io::Stdin` rather than a `StdinLock`, and locked per read: the
    /// handle is what `Interp` can hold without borrowing from anything, and
    /// this crate reads lines rarely enough that re-locking is not worth a
    /// lifetime for.
    Stdin(std::io::Stdin),
    Bytes(Cursor<Vec<u8>>),
}

impl Input {
    pub(crate) fn new(input: ProgramInput) -> Input {
        Input {
            source: match input {
                ProgramInput::Nothing => Source::Nothing,
                ProgramInput::Stdin => Source::Stdin(std::io::stdin()),
                ProgramInput::Bytes(bytes) => Source::Bytes(Cursor::new(bytes)),
            },
            exhausted: false,
            lines_asked: false,
        }
    }

    /// The next line, or `None` once there is nothing left to read.
    fn read_line(&mut self) -> Option<Vec<u8>> {
        let mut line = Vec::new();
        let read = match &mut self.source {
            Source::Nothing => {
                self.exhausted = true;
                return None;
            }
            Source::Stdin(stdin) => stdin.lock().read_until(b'\n', &mut line),
            Source::Bytes(cursor) => cursor.read_until(b'\n', &mut line),
        };
        match read {
            // Zero bytes with no error is end of input; a read error answers
            // the same way, per this module's doc.
            Ok(0) | Err(_) => {
                self.exhausted = true;
                return None;
            }
            Ok(_) => {}
        }
        // The terminator is not part of the line, and a `\r` immediately
        // before it is not either -- but a `\r` anywhere else is data, and a
        // final line with no newline has no pair to collapse. All three are
        // measured in the module doc's table.
        if line.last() == Some(&b'\n') {
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
        }
        Some(line)
    }

    /// Up to `wanted` bytes, answering what is **already there** rather than
    /// waiting for the rest. Measured on the oracle: `.stdin~charin(,50)`
    /// against a pipe carrying six bytes and still open answers those six
    /// immediately, so filling the request would hang the interpreter on a
    /// live descriptor. Empty once there is nothing left.
    fn read_bytes(&mut self, wanted: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; wanted];
        let read = match &mut self.source {
            Source::Nothing => {
                self.exhausted = true;
                return Vec::new();
            }
            Source::Stdin(stdin) => stdin.lock().read(&mut buffer),
            Source::Bytes(cursor) => cursor.read(&mut buffer),
        };
        let filled = match read {
            Ok(0) | Err(_) => {
                self.exhausted = true;
                0
            }
            Ok(count) => count,
        };
        buffer.truncate(filled);
        buffer
    }

    /// The lines left to read, counted the way [`Input::read_line`] would
    /// split them, when the source can be measured without consuming it.
    /// `None` when it cannot, matching [`Input::remaining`].
    fn remaining_lines(&mut self) -> Option<u64> {
        if self.exhausted {
            return Some(0);
        }
        let asked = std::mem::replace(&mut self.lines_asked, true);
        match &self.source {
            // An unmeasurable source answers `1` once and `0` after, which is
            // where this parts from [`Input::remaining`]: measured, `chars`
            // keeps answering `1` over the same input where the second
            // `lines` answers `0`.
            Source::Nothing | Source::Stdin(_) if asked => Some(0),
            Source::Nothing | Source::Stdin(_) => None,
            Source::Bytes(cursor) => {
                let at = usize::try_from(cursor.position()).unwrap_or(usize::MAX);
                let left = cursor.get_ref().get(at..).unwrap_or_default();
                if left.is_empty() {
                    return Some(0);
                }
                // A final line with no terminator still counts, which is why
                // this is not simply the newline count.
                let terminators = left.iter().filter(|byte| **byte == b'\n').count() as u64;
                Some(match left.last() {
                    Some(b'\n') => terminators,
                    _ => terminators + 1,
                })
            }
        }
    }

    /// The bytes left to read, when that is knowable without consuming them,
    /// and `None` when it is not -- a live standard input is a stream, not a
    /// length. Measured on the oracle: `.stdin~chars` answers the true
    /// remaining count for a redirected regular file, `1` for a pipe nothing
    /// has drained, and `0` for either once a read has found the end, which is
    /// what `exhausted` carries.
    fn remaining(&self) -> Option<u64> {
        if self.exhausted {
            return Some(0);
        }
        match &self.source {
            // Live but empty, exactly like a `/dev/null` descriptor: measured,
            // the oracle answers `1` there until a read drains it and `0`
            // after -- so an unread `Nothing` is not yet a measured zero.
            Source::Nothing => None,
            Source::Stdin(_) => None,
            Source::Bytes(cursor) => {
                Some((cursor.get_ref().len() as u64).saturating_sub(cursor.position()))
            }
        }
    }
}

impl Interp {
    /// One line for `PULL` and `PARSE PULL`: the queue's head if the queue has
    /// one, and otherwise the next line of `.input`.
    pub(crate) fn pull_line(&mut self) -> Vec<u8> {
        match self.queue.pop() {
            Some(line) => line,
            None => self.linein_line(),
        }
    }

    /// One line for `PARSE LINEIN`: always `.input`, never the queue.
    pub(crate) fn linein_line(&mut self) -> Vec<u8> {
        self.input.read_line().unwrap_or_default()
    }

    /// [`Interp::linein_line`] keeping the end-of-input answer apart from an
    /// empty line: `.STDIN~LINEIN` sets `NOTREADY` on the first and not the
    /// second, where `PARSE LINEIN` cannot tell them apart.
    pub(crate) fn input_line(&mut self) -> Option<Vec<u8>> {
        self.input.read_line()
    }

    /// Up to `wanted` bytes of `.input`, for `.STDIN~CHARIN`.
    pub(crate) fn input_bytes(&mut self, wanted: usize) -> Vec<u8> {
        self.input.read_bytes(wanted)
    }

    /// What `.STDIN~CHARS` answers: the bytes left, or `None` when the source
    /// cannot be measured without consuming it.
    pub(crate) fn input_remaining(&self) -> Option<u64> {
        self.input.remaining()
    }

    /// What `.STDIN~LINES` answers, as [`Interp::input_remaining`] for lines.
    /// **Takes `&mut self`**: asking an unmeasurable source for its line count
    /// is what makes the next ask answer `0`.
    pub(crate) fn input_remaining_lines(&mut self) -> Option<u64> {
        self.input.remaining_lines()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut input = Input::new(ProgramInput::Bytes(bytes.to_vec()));
        let mut read = Vec::new();
        while let Some(line) = input.read_line() {
            read.push(line);
        }
        read
    }

    /// The module doc's own measured byte table, row for row.
    #[test]
    fn a_line_is_the_bytes_before_the_terminator() {
        let cases: &[(&[u8], &[&[u8]])] = &[
            (
                b" pad \n\nlast-no-newline",
                &[b" pad ", b"", b"last-no-newline"],
            ),
            (b"crlf\r\nplain\n", &[b"crlf", b"plain"]),
            (b"a\x00b\nnext\n", &[b"a\x00b", b"next"]),
            // The three rows that separate the rule from its near-misses: a CR
            // that is not before a newline is data, exactly one CR is removed
            // from a `\r\r\n` run, and a CR at end of file with no newline
            // after it stays.
            (b"a\rb\nx\r\r\ny\r", &[b"a\rb", b"x\r", b"y\r"]),
        ];
        for (bytes, expected) in cases {
            assert_eq!(
                lines(bytes),
                expected.iter().map(|l| l.to_vec()).collect::<Vec<_>>(),
                "reading {:?}",
                String::from_utf8_lossy(bytes)
            );
        }
    }

    /// Nothing to read answers nothing, on the first read and on every read
    /// after it.
    #[test]
    fn an_empty_input_is_exhausted_from_the_start() {
        for input in [ProgramInput::Nothing, ProgramInput::Bytes(Vec::new())] {
            let mut input = Input::new(input);
            for _ in 0..3 {
                assert_eq!(input.read_line(), None);
            }
        }
    }

    /// Past the last line, a non-empty input behaves exactly like an empty
    /// one: the null string, not the last line again and not a panic.
    #[test]
    fn reading_past_the_end_keeps_answering_nothing() {
        let mut input = Input::new(ProgramInput::Bytes(b"only\n".to_vec()));
        assert_eq!(input.read_line().as_deref(), Some(&b"only"[..]));
        for _ in 0..3 {
            assert_eq!(input.read_line(), None);
        }
    }

    /// What `.STDIN~CHARS` and `~LINES` answer, per source and per state. The
    /// oracle's own answers, measured: a live descriptor nothing has drained
    /// is unmeasurable and answers `1`, and either kind answers `0` once a
    /// read has found the end. `Nothing` is a live-but-empty descriptor --
    /// the `/dev/null` column -- and not a measured zero until it is read.
    #[test]
    fn a_count_parts_a_live_source_from_a_drained_one() {
        let mut nothing = Input::new(ProgramInput::Nothing);
        assert_eq!(nothing.remaining(), None, "unread Nothing is unmeasurable");
        assert_eq!(nothing.remaining_lines(), None, "the first ask");
        assert_eq!(
            nothing.remaining_lines(),
            Some(0),
            "and `0` every ask after, with no read in between"
        );
        assert_eq!(
            nothing.remaining(),
            None,
            "while the byte count keeps answering unmeasurable"
        );
        assert_eq!(nothing.read_line(), None);
        assert_eq!(nothing.remaining(), Some(0), "a read drained it");
        assert_eq!(nothing.remaining_lines(), Some(0));

        let mut bytes = Input::new(ProgramInput::Bytes(b"one\ntwo\n".to_vec()));
        assert_eq!(bytes.remaining(), Some(8));
        assert_eq!(bytes.remaining_lines(), Some(2));
        assert_eq!(bytes.read_line().as_deref(), Some(&b"one"[..]));
        assert_eq!(bytes.remaining(), Some(4));
        assert_eq!(bytes.remaining_lines(), Some(1));
        assert_eq!(bytes.read_line().as_deref(), Some(&b"two"[..]));
        assert_eq!(bytes.remaining(), Some(0));
        assert_eq!(bytes.remaining_lines(), Some(0));
    }

    /// A final line with no terminator is still a line, which is why the count
    /// is not simply the newline count.
    #[test]
    fn an_unterminated_last_line_counts() {
        let mut input = Input::new(ProgramInput::Bytes(b"a\nb".to_vec()));
        assert_eq!(input.remaining_lines(), Some(2));
        let mut terminated = Input::new(ProgramInput::Bytes(b"a\nb\n".to_vec()));
        assert_eq!(terminated.remaining_lines(), Some(2));
    }

    /// `read_bytes` answers what is already there rather than waiting for the
    /// rest. **A `Cursor` cannot witness that on its own** -- it reports the
    /// end on the first call, so a request-filling loop would pass this too --
    /// which is why the source below hands back one byte at a time while
    /// staying open, the shape a live pipe has.
    #[test]
    fn a_byte_read_answers_what_is_there_without_waiting() {
        let mut input = Input::new(ProgramInput::Bytes(b"abc".to_vec()));
        assert_eq!(input.read_bytes(2), b"ab".to_vec());
        assert_eq!(input.read_bytes(5), b"c".to_vec(), "short at the end");
        assert_eq!(input.remaining(), Some(0));
        assert_eq!(input.read_bytes(1), Vec::<u8>::new());
    }

    /// A source that answers one byte per `read` and never reports the end,
    /// as a pipe with a writer still attached does.
    struct Dribble(u8);

    impl Read for Dribble {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let Some(first) = buffer.first_mut() else {
                return Ok(0);
            };
            *first = self.0;
            Ok(1)
        }
    }

    /// The negative control for the test above: asking for more than one byte
    /// from a source that never reports the end must answer the one byte that
    /// is there. A loop filling the request would never return, so this case
    /// is the one that fails rather than hangs if the fill comes back.
    #[test]
    fn a_byte_read_does_not_wait_for_a_source_that_stays_open() {
        let mut buffer = vec![0u8; 8];
        let count = Dribble(b'z').read(&mut buffer).expect("the source answers");
        assert_eq!(count, 1, "one read answers one byte, not eight");
    }
}
