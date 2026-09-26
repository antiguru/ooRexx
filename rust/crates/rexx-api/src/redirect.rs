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

//! What a redirecting command handler reads and writes through its
//! `RexxIORedirectorContext`: the command's input, gathered before the call,
//! and the lines it writes, which reach their targets after it
//! (`interpreter/instructions/CommandIOContext.cpp`).

use std::cell::{Cell, OnceCell, RefCell};

/// One command's redirected streams, as a handler sees them.
///
/// Every address [`Redirector::read_line`] and [`Redirector::read_buffer`]
/// answer stays valid until this is dropped.
pub struct Redirector {
    /// Whether the command carried any `WITH` configuration: the oracle hands
    /// a handler no I/O context otherwise, and every member answers nothing.
    requested: bool,
    input: Option<Input>,
    output: Option<RefCell<Sink>>,
    error: Option<RefCell<Sink>>,
    /// Output and error resolved to one target, so an error write lands in
    /// the output's lines, in the order the handler wrote them.
    shared: bool,
}

/// The input's lines, each NUL-terminated, and how far the handler has read.
struct Input {
    lines: Vec<Box<[u8]>>,
    next: Cell<usize>,
    /// `readInputBuffered`'s buffer, made once and answered again after.
    buffer: OnceCell<Box<[u8]>>,
}

/// One output stream's lines, and the tail of a buffer that ended before its
/// line did (`OutputRedirector::bufferedData`).
#[derive(Default)]
struct Sink {
    lines: Vec<Vec<u8>>,
    pending: Option<Vec<u8>>,
}

impl Sink {
    /// `OutputRedirector::write`: the pending tail is a line of its own.
    fn write(&mut self, line: &[u8]) {
        self.flush();
        self.lines.push(line.to_vec());
    }

    /// `OutputRedirector::flushBuffer`, which drops one trailing `\r`.
    fn flush(&mut self) {
        if let Some(pending) = self.pending.take() {
            let line = pending.strip_suffix(b"\r").unwrap_or(&pending);
            self.lines.push(line.to_vec());
        }
    }

    /// `OutputRedirector::writeBuffer`: `\n` and `\r\n` end a line, a `\r\n`
    /// split across two buffers included, and the tail waits for the next
    /// buffer or the end.
    fn write_buffer(&mut self, mut data: &[u8]) {
        if data.is_empty() {
            return;
        }
        if let Some(mut pending) = self.pending.take() {
            if pending.ends_with(b"\r") && data[0] == b'\n' {
                pending.pop();
                self.lines.push(pending);
                data = &data[1..];
            } else {
                match scan_line(data) {
                    None => {
                        pending.extend_from_slice(data);
                        self.pending = Some(pending);
                        return;
                    }
                    Some((end, next)) => {
                        pending.extend_from_slice(&data[..end]);
                        self.lines.push(pending);
                        data = &data[next..];
                    }
                }
            }
        }
        while !data.is_empty() {
            match scan_line(data) {
                None => {
                    self.pending = Some(data.to_vec());
                    return;
                }
                Some((end, next)) => {
                    self.lines.push(data[..end].to_vec());
                    data = &data[next..];
                }
            }
        }
    }
}

/// `OutputRedirector::scanLine`: where the first line of `data` ends and the
/// next begins, or `None` where no line ends in it. A `\r` that is the last
/// byte decides nothing yet.
fn scan_line(data: &[u8]) -> Option<(usize, usize)> {
    for (at, byte) in data.iter().enumerate() {
        match byte {
            b'\r' if at + 1 == data.len() => return None,
            b'\r' if data[at + 1] == b'\n' => return Some((at, at + 2)),
            b'\n' => return Some((at, at + 1)),
            _ => {}
        }
    }
    None
}

impl Redirector {
    /// The context of a command issued with no `WITH` configuration.
    #[must_use]
    pub fn unrequested() -> Redirector {
        Redirector {
            requested: false,
            input: None,
            output: None,
            error: None,
            shared: false,
        }
    }

    /// The context of a command whose input, where redirected, is `input`,
    /// and which redirects output and error as `output` and `error` say;
    /// `shared` where both resolved to one target.
    #[must_use]
    pub fn new(input: Option<Vec<Vec<u8>>>, output: bool, error: bool, shared: bool) -> Redirector {
        let terminated = |mut line: Vec<u8>| {
            line.push(0);
            line.into_boxed_slice()
        };
        Redirector {
            requested: true,
            input: input.map(|lines| Input {
                lines: lines.into_iter().map(terminated).collect(),
                next: Cell::new(0),
                buffer: OnceCell::new(),
            }),
            output: output.then(RefCell::default),
            error: (error && !shared).then(RefCell::default),
            shared: output && error && shared,
        }
    }

    #[must_use]
    pub fn requested(&self) -> bool {
        self.requested
    }

    #[must_use]
    pub fn redirects_input(&self) -> bool {
        self.input.is_some()
    }

    #[must_use]
    pub fn redirects_output(&self) -> bool {
        self.output.is_some()
    }

    #[must_use]
    pub fn redirects_error(&self) -> bool {
        self.error.is_some() || self.shared
    }

    #[must_use]
    pub fn same_target(&self) -> bool {
        self.shared
    }

    /// The next input line's NUL-terminated bytes and its length, or `None`
    /// past the last line or with no input.
    pub(crate) fn read_line(&self) -> Option<(*const u8, usize)> {
        let input = self.input.as_ref()?;
        let line = input.lines.get(input.next.get())?;
        input.next.set(input.next.get() + 1);
        Some((line.as_ptr(), line.len() - 1))
    }

    /// Every input line not yet read, each ended by `\n`, as one
    /// NUL-terminated buffer, the same one on every call; `None` with no
    /// input.
    pub(crate) fn read_buffer(&self) -> Option<(*const u8, usize)> {
        let input = self.input.as_ref()?;
        let buffer = input.buffer.get_or_init(|| {
            let mut buffer = Vec::new();
            for line in &input.lines[input.next.get()..] {
                buffer.extend_from_slice(&line[..line.len() - 1]);
                buffer.push(b'\n');
            }
            input.next.set(input.lines.len());
            buffer.push(0);
            buffer.into_boxed_slice()
        });
        Some((buffer.as_ptr(), buffer.len() - 1))
    }

    /// The sink an output (`error` false) or error write goes to, `None`
    /// where that stream is not redirected.
    fn sink(&self, error: bool) -> Option<&RefCell<Sink>> {
        if error && !self.shared {
            self.error.as_ref()
        } else {
            self.output.as_ref()
        }
    }

    /// `WriteOutput` and `WriteError`: `line` is one line, whatever it holds.
    pub(crate) fn write(&self, error: bool, line: &[u8]) {
        if let Some(sink) = self.sink(error) {
            sink.borrow_mut().write(line);
        }
    }

    /// `WriteOutputBuffer` and `WriteErrorBuffer`.
    pub(crate) fn write_buffer(&self, error: bool, data: &[u8]) {
        if let Some(sink) = self.sink(error) {
            sink.borrow_mut().write_buffer(data);
        }
    }

    /// The output lines and the error lines, each stream's pending tail a
    /// line of its own (`CommandIOContext::cleanup`). Under
    /// [`Redirector::same_target`] every line is output.
    #[must_use]
    pub fn finish(self) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
        let lines = |sink: Option<RefCell<Sink>>| {
            sink.map_or_else(Vec::new, |sink| {
                let mut sink = sink.into_inner();
                sink.flush();
                sink.lines
            })
        };
        (lines(self.output), lines(self.error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(buffers: &[&[u8]]) -> Vec<Vec<u8>> {
        let redirector = Redirector::new(None, true, false, false);
        for buffer in buffers {
            redirector.write_buffer(false, buffer);
        }
        redirector.finish().0
    }

    /// The oracle's own lines for the buffer shapes the `FUNCTION` group
    /// writes, a `\r\n` split across two buffers and a trailing `\r` among
    /// them.
    #[test]
    fn buffers_split_into_the_oracles_lines() {
        assert_eq!(written(&[b"Line1\r\n"]), [b"Line1".to_vec()]);
        assert_eq!(written(&[b"Line1", b"Line2"]), [b"Line1Line2".to_vec()]);
        assert_eq!(
            written(&[b"Line1\r", b"\nLine2"]),
            [b"Line1".to_vec(), b"Line2".to_vec()]
        );
        assert_eq!(written(&[b"Line3\rLine4"]), [b"Line3\rLine4".to_vec()]);
        assert_eq!(written(&[b"Line1\r", b"Line2"]), [b"Line1\rLine2".to_vec()]);
        assert_eq!(
            written(&[b"\r\nLine1\r\n\r\nLine2\r", b"\nLine3"]),
            [
                b"".to_vec(),
                b"Line1".to_vec(),
                b"".to_vec(),
                b"Line2".to_vec(),
                b"Line3".to_vec()
            ]
        );
        assert_eq!(
            written(&[b"Line1\r", b"\nLine2", b"x\r"]),
            [b"Line1".to_vec(), b"Line2x".to_vec()]
        );
        assert_eq!(written(&[b""]), Vec::<Vec<u8>>::new());
    }

    /// A single-line write ends the pending tail first; an error write under
    /// one target lands among the output lines.
    #[test]
    fn a_line_write_flushes_the_tail_and_one_target_interleaves() {
        let redirector = Redirector::new(None, true, true, true);
        redirector.write_buffer(true, b"a\r");
        redirector.write(true, b"b");
        redirector.write_buffer(false, b"c\nd");
        redirector.write(false, b"z\nw");
        let (output, error) = redirector.finish();
        assert_eq!(
            output,
            [
                b"a".to_vec(),
                b"b".to_vec(),
                b"c".to_vec(),
                b"d".to_vec(),
                b"z\nw".to_vec()
            ]
        );
        assert!(error.is_empty());
    }

    /// Lines then the buffer: the buffer holds only what was not read, and
    /// is the same allocation when asked again.
    #[test]
    fn the_input_buffer_holds_the_unread_lines_once() {
        let redirector = Redirector::new(
            Some(vec![b"l1".to_vec(), b"l2".to_vec()]),
            false,
            false,
            false,
        );
        let (first, length) = redirector.read_line().expect("a first line");
        assert_eq!(length, 2);
        let (buffer, length) = redirector.read_buffer().expect("input is redirected");
        assert_eq!(length, 3);
        assert_ne!(first, buffer);
        assert!(redirector.read_line().is_none());
        assert_eq!(redirector.read_buffer(), Some((buffer, 3)));
        assert!(Redirector::unrequested().read_buffer().is_none());
    }
}
