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

use crate::guards::{GuardKey, Reserve};
use crate::invocation::ProgramInput;
use crate::scheduler::ParkReason;
use crate::{Failure, Interp, Raised};

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
    /// A halt woke an access waiting in [`Interp::stdin_turn`]: its read
    /// answers nothing.
    halted: bool,
}

/// An access of standard input parked by [`Interp::stdin_turn`]: for `key`,
/// `.STDIN`'s guard, where another activity holds it, then for a chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StdinWait {
    key: Option<GuardKey>,
    wants: StdinWants,
}

/// What an access of standard input needs buffered before it can go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StdinWants {
    Line,
    Byte,
    /// Nothing: a count, which reads no input.
    Nothing,
}

impl StdinWait {
    pub(crate) fn key(self) -> Option<GuardKey> {
        self.key
    }
}

enum Source {
    /// Nothing to read, ever. Distinct from `Bytes` over an empty buffer only
    /// in costing no allocation; both answer `None` on the first read.
    Nothing,
    /// The process's standard input, read a chunk at a time off the baton,
    /// where a halt can abandon the wait ([`Interp::fill_stdin`]).
    Stdin(Stdin),
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

impl Input {
    /// Whether a read here can wait on something outside this process, which
    /// is what makes handing the buffered output over first worth doing.
    fn is_live(&self) -> bool {
        matches!(self.source, Source::Stdin(_))
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
                ProgramInput::Stdin => Source::Stdin(Stdin::default()),
                ProgramInput::Bytes(bytes) => Source::Bytes(Cursor::new(bytes)),
            },
            exhausted: false,
            lines_asked: false,
            halted: false,
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
    /// one, and otherwise [`Interp::linein_line`].
    ///
    /// **The queue is asked first and the route is not asked at all when it
    /// answers** -- measured, a program with one line pushed reads it and the
    /// destination sees a single `LINEIN`, for the second read.
    pub(crate) fn pull_line(&mut self) -> Result<Vec<u8>, Failure> {
        if self.holds_stdin_turn() {
            return self.linein_line();
        }
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
            return Ok(self.input_line()?.unwrap_or_default());
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
    pub(crate) fn input_line(&mut self) -> Result<Option<Vec<u8>>, Failure> {
        self.hand_over_before_read();
        if !self.fill_stdin(true)? {
            return Ok(None);
        }
        Ok(self.input.read_line())
    }

    /// Up to `wanted` bytes of `.input`, for `.STDIN~CHARIN`.
    pub(crate) fn input_bytes(&mut self, wanted: usize) -> Result<Vec<u8>, Failure> {
        self.hand_over_before_read();
        if !self.fill_stdin(false)? {
            return Ok(Vec::new());
        }
        Ok(self.input.read_bytes(wanted))
    }

    /// Reads standard input until it holds a line where `line`, else a byte,
    /// or has ended: each chunk off the baton, while the other activities run
    /// (rulings P60, P66). A halt that wakes the read answers `false`: the
    /// read answers nothing, as the oracle's interrupted read does, and the
    /// chunk it waited for is filed when it comes.
    fn fill_stdin(&mut self, line: bool) -> Result<bool, Failure> {
        if std::mem::take(&mut self.input.halted) {
            // The halt is taken where the oracle's read was when it came: in
            // the running activation, such as the `.INPUT` monitor's
            // `UNKNOWN`, whose clause end hands a `CALL ON HALT` to the
            // reading clause's own.
            if let Some(running) = self.running_activation().map(|running| running.id) {
                let depth = self.activity.fragment_depth;
                for pending in self.activity.pending_traps.iter_mut() {
                    if pending.request {
                        pending.activation = running;
                        pending.fragment_depth = depth;
                    }
                }
            }
            self.input.interrupted();
            return Ok(false);
        }
        let wants = match line {
            true => StdinWants::Line,
            false => StdinWants::Byte,
        };
        while self.request_stdin(wants) {
            if let Some(failure) = self.pinned_wait(ParkReason::Input) {
                return Err(failure);
            }
            if self.activity.woken_by_halt {
                self.input.interrupted();
                return Ok(false);
            }
        }
        self.input.reading_stdin();
        Ok(true)
    }

    /// Whether a read wanting `wants` must wait for a chunk of standard
    /// input: one is in flight, or this starts it.
    /// Where no pool thread can be reserved the chunk is read here instead.
    fn request_stdin(&mut self, wants: StdinWants) -> bool {
        let line = match wants {
            StdinWants::Line => true,
            StdinWants::Byte => false,
            StdinWants::Nothing => return false,
        };
        while self.input.stdin_wants(line) {
            if !self.input.start_stdin_read() {
                return true;
            }
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
            return true;
        }
        false
    }

    /// The park an access through `route` (`None` for no route) makes
    /// before it sends, from a frame that can park off every pinned one: for
    /// `.STDIN`'s guard where another activity holds it, else for the chunk
    /// it wants, holding that guard as the oracle's reader does across its
    /// read. `None` once it can go on, holding the guard until
    /// [`Interp::end_stdin_turn`], or where `route` ends elsewhere.
    ///
    /// # Errors
    ///
    /// 98.905 where the guard's owner waits on this activity.
    pub(crate) fn stdin_turn(
        &mut self,
        route: Option<ObjRef>,
        wants: StdinWants,
    ) -> Result<Option<ParkReason>, Failure> {
        if !self.input.is_live() || self.input.halted {
            return Ok(None);
        }
        let key = match route.filter(|route| *route != ObjRef::NIL) {
            Some(route) if !self.route_ends_at(route, self.bootstrap_stdin) => return Ok(None),
            Some(_) => self.stdin_key(),
            None => None,
        };
        if let Some(key) = key
            && self.activities.guards.is_live()
            && !self.granted(key)
            && let Reserve::Contended(owner) = self.take_guard(key)
        {
            if self.deadlocks(owner) {
                return Err(Raised::deadlock().into());
            }
            let me = self.running_activity();
            self.activities.guards.enqueue(key, me);
            return Ok(Some(ParkReason::Stdin(StdinWait {
                key: Some(key),
                wants,
            })));
        }
        Ok(self
            .request_stdin(wants)
            .then_some(ParkReason::Stdin(StdinWait { key, wants })))
    }

    /// [`Interp::stdin_turn`] for a `PARSE` from `source`, a line. `PULL`
    /// reads the queue instead where it has a line and no turn is held.
    pub(crate) fn parse_stdin_turn(
        &mut self,
        source: &rexx_parse::ParseSource,
    ) -> Result<Option<ParkReason>, Failure> {
        if matches!(source, rexx_parse::ParseSource::Pull)
            && self.queue.len() > 0
            && !self.holds_stdin_turn()
        {
            return Ok(None);
        }
        let route = self.local_route(b"INPUT")?;
        self.stdin_turn(route, StdinWants::Line)
    }

    /// The re-test of a woken [`Interp::stdin_turn`] park: `Some` to park
    /// again for a chunk. A halt that woke it makes the read answer nothing.
    pub(crate) fn retest_stdin(&mut self, wait: StdinWait) -> Option<ParkReason> {
        if std::mem::take(&mut self.activity.woken_by_halt) {
            self.input.halted = true;
            return None;
        }
        self.request_stdin(wait.wants)
            .then_some(ParkReason::Stdin(wait))
    }

    /// Ends an access [`Interp::stdin_turn`] let go on: `.STDIN`'s guard
    /// released where the running activity holds it.
    pub(crate) fn end_stdin_turn(&mut self) {
        self.input.halted = false;
        if self.activities.guards.is_live()
            && let Some(key) = self.stdin_key()
            && self.granted(key)
        {
            self.release_guard(key);
        }
    }

    /// Whether the running activity holds `.STDIN`'s guard from
    /// [`Interp::stdin_turn`].
    fn holds_stdin_turn(&mut self) -> bool {
        self.activities.guards.is_live() && self.stdin_key().is_some_and(|key| self.granted(key))
    }

    /// The guard `.STDIN`'s reads reserve.
    fn stdin_key(&mut self) -> Option<GuardKey> {
        let stdin = self.bootstrap_stdin?;
        let scope = self.lookup(stdin, crate::dispatch::LINEIN, None)?.scope;
        Some(GuardKey {
            object: stdin,
            scope,
        })
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
    /// it installed any: before a read, and before this thread idles.
    pub(crate) fn hand_over_output(&mut self) {
        if self.sinks.is_none() {
            return;
        }
        let mut out = std::mem::take(&mut self.out);
        let mut err = std::mem::take(&mut self.trace);
        let mut sinks = self.sinks.take().expect("a sink set that is present");
        if !out.is_empty() {
            (sinks.stdout)(&out);
        }
        if !err.is_empty() {
            (sinks.stderr)(&err);
        }
        self.sinks = Some(sinks);
        // The buffers themselves go back, emptied: their capacity is worth
        // keeping and everything in them has been handed over.
        out.clear();
        err.clear();
        self.out = out;
        self.trace = err;
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
