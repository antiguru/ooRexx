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
//!
//! # The model is one position, not one read per instruction
//!
//! `PULL`, `PARSE PULL` and `PARSE LINEIN` do not each open the console;
//! they each advance the same `.input` position. Measured, with an empty
//! queue and four lines on stdin, running `parse pull` / `parse linein` /
//! `parse pull` / `parse linein` in that order:
//!
//! ```text
//! 1=<line-A> 2=<line-B> 3=<line-C> 4=<line-D>
//! ```
//!
//! The interleaving is the whole point: two different constructs consumed
//! four consecutive lines, so neither has a position of its own.
//!
//! The suite asserts this in exactly one place, and it is not in any `PARSE`
//! or `PULL` group -- `runtime.objects/environmentEntries.testGroup` sets
//! `.input~destination(.ArrayStream~of("a", "b", "c", "d", "e"))` and then
//! runs `pull`, `parse pull`, `parse linein`, `linein()` and
//! `.input~lineIn` against it, expecting `"A b c d e"`. Five constructs, one
//! position, and only the first uppercasing. Three of the five are
//! implemented here; `LINEIN()` is an excluded builtin and `.input~lineIn` is
//! a message send, so that assertion as written is not a target this crate
//! can pass. It is the reason the position lives in one place regardless.
//!
//! # `PARSE LINEIN` never consults the queue
//!
//! This is the finding that makes "each instruction reads the console" wrong
//! rather than merely imprecise. Measured, the same four clauses with two
//! entries pushed onto the queue first and the same four-line stdin:
//!
//! ```text
//! 1=<qentry-one> 2=<line-A> 3=<qentry-two> 4=<line-B>
//! ```
//!
//! Adjacent `PARSE PULL` and `PARSE LINEIN` clauses returned values from
//! different places, and the two `PARSE PULL`s did not advance the `.input`
//! position at all while the queue still had entries. So `PULL`'s queue is a
//! store consulted *before* this position, not a buffer in front of it.
//!
//! # End of input, and an unreadable input, are the null string
//!
//! Measured, empty queue and stdin at `/dev/null`: all four clauses answer
//! the null string, rc 0, no condition raised, no hang, and repeatably. Past
//! the last line of a non-empty stdin, the same. There is no reachable state
//! in which a line read fails.
//!
//! **An unreadable descriptor is also the null string, and that was measured
//! rather than assumed.** With stdin closed (`exec 0<&-`) and with stdin
//! bound to a *directory* -- so the read itself fails with `EISDIR` rather
//! than reporting end of file -- the oracle answers the null string, rc 0,
//! empty stderr, in both cases. So [`Input::read_line`] reporting `None` for
//! an I/O error is not this crate deciding to swallow one; it is the answer
//! the oracle gives.
//!
//! # The line rule, byte for byte
//!
//! A line is the bytes up to and including the next newline, with the
//! newline removed, and with **one** carriage return removed if it was
//! immediately before that newline. The oracle does the `\r\n` collapse in
//! `SysFile::gets` (`common/platform/unix/SysFile.cpp`), which rewrites a
//! `\r` into a `\n` when the next byte is a `\n` and otherwise leaves the
//! `\r` as data. Measured, `length` and `c2x` of each line read:
//!
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
//!
//! The last three rows are what separate this rule from the three plausible
//! near-misses: stripping every trailing `\r`, stripping a trailing `\r`
//! whether or not a newline followed, and treating the input as text rather
//! than bytes.

use std::io::{BufRead, Cursor};

use crate::Interp;
use crate::invocation::ProgramInput;

/// `.input`'s position: the one line cursor every input construct advances.
///
/// Not a `Vec<u8>` with an index, because [`ProgramInput::Stdin`] must be read
/// incrementally: the oracle reads the console a line at a time, so a program
/// that reads one line and exits must not have required the whole descriptor
/// to reach end of file first. `BufRead::read_until` is that read for all
/// three arms.
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
    ///
    /// `None` covers both end of input and a failed read, which the module
    /// doc's own measurement is what justifies: the oracle answers the null
    /// string for a closed descriptor and for one that cannot be read at all,
    /// exactly as it does past the last line.
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
    ///
    /// The queue is consulted first and consumed from the **front**, which is
    /// what makes `PUSH`'s head insertion the LIFO order and `QUEUE`'s tail
    /// append the FIFO one relative to it -- `queue.rs`'s own module doc
    /// carries the measured order and named this as the premise its own tests
    /// could not check.
    ///
    /// Reaching `.input` only when the queue is empty is not a fallback bolted
    /// on: measured, a `PARSE PULL` served from the queue leaves the `.input`
    /// position untouched, so an adjacent `PARSE LINEIN` still gets the first
    /// line of stdin (this module's doc has the transcript).
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
    ///
    /// Each case is one buffer and the full list of lines read out of it, so a
    /// reader that dropped or duplicated a line fails here as well as one that
    /// mishandled a terminator.
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
    ///
    /// Repeated rather than asked once, because "the null string, repeatably"
    /// is the measured property and a reader that answered a line once and
    /// then something else would satisfy a single call.
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
