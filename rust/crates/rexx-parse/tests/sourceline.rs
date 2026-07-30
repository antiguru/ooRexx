use std::borrow::Cow;

use rexx_parse::{ProgramSource, SourceKind, parse_interpret, parse_program};

#[test]
fn sourceline_returns_lines_without_terminators() {
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec(), SourceKind::Program);
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
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec(), SourceKind::Program);
    assert_eq!(src.line_of(0), 1);
    assert_eq!(src.line_of(4), 1);
    assert_eq!(src.line_of(6), 2);
}

#[test]
fn source_may_hold_bytes_that_are_not_utf8() {
    // A Rexx literal may contain arbitrary bytes. Verified against the oracle:
    // a file holding a raw FF FE inside a literal runs, and c2x reports FFFE.
    // A String-typed source would refuse to construct here.
    let src = ProgramSource::new(b"s = '\xff\xfe'\n".to_vec(), SourceKind::Program);
    assert_eq!(src.line(1), Some(&b"s = '\xff\xfe'"[..]));
    assert_eq!(src.line_count(), 1);
}

#[test]
fn final_line_without_trailing_newline_still_counts() {
    // Verified: build/bin/rexx on this exact two-line, no-trailing-newline
    // file reports sourceline() == 2, and sourceline(1) has no newline in it.
    let src = ProgramSource::new(b"say sourceline()\nsay 2".to_vec(), SourceKind::Program);
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say sourceline()"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));
}

#[test]
fn crlf_terminators_are_excluded_from_line_content() {
    // Verified against build/bin/rexx: a CRLF file's sourceline(n) contains
    // neither the \r nor the \n, and the line count matches an LF-only file.
    let src = ProgramSource::new(b"say 1\r\nsay 2\r\n".to_vec(), SourceKind::Program);
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
    let src = ProgramSource::new(b"say 1\rsay 2\r".to_vec(), SourceKind::Program);
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say 1"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));

    // Both halves of this test's name, in one place: the CRLF pair collapses
    // to a single terminator, so the same two lines come out of a CRLF file.
    // Asserting the CONTENT and not only the count, because a count alone
    // cannot distinguish one terminator from two.
    let crlf = ProgramSource::new(b"say 1\r\nsay 2\r\n".to_vec(), SourceKind::Program);
    assert_eq!(crlf.line_count(), 2);
    assert_eq!(crlf.line(1), Some(&b"say 1"[..]));
    assert_eq!(crlf.line(2), Some(&b"say 2"[..]));

    // And the asymmetry that a CRLF-only rule gets wrong: \n\r is TWO
    // terminators, so an empty line sits between them. Oracle: sourceline()
    // reports 3 for this shape where \r\n reports 2.
    let lfcr = ProgramSource::new(b"a\n\rb\n".to_vec(), SourceKind::Program);
    assert_eq!(lfcr.line_count(), 3);
    assert_eq!(lfcr.line(1), Some(&b"a"[..]));
    assert_eq!(lfcr.line(2), Some(&b""[..]));
    assert_eq!(lfcr.line(3), Some(&b"b"[..]));
}

#[test]
fn line_of_holds_at_terminators_at_the_end_and_on_an_empty_source() {
    // The boundary class the brief flags as highest-risk, and the one an
    // off-by-one here would propagate into every later task's spans.
    let src = ProgramSource::new(b"ab\ncd\n".to_vec(), SourceKind::Program);

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
    let empty = ProgramSource::new(Vec::new(), SourceKind::Program);
    assert_eq!(empty.line_count(), 0);
    assert_eq!(empty.line_of(0), 1, "total even with nothing to point at");
    assert_eq!(empty.line(1), None, "and line() disagrees, deliberately");
}

#[test]
fn a_source_of_only_terminators_is_all_empty_lines() {
    let src = ProgramSource::new(b"\n\n\n".to_vec(), SourceKind::Program);
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
    let src = ProgramSource::new(b"say 1\n\rsay 2\n".to_vec(), SourceKind::Program);
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
    let src = ProgramSource::new(
        b"say 1\nsay 2\x1a more\nsay 3\n".to_vec(),
        SourceKind::Program,
    );
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some(&b"say 1"[..]));
    assert_eq!(src.line(2), Some(&b"say 2"[..]));
}

#[test]
fn empty_source_has_no_lines() {
    let src = ProgramSource::new(Vec::new(), SourceKind::Program);
    assert_eq!(src.line_count(), 0);
    assert_eq!(src.line(1), None);
    assert_eq!(src.line(0), None);
}

#[test]
fn line_span_indexes_the_same_bytes_line_returns() {
    // The scanner needs absolute offsets, so `line_span` must agree with
    // `line` byte for byte, including for the CRLF and bare-CR cases the
    // terminator rules cover.
    let src = ProgramSource::new(b"say 1\r\nsay 22\rsay 333".to_vec(), SourceKind::Program);
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

// ---------------------------------------------------------------------------
// Reconstructing the text TRACE prints on a `*-*` line, from
// `Instruction::clause_span` and the retained source. Reconstruction and line
// lookup are the same concern from two directions, so both live in this file.
//
// Every expected string below is measured: `( ulimit -v 1048576;
// build/bin/rexx FILE ) | cat -A`, captured 2026-07-28. The `*-*` text is the
// oracle's line after stripping the line number, the marker and the leading
// indentation, and NOTHING else: a trailing blank before a `then` and a
// terminating `;` are part of the text.
// ---------------------------------------------------------------------------

/// One measured `*-*` line: the source line number the interpreter printed,
/// the instruction the line came from, and the stripped text.
type TracedLine = (usize, usize, &'static [u8]);

/// Checks every measured `*-*` line against the clause it came from.
///
/// Per-line rather than sequence equality: `trace r` re-traces a loop body
/// once per iteration, so the `*-*` lines outnumber the clauses and a count
/// assertion would fail on any program containing a loop.
///
/// Completeness is the CALLER's obligation. Nothing here can know the oracle
/// transcript, so a list that omits a `*-*` line still passes. The
/// transcripts these lists were checked against, one per probe and
/// unfiltered, live in
/// `.superpowers/sdd/2026-07-28-phase-3-parser/task-3.9-report.md`.
fn assert_traced(text: &[u8], traced: &[TracedLine]) {
    assert!(
        !traced.is_empty(),
        "an empty expectation list checks nothing"
    );
    let program = parse_program(text.to_vec()).expect("the probe parses");
    for &(line, index, expected) in traced {
        let span = program.main.instructions[index].clause_span.clone();
        let got = program
            .source
            .join_span(span.clone())
            .expect("a clause span indexes the retained source");
        assert_eq!(
            got.as_ref(),
            expected,
            "instruction {index} (span {span:?}): reconstructed {:?}, oracle printed {:?}",
            String::from_utf8_lossy(got.as_ref()),
            String::from_utf8_lossy(expected),
        );
        assert_eq!(
            program.source.line_of(span.start),
            line,
            "instruction {index}: a *-* line carries the clause's FIRST line number"
        );
    }
}

#[test]
fn trace_output_rex_reconstructs_every_traced_line() {
    // Measured on the corpus file itself. It sets `trace i`, so the raw
    // capture also carries value-marker lines and the program's own output.
    // Those need an executor and are Phase 4's, so only the six *-* lines
    // appear here. (The markers THIS file happens to emit are >L>, >V>,
    // >O>, >>> and >=>. That is not the set: the authority is "everything
    // except *-*", eighteen prefixes, and enumerating them has gone wrong
    // three times in this phase.) Source line 4 is three clauses: the
    // condition KEEPS its trailing blank and stops before `then`, `then` is
    // a clause of its own, and the THEN arm is the third.
    let text = include_bytes!("../../../corpus/lang/trace_output.rex");
    assert_traced(
        text,
        &[
            (2, 1, b"x = 1 + 1"),
            (3, 2, b"y = x * 3"),
            (4, 3, b"if y > 5 "),
            (4, 4, b"then"),
            (4, 5, b"say \"big\""),
            (5, 6, b"trace off"),
        ],
    );
}

#[test]
fn probe_a_keeps_semicolons_and_end_is_its_own_clause() {
    // Scratch probe A from the task brief. Nine *-* lines for six
    // instructions: the loop's header, body and END repeat per iteration
    // and the header prints once more for the exit test, which is why the
    // triples below repeat instruction indices.
    let text = b"/* probe A: terminators are inside the clause span */\ntrace r\nnop;\ndo i = 1 to 2; say i; end\ntrace off\n";
    assert_traced(
        text,
        &[
            (3, 1, b"nop;"),
            (4, 2, b"do i = 1 to 2;"),
            (4, 3, b"say i;"),
            (4, 4, b"end"),
            (4, 2, b"do i = 1 to 2;"),
            (4, 3, b"say i;"),
            (4, 4, b"end"),
            (4, 2, b"do i = 1 to 2;"),
            (5, 5, b"trace off"),
        ],
    );
}

#[test]
fn probe_b_traces_a_label_with_its_colon() {
    // Scratch probe B from the task brief: `here:` / `nop;` / `say "two"`
    // are three clauses on one source line.
    let text = b"/* probe B: a label is its own clause, colon included */\ntrace r\nhere: nop; say \"two\"\ntrace off\n";
    assert_traced(
        text,
        &[
            (3, 1, b"here:"),
            (3, 2, b"nop;"),
            (3, 3, b"say \"two\""),
            (4, 4, b"trace off"),
        ],
    );
}

#[test]
fn probe_g_an_else_arm_continues_like_any_other_clause() {
    // Scratch probe G, measured 2026-07-29: the untaken THEN branch is not
    // traced, `else` carries no blank on either side, and the ELSE arm's
    // continuation keeps its four blanks like any other.
    let text = b"trace r\nif 1 = 2 then nop\nelse say 1,\n    2\ntrace off\n";
    assert_traced(
        text,
        &[
            (2, 1, b"if 1 = 2 "),
            (3, 4, b"else"),
            (3, 5, b"say 1,    2"),
            (5, 6, b"trace off"),
        ],
    );
}

#[test]
fn probe_h_an_otherwise_arm_continues_like_any_other_clause() {
    // Scratch probe H, measured 2026-07-29, the SELECT equivalent of probe
    // G. The WHEN condition keeps its trailing blank the way an IF's does,
    // `otherwise` carries no blank on either side, and the arm's
    // continuation joins with its four blanks kept.
    let text =
        b"trace r\nselect\n  when 1 = 2 then nop\n  otherwise say 1,\n    2\nend\ntrace off\n";
    assert_traced(
        text,
        &[
            (2, 1, b"select"),
            (3, 2, b"when 1 = 2 "),
            (4, 5, b"otherwise"),
            (4, 6, b"say 1,    2"),
            (6, 7, b"end"),
            (7, 8, b"trace off"),
        ],
    );
}

#[test]
fn probe_i_a_three_fragment_continuation_drops_every_terminator() {
    // Scratch probe I, measured 2026-07-29: two continuations in one
    // clause, so the join drops two terminators and keeps both
    // continuation lines' leading blanks. Every other probe joins exactly
    // two fragments, and this is the one that makes the join loop more
    // than once.
    let text = b"trace r\nsay 1,\n  2,\n    3\ntrace off\n";
    assert_traced(text, &[(2, 1, b"say 1,  2,    3"), (5, 2, b"trace off")]);
}

#[test]
fn probe_j_a_multi_label_clause_is_one_clause_per_label() {
    // Scratch probe J, measured 2026-07-29: `a: b: nop` traces as three
    // clauses on one source line, each label with its own colon.
    //
    // This one pins span extraction, not the join. None of its three spans
    // holds a terminator, so `span_bytes` alone already answers correctly
    // and a `join_span` replaced by `span_bytes` leaves this test green.
    // Measured: defeating the join reddens G, H and I but not J, so the
    // four probes added here give three tests of the join, not four.
    let text = b"trace r\na: b: nop\ntrace off\n";
    assert_traced(
        text,
        &[
            (2, 1, b"a:"),
            (2, 2, b"b:"),
            (2, 3, b"nop"),
            (3, 4, b"trace off"),
        ],
    );
}

#[test]
fn a_continued_clause_joins_without_its_terminator() {
    // Scratch probe C: the comma kept, the newline dropped, the
    // continuation line's four leading blanks kept, and the *-* line number
    // is the clause's FIRST line.
    let text = b"/* probe C: continuation join */\ntrace r\nsay \"x\",\n    \"y\"\ntrace off\n";
    assert_traced(
        text,
        &[(3, 1, b"say \"x\",    \"y\""), (5, 2, b"trace off")],
    );

    // The trap the task brief names: `span_bytes` alone is WRONG here,
    // because the clause span still CONTAINS the terminator it joins out.
    let program = parse_program(text.to_vec()).unwrap();
    let span = program.main.instructions[1].clause_span.clone();
    assert_eq!(
        program.source.span_bytes(span).unwrap(),
        b"say \"x\",\n    \"y\""
    );
}

#[test]
fn a_continued_clause_span_contains_the_terminator_it_joins_out() {
    // The brief's second continuation measurement: `say 1,` / `  + 2`
    // traces as `say 1,  + 2` (probe D) and, parsed on its own, has clause
    // span 0..12 with the newline at byte 6 inside it.
    let text = b"say 1,\n  + 2";
    let program = parse_program(text.to_vec()).unwrap();
    let span = program.main.instructions[0].clause_span.clone();
    assert_eq!(span, 0..12);
    assert_eq!(
        program.source.span_bytes(span.clone()).unwrap(),
        b"say 1,\n  + 2"
    );
    assert_eq!(
        program.source.join_span(span).unwrap().as_ref(),
        b"say 1,  + 2"
    );
}

#[test]
fn a_crlf_continuation_drops_both_terminator_bytes() {
    // Scratch probe E, the CRLF spelling of probe C: the oracle traces the
    // identical joined text, so the join drops the whole two-byte pair.
    let text = b"trace r\r\nsay \"x\",\r\n    \"y\"\r\ntrace off\r\n";
    assert_traced(
        text,
        &[(2, 1, b"say \"x\",    \"y\""), (4, 2, b"trace off")],
    );
}

#[test]
fn interpret_fragment_clauses_reconstruct_too() {
    // Scratch probe F: `trace r` around `interpret 'nop; say 1'` traces the
    // interpreted clauses as `nop;` and `say 1`, from the fragment's own
    // one-line text. The line number the oracle prints is the INTERPRET
    // instruction's own, which is the caller's to resolve, so only the text
    // is checked here.
    let fragment = parse_interpret(b"nop; say 1".to_vec()).expect("the fragment parses");
    let texts: Vec<_> = fragment
        .body
        .instructions
        .iter()
        .map(|i| {
            fragment
                .source
                .join_span(i.clause_span.clone())
                .expect("a clause span indexes the fragment text")
        })
        .collect();
    assert_eq!(texts, [&b"nop;"[..], &b"say 1"[..]]);
}

#[test]
fn join_span_borrows_on_one_line_and_agrees_with_span_bytes_about_none() {
    let src = ProgramSource::new(b"say 1\nsay 2,\n 3\n".to_vec(), SourceKind::Program);
    // An uncontinued clause sits on one line, and the join is then exactly
    // `span_bytes`, borrowed rather than copied.
    match src.join_span(0..5) {
        Some(Cow::Borrowed(bytes)) => assert_eq!(bytes, b"say 1"),
        other => panic!("expected a borrowed single-line join, got {other:?}"),
    }
    // A span crossing a terminator joins owned.
    match src.join_span(6..15) {
        Some(Cow::Owned(bytes)) => assert_eq!(bytes, b"say 2, 3"),
        other => panic!("expected an owned multi-line join, got {other:?}"),
    }
    // `None` exactly when `span_bytes` answers `None`.
    assert_eq!(src.join_span(0..999), None);
    assert_eq!(src.span_bytes(0..999), None);
    // An INTERPRET source is one line end to end, so nothing ever joins.
    let interp = ProgramSource::new(b"say 1; say 2".to_vec(), SourceKind::Interpret);
    assert!(matches!(interp.join_span(0..12), Some(Cow::Borrowed(_))));
    // An empty span borrows even on a source with no lines at all, which
    // keeps the contract exact: owned means a terminator was dropped.
    let empty = ProgramSource::new(Vec::new(), SourceKind::Program);
    match empty.join_span(0..0) {
        Some(Cow::Borrowed(bytes)) => assert_eq!(bytes, b""),
        other => panic!("expected a borrowed empty join, got {other:?}"),
    }
}

#[test]
fn interpret_text_is_one_line_from_end_to_end() {
    // `ArrayProgramSource` holds the interpret string as a single array
    // element (`LanguageParser.cpp:450`), so neither of `new`'s program rules
    // applies: nothing splits it and nothing truncates it. Measured against
    // the oracle: `interpret "say 1" || '0a'x || "say 2"` is error 13.1 on the
    // 0A byte, and so are the `'0d'x`, `'0d0a'x` and `'1a'x` versions, which
    // can only happen if those bytes are still on the line.
    let src = ProgramSource::new(b"say 1\n\rsay 2\x1amore".to_vec(), SourceKind::Interpret);
    assert_eq!(src.kind(), SourceKind::Interpret);
    assert_eq!(src.line_count(), 1);
    assert_eq!(src.line(1), Some(&b"say 1\n\rsay 2\x1amore"[..]));
    assert_eq!(src.line_span(1), Some(0..17));
    assert_eq!(src.line(2), None);
    // Every byte is on line 1, including the ones a program would have used as
    // terminators.
    for byte in 0..17 {
        assert_eq!(src.line_of(byte), 1, "byte {byte}");
    }
    assert_eq!(src.span_bytes(6..12), Some(&b"\rsay 2"[..]));

    // The identical bytes as a program split into three lines and truncate at
    // the Ctrl-Z, which is what makes this a property of the source.
    let program = ProgramSource::new(b"say 1\n\rsay 2\x1amore".to_vec(), SourceKind::Program);
    assert_eq!(program.line_count(), 3);
    assert_eq!(program.line(2), Some(&b""[..]));
    assert_eq!(program.line(3), Some(&b"say 2"[..]));

    // Empty interpret text still has its one line, where an empty program has
    // none. Measured: `interpret ""` is accepted and the program runs on.
    let empty = ProgramSource::new(Vec::new(), SourceKind::Interpret);
    assert_eq!(empty.line_count(), 1);
    assert_eq!(empty.line(1), Some(&b""[..]));
    assert_eq!(
        ProgramSource::new(Vec::new(), SourceKind::Program).line_count(),
        0
    );
}
