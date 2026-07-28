use rexx_parse::ProgramSource;

#[test]
fn sourceline_returns_lines_without_terminators() {
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec());
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say 1"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));
    // Out of range is an ERROR in the interpreter, not an empty answer:
    // sourceline(0) raises 40.14 and sourceline(99) raises 40.34. Verified.
    // `line` returning None is how this crate reports that; Task 3.8 turns
    // it into the right error number. Do not let it render as "".
    assert_eq!(src.line(3), None);
    assert_eq!(src.line(0), None);
}

#[test]
fn line_of_is_one_based() {
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec());
    assert_eq!(src.line_of(0), 1);
    assert_eq!(src.line_of(4), 1);
    assert_eq!(src.line_of(6), 2);
}

#[test]
fn source_may_hold_bytes_that_are_not_utf8() {
    // A Rexx literal may contain arbitrary bytes. Verified against the oracle:
    // a file holding a raw FF FE inside a literal runs, and c2x reports FFFE.
    // A String-typed source would refuse to construct here.
    let src = ProgramSource::new(b"s = '\xff\xfe'\n".to_vec());
    assert_eq!(src.line(1), Some(&b"s = '\xff\xfe'"[..]));
    assert_eq!(src.line_count(), 1);
}

#[test]
fn final_line_without_trailing_newline_still_counts() {
    // Verified: build/bin/rexx on this exact two-line, no-trailing-newline
    // file reports sourceline() == 2, and sourceline(1) has no newline in it.
    let src = ProgramSource::new(b"say sourceline()\nsay 2".to_vec());
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say sourceline()"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));
}

#[test]
fn crlf_terminators_are_excluded_from_line_content() {
    // Verified against build/bin/rexx: a CRLF file's sourceline(n) contains
    // neither the \r nor the \n, and the line count matches an LF-only file.
    let src = ProgramSource::new(b"say 1\r\nsay 2\r\n".to_vec());
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say 1"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));
}

#[test]
fn crlf_pair_is_one_terminator_but_lone_cr_ends_a_line_on_its_own() {
    // Verified against build/bin/rexx: a file using bare \r (no \n) as its
    // only line terminator (old Mac style) still reports the right line
    // count and content, matching ProgramSource.cpp's line_delimiters scan,
    // which treats \r and \n as equally valid terminators and only pairs a
    // \r immediately followed by \n into a single CRLF terminator.
    let src = ProgramSource::new(b"say 1\rsay 2\r".to_vec());
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say 1"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));

    // Both halves of this test's name, in one place: the CRLF pair collapses
    // to a single terminator, so the same two lines come out of a CRLF file.
    // Asserting the CONTENT and not only the count, because a count alone
    // cannot distinguish one terminator from two.
    let crlf = ProgramSource::new(b"say 1\r\nsay 2\r\n".to_vec());
    assert_eq!(crlf.line_count(), 2);
    assert_eq!(crlf.line(1), Some(&b"say 1"[..]));
    assert_eq!(crlf.line(2), Some(&b"say 2"[..]));

    // And the asymmetry that a CRLF-only rule gets wrong: \n\r is TWO
    // terminators, so an empty line sits between them. Oracle: sourceline()
    // reports 3 for this shape where \r\n reports 2.
    let lfcr = ProgramSource::new(b"a\n\rb\n".to_vec());
    assert_eq!(lfcr.line_count(), 3);
    assert_eq!(lfcr.line(1), Some(&b"a"[..]));
    assert_eq!(lfcr.line(2), Some(&b""[..]));
    assert_eq!(lfcr.line(3), Some(&b"b"[..]));
}

#[test]
fn line_of_holds_at_terminators_at_the_end_and_on_an_empty_source() {
    // The boundary class the brief flags as highest-risk, and the one an
    // off-by-one here would propagate into every later task's spans.
    let src = ProgramSource::new(b"ab\ncd\n".to_vec());

    // A byte on a terminator belongs to the line that terminator ends.
    assert_eq!(src.line_of(2), 1, "the \\n closing line 1");
    assert_eq!(src.line_of(5), 2, "the \\n closing line 2");

    // First and last content bytes of each line.
    assert_eq!(src.line_of(0), 1);
    assert_eq!(src.line_of(1), 1);
    assert_eq!(src.line_of(3), 2);
    assert_eq!(src.line_of(4), 2);

    // Exactly len, and past it: clamp to the last line rather than panic.
    // 6 is b"ab\ncd\n".len(); ProgramSource exposes no length accessor and
    // this test has no business adding one.
    assert_eq!(src.line_of(6), 2, "one past the last byte");
    assert_eq!(src.line_of(usize::MAX), 2, "far past the end");

    // An empty source has no lines, and line_of is still total.
    let empty = ProgramSource::new(Vec::new());
    assert_eq!(empty.line_count(), 0);
    assert_eq!(empty.line_of(0), 1, "total even with nothing to point at");
    assert_eq!(empty.line(1), None, "and line() disagrees, deliberately");
}

#[test]
fn a_source_of_only_terminators_is_all_empty_lines() {
    let src = ProgramSource::new(b"\n\n\n".to_vec());
    assert_eq!(src.line_count(), 3);
    for n in 1..=3 {
        assert_eq!(src.line(n), Some(&b""[..]), "line {n}");
    }
    assert_eq!(src.line_of(0), 1);
    assert_eq!(src.line_of(2), 3);
}

#[test]
fn lf_then_cr_is_two_terminators_producing_an_empty_line() {
    // Verified against build/bin/rexx: unlike CR-then-LF, LF-then-CR is NOT
    // collapsed into one terminator, so it produces an empty line between
    // the two real lines. sourceline() reported 3 for this exact layout.
    let src = ProgramSource::new(b"say 1\n\rsay 2\n".to_vec());
    assert_eq!(src.line_count(), 3);
    assert_eq!(src.line(1), Some(&b"say 1"[..]));
    assert_eq!(src.line(2), Some(&b""[..]));
    assert_eq!(src.line(3), Some(&b"say 2"[..]));
}

#[test]
fn ctrl_z_truncates_the_source_including_a_partial_line() {
    // Verified against build/bin/rexx: a 0x1A (Ctrl-Z) byte marks end of
    // file, DOS/CP-M style. Everything at and after it is discarded before
    // line scanning, even mid-line: sourceline() dropped from 3 to 1 for a
    // file with a mid-second-line 0x1A, and the truncated first line kept
    // exactly the bytes before the 0x1A.
    let src = ProgramSource::new(b"say 1\nsay 2\x1a more\nsay 3\n".to_vec());
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say 1"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));
}

#[test]
fn empty_source_has_no_lines() {
    let src = ProgramSource::new(Vec::new());
    assert_eq!(src.line_count(), 0);
    assert_eq!(src.line(1), None);
    assert_eq!(src.line(0), None);
}

#[test]
fn line_span_indexes_the_same_bytes_line_returns() {
    // The scanner needs absolute offsets, so `line_span` must agree with
    // `line` byte for byte, including for the CRLF and bare-CR cases the
    // terminator rules cover.
    let src = ProgramSource::new(b"say 1\r\nsay 22\rsay 333".to_vec());
    assert_eq!(src.line_count(), 3);
    for n in 1..=3 {
        let span = src.line_span(n).expect("line exists");
        assert_eq!(src.line(n).expect("line exists").len(), span.len());
    }
    assert_eq!(src.line_span(1), Some(0..5));
    // Line 2 starts after CRLF, both bytes consumed as one terminator.
    assert_eq!(src.line_span(2), Some(7..13));
    // Line 3 starts after a bare CR.
    assert_eq!(src.line_span(3), Some(14..21));
    assert_eq!(src.line_span(0), None);
    assert_eq!(src.line_span(4), None);
}
