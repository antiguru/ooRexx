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

use rexx_core::ObjRef;

use crate::invocation::ProgramInput;
use crate::lines::{as_position, lines_after, offset_of, read_from, size_of};
use crate::{Failure, Interp};

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
    /// The process's standard input, read a chunk at a time off the baton,
    /// where a halt can abandon the wait ([`Interp::fill_stdin`]).
    Stdin(Stdin),
    /// The process's standard input when it is a regular file.
    File(FileConsole),
    Bytes(Cursor<Vec<u8>>),
}

/// What standard input has delivered and not been read yet.
#[derive(Default)]
struct Stdin {
    buffered: Vec<u8>,
    /// A chunk read found the end of the input, or failed.
    ended: bool,
    /// A chunk read is in flight, perhaps one a halt abandoned.
    reading: bool,
}

/// How much one read of standard input asks for: `std`'s own buffer size,
/// which a read of at least that size bypasses.
const STDIN_CHUNK: usize = 8 * 1024;

/// Reads one chunk of standard input: empty at its end or on an error.
pub(crate) fn read_stdin_chunk() -> Vec<u8> {
    let mut chunk = vec![0u8; STDIN_CHUNK];
    let read = loop {
        match std::io::stdin().lock().read(&mut chunk) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            read => break read.unwrap_or(0),
        }
    };
    chunk.truncate(read);
    chunk
}

/// A standard input that is a regular file: a persistent stream with one read
/// position, which CHARIN's start and LINEIN's line move (rexxref
/// funct.xml:1314-1322 and :3080-3094), and whose counts are taken from that
/// position (iostrms.xml:155-165), with [`crate::lines`]'s walkers and line
/// cache on an [`rexx_core::OpenFile`]. The descriptor's offset is the read
/// position whenever another process may read from it: it is put there before
/// a command starts and when the console is dropped, and taken from there
/// when a command has returned.
struct FileConsole {
    open: rexx_core::OpenFile,
    /// Bytes read ahead, from character position `buffered_at`.
    buffered: Vec<u8>,
    buffered_at: u64,
    /// The size the line count was taken at; a changed size drops the count.
    counted_size: Option<u64>,
    /// The descriptor's offset where this console's own reads, walks and
    /// syncs last left it; any other offset was moved by a command.
    left_at: std::cell::Cell<Option<u64>>,
}

impl FileConsole {
    /// Standard input, when it is a regular file.
    fn open() -> Option<FileConsole> {
        use std::os::fd::AsFd;
        let stdin = std::io::stdin();
        FileConsole::over(std::fs::File::from(
            stdin.as_fd().try_clone_to_owned().ok()?,
        ))
    }

    /// `file` as a console, positioned at its descriptor's offset; `None`
    /// unless it is a regular file.
    fn over(mut file: std::fs::File) -> Option<FileConsole> {
        use std::io::Seek;
        if !file.metadata().ok()?.file_type().is_file() {
            return None;
        }
        let offset = file.stream_position().unwrap_or(0);
        let mut open = rexx_core::OpenFile::new(file, 1, false, false);
        open.read_position = as_position(offset + 1);
        Some(FileConsole {
            open,
            buffered: Vec::new(),
            buffered_at: 0,
            counted_size: None,
            left_at: std::cell::Cell::new(Some(offset)),
        })
    }

    /// Records where the console's own use of the descriptor left its offset.
    fn note_offset(&self) {
        use std::io::Seek;
        self.left_at.set(self.open.file().stream_position().ok());
    }

    /// The read position, as `read_from` takes it.
    fn position(&self) -> u64 {
        offset_of(self.open.read_position)
    }

    /// Makes `buffered` hold the character at the read position, answering
    /// whether the file has one.
    fn fill(&mut self) -> bool {
        let at = self.position();
        let end = self.buffered_at + self.buffered.len() as u64;
        if at >= self.buffered_at && at < end {
            return true;
        }
        self.buffered = read_from(self.open.flushed(), at, STDIN_CHUNK).unwrap_or_default();
        self.buffered_at = at;
        self.note_offset();
        !self.buffered.is_empty()
    }

    /// The buffered bytes from the read position on, after [`FileConsole::fill`].
    fn ahead(&self) -> &[u8] {
        let from = usize::try_from(self.position().saturating_sub(self.buffered_at)).unwrap_or(0);
        self.buffered.get(from..).unwrap_or_default()
    }

    /// One line with its terminator, or nothing at the end of the file.
    fn read_line(&mut self) -> Vec<u8> {
        let mut line = Vec::new();
        while self.fill() {
            let ahead = self.ahead();
            let found = ahead.iter().position(|byte| *byte == b'\n');
            let take = found.map_or(ahead.len(), |at| at + 1);
            line.extend_from_slice(&ahead[..take]);
            self.advance(take, i64::from(found.is_some()));
            if found.is_some() {
                break;
            }
        }
        line
    }

    /// Moves the read position over `taken` characters holding `line_ends`
    /// line ends. A line mark at the old position moves with it, so a count
    /// after a loop of reads walks nothing; its debug digest is dropped, as
    /// a mark moved without a walk cannot vouch for one.
    fn advance(&mut self, taken: usize, line_ends: i64) {
        let from = self.open.read_position;
        self.open.read_position += taken as i64;
        if let Some((mark, line)) = self.open.line_mark
            && mark == from
        {
            self.open.line_mark = Some((self.open.read_position, line + line_ends));
            self.open.line_mark_digest = None;
        }
    }

    /// Up to `wanted` bytes from the read position, short only at the end of
    /// the file.
    fn read_bytes(&mut self, wanted: usize) -> Vec<u8> {
        let mut out = Vec::new();
        while out.len() < wanted && self.fill() {
            let ahead = self.ahead();
            let take = ahead.len().min(wanted - out.len());
            out.extend_from_slice(&ahead[..take]);
            let line_ends = ahead[..take].iter().filter(|byte| **byte == b'\n').count();
            self.advance(take, line_ends as i64);
        }
        out
    }

    /// CHARS: the characters from the read position to the end of the file.
    fn chars_left(&self) -> u64 {
        let left = size_of(self.open.file()) as i64 - (self.open.read_position - 1);
        u64::try_from(left).unwrap_or(0)
    }

    /// LINES Normal: whether any character remains.
    fn has_data(&mut self) -> bool {
        self.fill()
    }

    /// LINES Count: the lines from the read position on, a last line with no
    /// line end included, from the line cache. Nothing here writes the file,
    /// so the count is dropped when the file's size changes instead.
    fn lines_left(&mut self) -> u64 {
        let size = size_of(self.open.file());
        if self.open.read_position > size as i64 {
            return 0;
        }
        match self.counted_size {
            Some(counted_at) if counted_at == size => {}
            _ => {
                self.open.line_total = None;
                self.open.line_total_digest = None;
                self.counted_size = Some(size);
            }
        }
        let position = self.open.read_position;
        let left = lines_after(&mut self.open, position).unwrap_or(0);
        self.note_offset();
        left
    }

    /// CHARIN's start: moves the read position to character `position`,
    /// counted from 1, answering `false` past the end of the file, where
    /// nothing moves (`StreamInfo::setCharReadPosition`).
    fn seek_char(&mut self, position: u64) -> bool {
        if position == 0 || position > size_of(self.open.file()) {
            return false;
        }
        self.open.read_position = as_position(position);
        true
    }

    /// LINEIN's line: moves the read position to the start of line `line`,
    /// or to the end of the file when it has fewer lines.
    fn seek_line(&mut self, line: u64) {
        let (_, at) = crate::lines::seek_line(&mut self.open, as_position(line), 0);
        self.open.read_position = at;
        self.note_offset();
    }

    /// Puts the descriptor's offset at the read position where the console's
    /// own reads moved it, for whatever reads the descriptor next. Where they
    /// did not, the offset is where the last command left it, which is the
    /// read position already.
    fn sync_offset(&self) {
        use std::io::{Seek, SeekFrom};
        let want = self.position().saturating_sub(1);
        if self
            .open
            .file()
            .stream_position()
            .is_ok_and(|at| at != want)
        {
            let _ = self.open.file().seek(SeekFrom::Start(want));
        }
        self.note_offset();
    }

    /// Takes the read position from the descriptor's offset, where a command
    /// that read the descriptor left it. The line mark and the read-ahead go.
    /// An offset where the console itself left it means no command read, and
    /// nothing moves.
    fn adopt_offset(&mut self) {
        use std::io::Seek;
        let Ok(offset) = self.open.file().stream_position() else {
            return;
        };
        if self.left_at.get() == Some(offset) {
            return;
        }
        self.left_at.set(Some(offset));
        self.open.read_position = as_position(offset + 1);
        self.open.line_mark = None;
        self.open.line_mark_digest = None;
        self.buffered.clear();
    }
}

impl Drop for FileConsole {
    fn drop(&mut self) {
        self.sync_offset();
    }
}

impl Input {
    /// Whether this is the process's own standard input, whose reads hand
    /// the buffered output over first.
    fn is_live(&self) -> bool {
        matches!(self.source, Source::Stdin(_) | Source::File(_))
    }

    /// Whether standard input needs a chunk more before the next read can
    /// answer: a whole line where `line`, else any byte.
    fn stdin_wants(&self, line: bool) -> bool {
        let Source::Stdin(stdin) = &self.source else {
            return false;
        };
        let ready = match line {
            true => stdin.buffered.contains(&b'\n'),
            false => !stdin.buffered.is_empty(),
        };
        !ready && !stdin.ended
    }

    /// Starts a chunk read where none is in flight, answering whether one
    /// was started.
    fn start_stdin_read(&mut self) -> bool {
        match &mut self.source {
            Source::Stdin(stdin) if !stdin.reading => {
                stdin.reading = true;
                true
            }
            _ => false,
        }
    }

    /// A halt abandoned a read: the counts answer `0` until the next read,
    /// as the oracle's do after its interrupted read.
    fn interrupted(&mut self) {
        self.exhausted = true;
    }

    /// A read of standard input follows, which decides the counts afresh.
    fn reading_stdin(&mut self) {
        if matches!(self.source, Source::Stdin(_)) {
            self.exhausted = false;
        }
    }

    /// Files a chunk standard input delivered: its end where empty.
    pub(crate) fn receive(&mut self, chunk: Vec<u8>) {
        if let Source::Stdin(stdin) = &mut self.source {
            stdin.reading = false;
            if chunk.is_empty() {
                stdin.ended = true;
            }
            stdin.buffered.extend(chunk);
        }
    }

    pub(crate) fn new(input: ProgramInput) -> Input {
        Input {
            source: match input {
                ProgramInput::Nothing => Source::Nothing,
                ProgramInput::Stdin => match FileConsole::open() {
                    Some(file) => Source::File(file),
                    None => Source::Stdin(Stdin::default()),
                },
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
            Source::Stdin(stdin) => {
                let end = match stdin.buffered.iter().position(|byte| *byte == b'\n') {
                    Some(at) => at + 1,
                    None => stdin.buffered.len(),
                };
                line.extend(stdin.buffered.drain(..end));
                Ok(end)
            }
            Source::File(file) => {
                line = file.read_line();
                Ok(line.len())
            }
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
            Source::Stdin(stdin) => {
                let count = wanted.min(stdin.buffered.len());
                buffer[..count].copy_from_slice(&stdin.buffered[..count]);
                stdin.buffered.drain(..count);
                Ok(count)
            }
            Source::File(file) => return file.read_bytes(wanted),
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
    /// `None` when it cannot, matching [`Input::remaining`]. Without `count`
    /// a file console answers 1 or 0 for whether anything is left, with no
    /// scan (rexxref funct.xml:3296-3306).
    fn remaining_lines(&mut self, count: bool) -> Option<u64> {
        if let Source::File(file) = &mut self.source {
            return Some(match count {
                true => file.lines_left(),
                false => u64::from(file.has_data()),
            });
        }
        if self.exhausted {
            return Some(0);
        }
        if let Source::Stdin(_) = &self.source {
            return self.stdin_waiting().map(u64::from);
        }
        let asked = std::mem::replace(&mut self.lines_asked, true);
        match &self.source {
            // No input answers `1` once and `0` after, which is where this
            // parts from [`Input::remaining`].
            Source::Nothing if asked => Some(0),
            Source::Nothing | Source::Stdin(_) | Source::File(_) => None,
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

    /// Whether data waits on a live standard input, found without blocking:
    /// what is buffered, else a `poll` with no wait, and when it reports the
    /// descriptor readable, a read that cannot block takes what is there into
    /// the buffer (rexxref iostrms.xml:189-193). `None` while a read is
    /// already in flight, when the state cannot be learned.
    fn stdin_waiting(&mut self) -> Option<bool> {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};
        let Source::Stdin(stdin) = &self.source else {
            return None;
        };
        if !stdin.buffered.is_empty() {
            return Some(true);
        }
        if stdin.ended {
            return Some(false);
        }
        if stdin.reading {
            return None;
        }
        let zero = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let descriptor = std::io::stdin();
        let mut polled = [PollFd::new(&descriptor, PollFlags::IN)];
        match poll(&mut polled, Some(&zero)) {
            Ok(_) if polled[0].revents().is_empty() => return Some(false),
            Ok(_) => {}
            Err(_) => return None,
        }
        let chunk = read_stdin_chunk();
        self.receive(chunk);
        let Source::Stdin(stdin) = &self.source else {
            return None;
        };
        Some(!stdin.buffered.is_empty())
    }

    /// The bytes left to read, when that is knowable without consuming them,
    /// and `None` when it is not -- a live standard input is a stream, not a
    /// length. Measured on the oracle: `.stdin~chars` answers the true
    /// remaining count for a redirected regular file, `1` for a pipe nothing
    /// has drained, and `0` for either once a read has found the end, which is
    /// what `exhausted` carries.
    fn remaining(&mut self) -> Option<u64> {
        if let Source::File(file) = &self.source {
            return Some(file.chars_left());
        }
        if self.exhausted {
            return Some(0);
        }
        if let Source::Stdin(_) = &self.source {
            return self.stdin_waiting().map(u64::from);
        }
        match &self.source {
            // Live but empty, exactly like a `/dev/null` descriptor: measured,
            // the oracle answers `1` there until a read drains it and `0`
            // after -- so an unread `Nothing` is not yet a measured zero.
            Source::Nothing => None,
            Source::Stdin(_) | Source::File(_) => None,
            Source::Bytes(cursor) => {
                Some((cursor.get_ref().len() as u64).saturating_sub(cursor.position()))
            }
        }
    }
}

impl Interp {
    /// One line for `PULL` and `PARSE PULL`: the queue's head if the queue has
    /// one, and otherwise [`Interp::linein_line`].
    ///
    /// **The queue is asked first and the route is not asked at all when it
    /// answers** -- measured, a program with one line pushed reads it and the
    /// destination sees a single `LINEIN`, for the second read.
    pub(crate) fn pull_line(&mut self) -> Result<Vec<u8>, Failure> {
        match self.queue.pop() {
            Some(line) => Ok(line),
            None => self.linein_line(),
        }
    }

    /// One line for `PARSE LINEIN`: always the route, never the queue.
    ///
    /// **A message, not a descriptor read.** Measured, the oracle sends
    /// `LINEIN` with no arguments to whatever `.local` holds under `INPUT`, so
    /// a program that redirects the destination answers the instruction. End
    /// of input raises `NOTREADY` for the same reason: the untouched route
    /// ends at `.STDIN`, whose own `LINEIN` raises it -- measured, `PARSE
    /// LINEIN` and `PARSE PULL` on empty input are both rc 9 under a trap
    /// where this crate used to answer an empty line and carry on.
    ///
    /// A missing or `.nil` entry reads the descriptor instead, which is this
    /// crate's licensed answer where the oracle dies
    /// (`corpus/oracle-crashes.txt` entry 11b).
    pub(crate) fn linein_line(&mut self) -> Result<Vec<u8>, Failure> {
        let route = self.local_route(b"INPUT")?;
        let Some(route) = route.filter(|route| *route != ObjRef::NIL) else {
            return Ok(self.input_line().unwrap_or_default());
        };
        let caller = self.caller();
        let answer = pinned!(
            self,
            crate::pinning::PinKind::PullWrapper,
            self.send_message(route, crate::dispatch::LINEIN, None, &[], caller)
        )?;
        Ok(match answer {
            Some(value) => self.to_text(value).into_owned(),
            None => Vec::new(),
        })
    }

    /// The reader behind `.STDIN~LINEIN`, keeping end of input apart from an
    /// empty line: its caller raises `NOTREADY` on the first and not on the
    /// second.
    pub(crate) fn input_line(&mut self) -> Option<Vec<u8>> {
        self.hand_over_before_read();
        if !self.fill_stdin(true) {
            return None;
        }
        self.input.read_line()
    }

    /// Whether `.input` is the process's own standard input.
    pub(crate) fn input_is_process_stdin(&self) -> bool {
        self.input.is_live()
    }

    /// Up to `wanted` bytes of `.input`, for `.STDIN~CHARIN`.
    pub(crate) fn input_bytes(&mut self, wanted: usize) -> Vec<u8> {
        self.hand_over_before_read();
        if !self.fill_stdin(false) {
            return Vec::new();
        }
        self.input.read_bytes(wanted)
    }

    /// Reads standard input until it holds a line where `line`, else a byte,
    /// or has ended: each chunk off the baton, with this thread idle until
    /// it is posted or a signal's halt arrives (ruling P60). A halt abandons
    /// the wait, whose chunk is filed when it comes, and answers `false`:
    /// the read answers nothing, as the oracle's interrupted read does.
    /// Other posts wait for the next drain, held here so that an idle waits
    /// for a new post.
    fn fill_stdin(&mut self, line: bool) -> bool {
        let mut kept = std::collections::VecDeque::new();
        let filled = self.fill_stdin_keeping(line, &mut kept);
        self.timer.requeue(kept);
        if filled {
            self.input.reading_stdin();
        }
        filled
    }

    fn fill_stdin_keeping(
        &mut self,
        line: bool,
        kept: &mut std::collections::VecDeque<crate::scheduler::Posted>,
    ) -> bool {
        while self.input.stdin_wants(line) {
            if self.input.start_stdin_read() {
                let Some(worker) = self.pool.reserve() else {
                    self.input.receive(read_stdin_chunk());
                    continue;
                };
                let inbox = self.timer.inbox();
                let baton = crate::sync::Arc::clone(&self.baton);
                worker.run(Box::new(move || {
                    crate::scheduler::posting_panics(&inbox, &baton, || {
                        let chunk = crate::signal::unblocked(read_stdin_chunk);
                        inbox.post(crate::scheduler::Posted::Input(chunk));
                    });
                }));
            }
            let mut halted = false;
            for post in self.timer.idle() {
                match post {
                    crate::scheduler::Posted::Input(chunk) => self.input.receive(chunk),
                    crate::scheduler::Posted::Halt => halted = true,
                    other => kept.push_back(other),
                }
            }
            if halted {
                self.input.interrupted();
                self.halt_all();
                return false;
            }
        }
        true
    }

    /// Hands the two output buffers to the embedding's [`crate::Sinks`],
    /// which is what puts a prompt in front of the reader before the read
    /// that waits for their answer.
    ///
    /// Does nothing for an embedding that installed none, and nothing for a
    /// source that cannot wait -- so no differential run's output moves.
    fn hand_over_before_read(&mut self) {
        if self.input.is_live() {
            self.hand_over_output();
        }
    }

    /// Hands the two output buffers to the embedding's [`crate::Sinks`], if
    /// it installed any: before a read, before this thread idles and before
    /// a command starts.
    pub(crate) fn hand_over_output(&mut self) {
        self.hand_over_stdout();
        self.hand_over_stderr();
    }

    /// [`Interp::hand_over_output`] for standard output alone. A refusal is
    /// kept in [`Interp::sink_errors`].
    pub(crate) fn hand_over_stdout(&mut self) {
        if let Some(sinks) = &mut self.sinks
            && !self.out.is_empty()
        {
            if let Err(error) = (sinks.stdout)(&self.out) {
                self.sink_errors[0] = Some(error.raw_os_error().unwrap_or(0));
            }
            self.out.clear();
        }
    }

    /// [`Interp::hand_over_output`] for standard error alone.
    pub(crate) fn hand_over_stderr(&mut self) {
        if let Some(sinks) = &mut self.sinks
            && !self.trace.is_empty()
        {
            if let Err(error) = (sinks.stderr)(&self.trace) {
                self.sink_errors[1] = Some(error.raw_os_error().unwrap_or(0));
            }
            self.trace.clear();
        }
    }

    /// What `.STDIN~CHARS` answers: the bytes left, 1 or 0 for whether data
    /// waits on a live standard input, or `None` when that cannot be learned.
    pub(crate) fn input_remaining(&mut self) -> Option<u64> {
        self.input.remaining()
    }

    /// What `.STDIN~LINES` answers, as [`Interp::input_remaining`] for lines:
    /// their count where `count`, else 1 or 0 for whether any is left.
    pub(crate) fn input_remaining_lines(&mut self, count: bool) -> Option<u64> {
        self.input.remaining_lines(count)
    }

    /// Puts standard input's descriptor offset at `.STDIN`'s read position,
    /// for a child that inherits the descriptor.
    pub(crate) fn sync_input_offset(&self) {
        if let Source::File(file) = &self.input.source {
            file.sync_offset();
        }
    }

    /// Takes `.STDIN`'s read position from the descriptor's offset, after a
    /// child that inherited the descriptor has returned.
    pub(crate) fn adopt_input_offset(&mut self) {
        if let Source::File(file) = &mut self.input.source {
            file.adopt_offset();
        }
    }

    /// The file `.STDIN` reads, when standard input is a regular file.
    pub(crate) fn input_file(&mut self) -> Option<&mut rexx_core::OpenFile> {
        match &mut self.input.source {
            Source::File(file) => Some(&mut file.open),
            _ => None,
        }
    }

    /// Moves `.STDIN`'s read position to character `position` for a CHARIN
    /// start: `Some(false)` past the end, where nothing moves, and `None`
    /// when standard input is not a regular file.
    pub(crate) fn input_seek_char(&mut self, position: u64) -> Option<bool> {
        match &mut self.input.source {
            Source::File(file) => Some(file.seek_char(position)),
            _ => None,
        }
    }

    /// Moves `.STDIN`'s read position to the start of `line` for a LINEIN
    /// line, or to the end of the file when it has fewer lines: `false` when
    /// standard input is not a regular file.
    pub(crate) fn input_seek_line(&mut self, line: u64) -> bool {
        match &mut self.input.source {
            Source::File(file) => {
                file.seek_line(line);
                true
            }
            _ => false,
        }
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
        assert_eq!(nothing.remaining_lines(true), None, "the first ask");
        assert_eq!(
            nothing.remaining_lines(true),
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
        assert_eq!(nothing.remaining_lines(true), Some(0));

        let mut bytes = Input::new(ProgramInput::Bytes(b"one\ntwo\n".to_vec()));
        assert_eq!(bytes.remaining(), Some(8));
        assert_eq!(bytes.remaining_lines(true), Some(2));
        assert_eq!(bytes.read_line().as_deref(), Some(&b"one"[..]));
        assert_eq!(bytes.remaining(), Some(4));
        assert_eq!(bytes.remaining_lines(true), Some(1));
        assert_eq!(bytes.read_line().as_deref(), Some(&b"two"[..]));
        assert_eq!(bytes.remaining(), Some(0));
        assert_eq!(bytes.remaining_lines(true), Some(0));
    }

    /// A final line with no terminator is still a line, which is why the count
    /// is not simply the newline count.
    #[test]
    fn an_unterminated_last_line_counts() {
        let mut input = Input::new(ProgramInput::Bytes(b"a\nb".to_vec()));
        assert_eq!(input.remaining_lines(true), Some(2));
        let mut terminated = Input::new(ProgramInput::Bytes(b"a\nb\n".to_vec()));
        assert_eq!(terminated.remaining_lines(true), Some(2));
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

    /// A file console over a fresh file holding `bytes`, and the file's path.
    fn console(name: &str, bytes: &[u8]) -> (FileConsole, std::path::PathBuf) {
        let path =
            std::env::temp_dir().join(format!("rexx-file-console-{}-{name}", std::process::id()));
        std::fs::write(&path, bytes).expect("the console file is written");
        let file = std::fs::File::open(&path).expect("the console file opens");
        let console = FileConsole::over(file).expect("a regular file is a file console");
        (console, path)
    }

    /// CHARS, LINES Count and LINES Normal.
    fn counts(console: &mut FileConsole) -> (u64, u64, bool) {
        (
            console.chars_left(),
            console.lines_left(),
            console.has_data(),
        )
    }

    /// The oracle's answers on the same sequence, measured on a file stdin,
    /// except the two LINES Counts after a line seek: the oracle answers 1
    /// and -1 there from a stale line cache, and 3 and 1 lines remain.
    #[test]
    fn a_file_console_has_one_read_position() {
        let (mut c, path) = console("position", b"hello\nworld\nthird\nlast");
        assert_eq!(counts(&mut c), (22, 4, true));
        assert_eq!(c.read_line(), b"hello\n");
        assert_eq!(counts(&mut c), (16, 3, true));
        assert!(c.seek_char(3));
        assert_eq!(c.read_bytes(1), b"l");
        assert_eq!(counts(&mut c), (19, 4, true));
        assert!(c.seek_char(1));
        assert_eq!(c.read_bytes(3), b"hel");
        assert_eq!(counts(&mut c), (19, 4, true));
        assert_eq!(c.read_line(), b"lo\n", "the rest of the line CHARIN was in");
        assert_eq!(counts(&mut c), (16, 3, true));
        assert_eq!(c.read_bytes(3), b"wor");
        assert_eq!(c.read_line(), b"ld\n");
        assert_eq!(counts(&mut c), (10, 2, true));
        c.seek_line(1);
        assert_eq!(c.read_line(), b"hello\n");
        assert_eq!(counts(&mut c), (16, 3, true));
        c.seek_line(3);
        assert_eq!(c.read_line(), b"third\n");
        assert_eq!(counts(&mut c), (4, 1, true));
        assert!(c.seek_char(2));
        assert_eq!(c.read_bytes(2), b"el");
        assert_eq!(counts(&mut c), (19, 4, true));
        c.seek_line(4);
        assert_eq!(c.read_line(), b"last", "a last line with no line end");
        assert_eq!(counts(&mut c), (0, 0, false));
        assert!(c.seek_char(22));
        assert_eq!(c.read_bytes(1), b"t");
        assert_eq!(c.read_line(), b"", "the end of the file");
        assert_eq!(counts(&mut c), (0, 0, false));
        let _ = std::fs::remove_file(&path);
    }

    /// A start past the end moves nothing; a line past the last one is the end
    /// of the file (measured on the oracle: `charin(,23)` answers the null
    /// string with CHARS still 22, `linein(,9)` leaves CHARS 0).
    #[test]
    fn a_seek_past_the_end_of_a_file_console() {
        let (mut c, path) = console("past", b"hello\nworld\nthird\nlast");
        assert!(!c.seek_char(23));
        assert!(!c.seek_char(100));
        assert_eq!(counts(&mut c), (22, 4, true), "nothing moved");
        assert!(c.seek_char(22));
        assert_eq!(c.read_bytes(5), b"t", "short at the end");
        c.seek_line(9);
        assert_eq!(counts(&mut c), (0, 0, false));
        assert_eq!(c.read_line(), b"");
        c.seek_line(4);
        assert_eq!(c.read_line(), b"last");
        c.seek_line(5);
        assert_eq!(
            counts(&mut c),
            (0, 0, false),
            "the line after the last line end"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// A line seek to the line the position lies in goes to its start, and a
    /// seek to each line in turn reads each line.
    #[test]
    fn a_line_seek_lands_on_the_start_of_the_line() {
        let (mut c, path) = console("midline", b"hello\nworld\nthird\n");
        assert_eq!(c.read_bytes(2), b"he");
        c.seek_line(1);
        assert_eq!(c.read_line(), b"hello\n");
        assert_eq!(c.read_bytes(2), b"wo");
        c.seek_line(2);
        assert_eq!(c.read_line(), b"world\n");
        for (line, expected) in [
            (1, "hello\n"),
            (2, "world\n"),
            (3, "third\n"),
            (2, "world\n"),
        ] {
            c.seek_line(line);
            assert_eq!(c.read_line(), expected.as_bytes(), "line {line}");
        }
        let _ = std::fs::remove_file(&path);
    }

    /// The lines left from `position` in `bytes`, counted afresh.
    fn lines_in(bytes: &[u8], position: u64) -> u64 {
        let left = bytes
            .get(usize::try_from(position).unwrap_or(usize::MAX)..)
            .unwrap_or_default();
        let ends = left.iter().filter(|byte| **byte == b'\n').count() as u64;
        ends + u64::from(left.last().is_some_and(|byte| *byte != b'\n'))
    }

    /// The cached line count answers what a fresh count answers after every
    /// read and after a seek either way, and the file is counted once while
    /// its size stays.
    #[test]
    fn a_file_console_counts_lines_once() {
        let mut bytes = (1..=300).map(|n| format!("line {n}\n")).collect::<String>();
        bytes.push_str("tail");
        let (mut c, path) = console("count", bytes.as_bytes());
        let mut step = 0;
        while c.position() <= bytes.len() as u64 {
            let offset = c.position() - 1;
            assert_eq!(
                c.lines_left(),
                lines_in(bytes.as_bytes(), offset),
                "at {offset}"
            );
            assert_eq!(c.open.line_total, Some(301), "counted once");
            assert_eq!(c.counted_size, Some(bytes.len() as u64));
            if step % 3 == 1 {
                c.read_bytes(5);
            } else {
                c.read_line();
            }
            step += 1;
        }
        assert_eq!(c.lines_left(), 0);
        assert!(c.seek_char(10));
        assert_eq!(
            c.lines_left(),
            lines_in(bytes.as_bytes(), 9),
            "a seek backwards"
        );
        assert!(c.seek_char(100));
        assert_eq!(
            c.lines_left(),
            lines_in(bytes.as_bytes(), 99),
            "a seek forwards"
        );
        assert_eq!(c.open.line_total, Some(301));
        let _ = std::fs::remove_file(&path);
    }

    /// Appends `bytes` to the console's file, as another writer would.
    fn append(path: &std::path::Path, bytes: &[u8]) {
        use std::io::Write;
        std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(bytes))
            .expect("the console file grows");
    }

    /// A count taken before another writer appended is not reused after: the
    /// file is counted again at its new size.
    #[test]
    fn a_file_console_recounts_a_grown_file() {
        let (mut c, path) = console("recount", b"a\nb\nc\n");
        assert_eq!(c.lines_left(), 3);
        assert_eq!(c.read_line(), b"a\n");
        assert_eq!(c.lines_left(), 2);
        append(&path, b"x\ny\n");
        assert_eq!(c.lines_left(), 4, "b, c and the two appended lines");
        assert_eq!(c.counted_size, Some(10));
        let _ = std::fs::remove_file(&path);
    }

    /// The descriptor's offset is the read position once the console syncs
    /// it, and when the console is dropped, so a process reading the
    /// descriptor next starts where the program stopped.
    #[test]
    fn a_file_console_leaves_the_descriptor_at_the_read_position() {
        use std::io::Seek;
        let (mut c, path) = console("sync", b"hello\nworld\nthird\n");
        assert_eq!(c.read_line(), b"hello\n");
        let shared = c
            .open
            .file()
            .try_clone()
            .expect("the descriptor duplicates");
        assert_ne!(
            (&shared).stream_position().ok(),
            Some(6),
            "the read moved the offset elsewhere"
        );
        c.sync_offset();
        assert_eq!((&shared).stream_position().ok(), Some(6));
        assert_eq!(c.read_bytes(3), b"wor");
        drop(c);
        assert_eq!(
            (&shared).stream_position().ok(),
            Some(9),
            "dropped at the read position"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// A command that read the descriptor moved it, and the console reads on
    /// from there; a sync with nothing read since leaves what the command
    /// consumed consumed.
    #[test]
    fn a_command_that_consumed_moves_the_read_position() {
        use std::io::{Seek, SeekFrom};
        let (mut c, path) = console("adopt", b"one\ntwo\nthree\nfour\n");
        let mut shared = c
            .open
            .file()
            .try_clone()
            .expect("the descriptor duplicates");
        c.sync_offset();
        shared
            .seek(SeekFrom::Start(4))
            .expect("a command consumed `one`");
        c.adopt_offset();
        assert_eq!(c.chars_left(), 15);
        c.sync_offset();
        assert_eq!(
            shared.stream_position().ok(),
            Some(4),
            "nothing read, nothing moved back"
        );
        shared
            .seek(SeekFrom::Start(8))
            .expect("a command consumed `two`");
        c.adopt_offset();
        assert_eq!(c.read_line(), b"three\n");
        assert_eq!(c.lines_left(), 1);
        c.sync_offset();
        assert_eq!(
            shared.stream_position().ok(),
            Some(14),
            "read, so moved to the read position"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// A command that read nothing leaves the read position alone, even
    /// when the console read on while the command ran.
    #[test]
    fn a_command_that_read_nothing_moves_nothing() {
        let (mut c, path) = console("idle", b"one\ntwo\nthree\nfour\n");
        assert_eq!(c.read_line(), b"one\n");
        c.sync_offset();
        assert_eq!(c.read_line(), b"two\n");
        c.adopt_offset();
        assert_eq!(c.read_line(), b"three\n");
        assert_eq!(c.lines_left(), 1);
        c.adopt_offset();
        assert_eq!(c.read_line(), b"four\n");
        let _ = std::fs::remove_file(&path);
    }

    /// A file that grows after its end was read is read on: the counts see
    /// the new bytes and the next line is the appended one.
    #[test]
    fn a_file_console_reads_what_is_appended() {
        let (mut c, path) = console("grow", b"a\nb\n");
        assert_eq!(c.read_line(), b"a\n");
        assert_eq!(c.read_line(), b"b\n");
        assert_eq!(counts(&mut c), (0, 0, false));
        append(&path, b"more\nlines");
        assert_eq!(counts(&mut c), (10, 2, true));
        assert_eq!(c.read_line(), b"more\n");
        assert_eq!(counts(&mut c), (5, 1, true));
        let _ = std::fs::remove_file(&path);
    }

    /// The console starts at the descriptor's offset, and anything but a
    /// regular file is not one.
    #[test]
    fn a_file_console_starts_where_the_descriptor_is() {
        use std::io::Seek;
        let path =
            std::env::temp_dir().join(format!("rexx-file-console-{}-start", std::process::id()));
        std::fs::write(&path, b"hello\nworld\n").expect("the console file is written");
        let mut file = std::fs::File::open(&path).expect("the console file opens");
        file.seek(std::io::SeekFrom::Start(6))
            .expect("the offset moves");
        let mut c = FileConsole::over(file).expect("a regular file is a file console");
        assert_eq!(counts(&mut c), (6, 1, true));
        assert_eq!(c.read_line(), b"world\n");
        let _ = std::fs::remove_file(&path);
        let null = std::fs::File::open("/dev/null").expect("/dev/null opens");
        assert!(FileConsole::over(null).is_none());
    }
}
