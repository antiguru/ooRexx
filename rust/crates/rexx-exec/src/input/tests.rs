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
