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

use std::io::{BufRead, Cursor};

use crate::Interp;
use crate::invocation::ProgramInput;

/// `.input`'s position: the one line cursor every input construct advances.
pub(crate) struct Input {
    source: Source,
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
        }
    }

    /// The next line, or `None` once there is nothing left to read.
    fn read_line(&mut self) -> Option<Vec<u8>> {
        let mut line = Vec::new();
        let read = match &mut self.source {
            Source::Nothing => return None,
            Source::Stdin(stdin) => stdin.lock().read_until(b'\n', &mut line),
            Source::Bytes(cursor) => cursor.read_until(b'\n', &mut line),
        };
        match read {
            // Zero bytes with no error is end of input; a read error answers
            // the same way, per this module's doc.
            Ok(0) | Err(_) => return None,
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
}
