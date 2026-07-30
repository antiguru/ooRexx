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

//! The phase's error gate: what this parser rejects, what it accepts, and the
//! number, sub-number and line it reports, all against `build/bin/rexxc`.
//!
//! Two directions, because either alone is satisfiable without meaning
//! anything. **Soundness**: every program the oracle refuses to TRANSLATE, this
//! parser rejects with the same number and sub-number on the same line.
//! **Completeness**: every program the oracle translates, this parser accepts,
//! and so is every program it rejects for a reason that is not a translation
//! error. A parser that rejected nothing would pass the first half and fail the
//! second, and one that rejected everything the reverse.
//!
//! Which of those a row is, is a **field in the corpus** and not something
//! derived from the error number here. See the corpus header for why: the two
//! install-time classes are 98.903 and 90.999, so a `98.9xx` prefix rule would
//! have covered half of them and read as correct.
//!
//! The oracle's answers are baked into `rust/corpus/errors/parse-errors.tsv`
//! rather than recomputed, the same way `corpus/expr/precedence.tsv` bakes in
//! the interpreter's expression values: `cargo test` must not need a built C++
//! interpreter. That file's own header records how it was generated. Nothing in
//! it is this parser's answer, so a divergence surfaces as a failure here and
//! never as an expected value.
//!
//! The `samples/` and bootstrap half of the completeness direction reads the
//! real files instead, because all 303 of them are in the tree already and all
//! 303 get rc 0 from `rexxc` (measured), so there is no per-file expectation to
//! curate.
//!
//! # What the corpus cannot reach, and what covers it instead
//!
//! Every corpus program is `SourceKind::Program`, because `rexxc` compiles files
//! and cannot be pointed at an `INTERPRET` string. The `INTERPRET`-only errors
//! are therefore measured through the condition object instead, and
//! `the_interpret_only_errors_match_the_condition_objects_own_code` holds those
//! measurements.
//!
//! One input is deliberately absent rather than unreachable: an input holding
//! *two* errors, where eager scanning reports the later one. `Task 3.3` records
//! that deviation and `the_eager_scan_deviation_still_deviates` pins it, outside
//! the corpus, because a corpus row for it would enshrine our answer as the
//! expected one.

use std::path::{Path, PathBuf};

use rexx_parse::{ParseError, ProgramSource, SourceKind, parse_interpret, parse_program};

/// What kind of answer the oracle gave, as the corpus records it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Class {
    /// `rexxc` refused to translate. This parser must reproduce the number, the
    /// sub-number and the line.
    Translation,
    /// `rexxc` rejected it for something outside the program text, a library or
    /// an external routine it could not bind. This parser must accept it.
    ///
    /// Not "after translating it": both codes fire mid-translation, and `rexxc`
    /// has no install step yet reports them. See the corpus header for the raise
    /// sites and the measurement.
    Install,
    /// `rexxc` answered rc 0.
    Accepted,
}

/// One row of the corpus: a program and what the oracle answered for it.
struct Case {
    program: Vec<u8>,
    class: Class,
    /// `None` when `rexxc` answered rc 0, meaning the program translates.
    error: Option<(u16, u16)>,
    /// The line the oracle's main message named. `None` for an accepted
    /// program.
    line: Option<usize>,
    /// 1-based row number in the corpus file, for a failure message that can be
    /// looked up.
    row: usize,
}

fn corpus_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/errors/parse-errors.tsv")
}

/// Reverses the corpus file's escaping: `\\`, `\t`, `\n`, `\r` and `\xNN`.
///
/// A program is bytes and not text -- a literal may hold anything, and one case
/// here is a non-UTF-8 byte pair -- so this produces `Vec<u8>` and the escaping
/// exists precisely so that a row stays one line of a text file.
fn unescape(field: &str, row: usize) -> Vec<u8> {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let next = *bytes
            .get(i + 1)
            .unwrap_or_else(|| panic!("row {row}: trailing backslash"));
        match next {
            b'\\' => out.push(b'\\'),
            b't' => out.push(b'\t'),
            b'n' => out.push(b'\n'),
            b'r' => out.push(b'\r'),
            b'x' => {
                let hex = field
                    .get(i + 2..i + 4)
                    .unwrap_or_else(|| panic!("row {row}: truncated \\x escape"));
                out.push(
                    u8::from_str_radix(hex, 16)
                        .unwrap_or_else(|e| panic!("row {row}: bad \\x escape {hex:?}: {e}")),
                );
                i += 2;
            }
            other => panic!("row {row}: unknown escape \\{}", other as char),
        }
        i += 2;
    }
    out
}

fn cases() -> Vec<Case> {
    let text = std::fs::read_to_string(corpus_path()).expect("the error corpus is readable");
    let mut cases = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let row = index + 1;
        if line.starts_with('#') || line.starts_with("class\t") {
            continue;
        }
        let mut fields = line.splitn(4, '\t');
        let mut field = |what: &str| {
            fields
                .next()
                .unwrap_or_else(|| panic!("row {row}: no {what} field"))
                .to_string()
        };
        let class = field("class");
        let expect = field("expect");
        let at = field("line");
        let program = field("program");
        let class = match class.as_str() {
            "translation" => Class::Translation,
            "install" => Class::Install,
            "-" => Class::Accepted,
            other => panic!("row {row}: {other:?} is not a class"),
        };
        let error = if expect == "ok" {
            assert_eq!(class, Class::Accepted, "row {row}: rc 0 is not a rejection");
            assert_eq!(at, "-", "row {row}: an accepted program has no line");
            None
        } else {
            assert_ne!(
                class,
                Class::Accepted,
                "row {row}: a rejection needs a class"
            );
            let (code, sub) = expect
                .split_once('.')
                .unwrap_or_else(|| panic!("row {row}: {expect:?} is not major.sub"));
            Some((
                code.parse().unwrap_or_else(|e| panic!("row {row}: {e}")),
                sub.parse().unwrap_or_else(|e| panic!("row {row}: {e}")),
            ))
        };
        let line = error
            .is_some()
            .then(|| at.parse().unwrap_or_else(|e| panic!("row {row}: {e}")));
        cases.push(Case {
            program: unescape(&program, row),
            class,
            error,
            line,
            row,
        });
    }
    cases
}

fn parse(program: &[u8]) -> Result<(), ParseError> {
    parse_program(program.to_vec()).map(|_| ())
}

/// Whether `text` holds an `&`-and-digit substitution placeholder.
///
/// A second implementation of `error.rs`'s own check, deliberately: a test that
/// called the code under test to decide what "unfilled" means would pass however
/// that code was broken.
fn holds_placeholder(text: &str) -> bool {
    text.char_indices().any(|(at, c)| {
        c == '&'
            && text[at + 1..]
                .chars()
                .next()
                .is_some_and(|next| next.is_ascii_digit())
    })
}

// ---- the corpus decoder, asserted before anything relies on it ----

#[test]
fn the_corpus_escaping_decodes_every_escape_the_format_defines() {
    // The escaping is the whole reason a program with newlines, a tab and a
    // non-UTF-8 byte fits on one line of a text file, so it is asserted directly
    // rather than only through whether the corpus happens to pass.
    assert_eq!(
        unescape(r"a\tb\nc\\d\r\xc3\xa4e", 0),
        b"a\tb\nc\\d\r\xc3\xa4e".to_vec()
    );
}

#[test]
fn the_corpus_escaping_leaves_an_unescaped_program_alone() {
    assert_eq!(unescape("say 1", 0), b"say 1".to_vec());
}

#[test]
fn the_corpus_escaping_decodes_an_empty_field_to_no_bytes() {
    assert_eq!(unescape("", 0), Vec::<u8>::new());
}

#[test]
fn a_doubled_backslash_does_not_swallow_the_byte_after_it() {
    // A decoder that dropped the doubling would read `\\n` as a newline.
    assert_eq!(unescape(r"\\n", 0), b"\\n".to_vec());
}

#[test]
fn the_hex_escape_reaches_both_ends_of_the_byte_range() {
    assert_eq!(unescape(r"\x00\xff", 0), vec![0x00, 0xff]);
}

#[test]
fn the_gates_own_placeholder_check_finds_a_placeholder() {
    // Asserted for the same reason `error.rs` asserts its own copy: a check that
    // silently never fired would make the message assertion below vacuous, and
    // that is exactly what a mutation of it must not get away with.
    assert!(holds_placeholder("found &1."));
    assert!(holds_placeholder("&9"));
}

#[test]
fn the_gates_own_placeholder_check_scans_past_a_bare_ampersand() {
    assert!(holds_placeholder("A & B &2"));
}

#[test]
fn the_gates_own_placeholder_check_rejects_text_with_no_placeholder() {
    assert!(!holds_placeholder("A & B"));
    assert!(!holds_placeholder("Invalid subkeyword found."));
    assert!(!holds_placeholder(""));
}

// ---- the corpus file's own shape ----

#[test]
fn the_corpus_holds_at_least_the_rows_it_was_measured_with() {
    let cases = cases();
    // Floors rather than exact counts, because the corpus may grow. A file that
    // silently emptied would make every loop below vacuously pass.
    assert!(
        cases.len() >= 1020,
        "expected at least 1020 corpus rows, found {}",
        cases.len()
    );
    let translation = cases
        .iter()
        .filter(|c| c.class == Class::Translation)
        .count();
    let install = cases.iter().filter(|c| c.class == Class::Install).count();
    let accepted = cases.iter().filter(|c| c.class == Class::Accepted).count();
    assert!(
        translation >= 567,
        "expected at least 567 translation errors, found {translation}"
    );
    assert!(
        accepted >= 444,
        "expected at least 444 accepted programs, found {accepted}"
    );
    // Exact, and the only count here that is: every install-time row is one of
    // nine hand-classified programs. A tenth has to be argued, not absorbed.
    assert_eq!(install, 9, "the install-time classification grew");
}

#[test]
fn no_corpus_program_appears_twice() {
    // A duplicate row would inflate every count without adding a case.
    let cases = cases();
    let mut programs: Vec<&[u8]> = cases.iter().map(|c| c.program.as_slice()).collect();
    programs.sort_unstable();
    let before = programs.len();
    programs.dedup();
    assert_eq!(
        before,
        programs.len(),
        "the corpus holds a duplicate program"
    );
}

#[test]
fn every_recorded_line_is_a_line_its_program_has() {
    let cases = cases();
    for case in &cases {
        let Some(line) = case.line else { continue };
        let source = ProgramSource::new(case.program.clone(), SourceKind::Program);
        assert!(
            line >= 1 && line <= source.line_count(),
            "row {}: the oracle's line {line} is not one of the program's {} lines",
            case.row,
            source.line_count()
        );
    }
}

// ---- soundness ----

#[test]
fn every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way() {
    let cases = cases();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for case in &cases {
        if case.class != Class::Translation {
            continue;
        }
        let expected = case.error.expect("a translation error has a number");
        let ours = parse(&case.program).err().map(|e| (e.code, e.sub));
        if label_colon_deviation(ours, expected) {
            continue;
        }
        checked += 1;
        if ours != Some(expected) {
            wrong.push(format!(
                "row {}: expected {}.{}, got {}",
                case.row,
                expected.0,
                expected.1,
                match ours {
                    None => "no error".to_string(),
                    Some((code, sub)) => format!("{code}.{sub}"),
                }
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {checked}:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert!(
        checked >= 565,
        "only {checked} rejections were checked; the deviation rule is absorbing cases"
    );
}

#[test]
fn the_reported_line_matches_the_oracles_main_message() {
    let cases = cases();
    let mut wrong = Vec::new();
    for case in &cases {
        if case.class != Class::Translation {
            continue;
        }
        let expected = case.error.expect("a translation error has a number");
        let Err(error) = parse(&case.program) else {
            continue;
        };
        if (error.code, error.sub) != expected {
            continue;
        }
        let source = ProgramSource::new(case.program.clone(), SourceKind::Program);
        let ours = error.line(&source);
        let theirs = case.line.expect("a translation error has a line");
        if ours != theirs {
            wrong.push(format!(
                "row {}: error {}.{} reported on line {ours}, oracle says {theirs}",
                case.row, expected.0, expected.1
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn the_line_check_sees_more_than_one_line_convention() {
    // Line 1 is the answer for most of the corpus, so a check that only saw
    // one-line programs would pass without discriminating between the three
    // conventions the oracle uses. Count the rows reported past line 1.
    let cases = cases();
    let past_first_line = cases
        .iter()
        .filter(|c| c.class == Class::Translation && c.line.is_some_and(|line| line > 1))
        .count();
    assert!(
        past_first_line >= 140,
        "only {past_first_line} corpus errors are reported past line 1, so the \
         line check is not discriminating between line conventions"
    );
}

// ---- completeness ----

#[test]
fn every_program_the_oracle_translates_this_parser_accepts() {
    let cases = cases();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for case in &cases {
        if case.class != Class::Accepted {
            continue;
        }
        checked += 1;
        if let Err(e) = parse(&case.program) {
            wrong.push(format!("row {}: rejected with {e}", case.row));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(
        checked >= 444,
        "only {checked} accepted programs were checked"
    );
}

#[test]
fn a_rejection_that_is_not_a_translation_error_is_accepted() {
    // The oracle rejects all nine of these and none of the nine is a translation
    // error: the failure is that a library or a registered routine could not be
    // bound, which depends on the machine and never on the program text. It is
    // NOT that the directive parsed and the interpreter then failed to bind it:
    // both codes are raised from inside the parser, and `rexxc`, which never
    // installs anything, reports them. The classification is the corpus's, per
    // row, so this loop reads it rather than inferring it from 98 or 90.
    let cases = cases();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for case in &cases {
        if case.class != Class::Install {
            continue;
        }
        checked += 1;
        if let Err(e) = parse(&case.program) {
            wrong.push(format!(
                "row {}: {:?} is not a translation error but was rejected with {e}",
                case.row,
                String::from_utf8_lossy(&case.program)
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(
        checked, 9,
        "the install-time rows are not all being checked"
    );
}

// ---- what the messages read like ----

#[test]
fn every_error_the_corpus_raises_renders_a_non_empty_message() {
    let cases = cases();
    let mut checked = 0;
    for case in &cases {
        let Err(error) = parse(&case.program) else {
            continue;
        };
        checked += 1;
        assert!(
            !error.message().is_empty(),
            "row {}: empty message",
            case.row
        );
    }
    assert!(checked >= 567, "only {checked} messages were rendered");
}

#[test]
fn no_message_the_corpus_raises_holds_an_unfilled_substitution() {
    // The one property the scope decision does not relax: whatever the message
    // says, it must not say `&1`. An unfilled placeholder reaching a user would
    // be worse than the generic text that replaces it.
    let cases = cases();
    for case in &cases {
        let Err(error) = parse(&case.program) else {
            continue;
        };
        let message = error.message();
        assert!(
            !holds_placeholder(&message),
            "row {}: {}.{} renders {message:?}",
            case.row,
            error.code,
            error.sub
        );
    }
}

// ---- the recorded deviations, each pinned in both directions ----

/// Whether `(ours, oracle)` is the recorded label-colon deviation.
///
/// A rule rather than a list of programs, so a new program of the same shape is
/// classified instead of failing, while a new *shape* still fails. The count
/// asserted below is what stops it from silently absorbing more than the two
/// cases it was written for.
fn label_colon_deviation(ours: Option<(u16, u16)>, oracle: (u16, u16)) -> bool {
    matches!((ours, oracle), (Some((18, 1 | 2)), (35, 1)))
}

#[test]
fn a_then_label_is_a_missing_then_here_and_a_bad_expression_there() {
    // Structural, not a near miss: Task 3.4 splits `then:` into a label, so the
    // clause the oracle finds an invalid expression in does not exist by the time
    // the grammar runs. Both reject, which is the part that matters.
    //
    // Measured, `if 1 = 1` then `then: nop`:
    //     Error 35 running <file> line 2:  Invalid expression.
    //     Error 35.1:  Incorrect expression detected at ":".
    let error = parse(b"if 1 = 1\nthen: nop").expect_err("both reject it");
    assert_eq!((error.code, error.sub), (18, 1));
}

#[test]
fn a_when_label_is_a_missing_then_here_and_a_bad_expression_there() {
    // The WHEN spelling, which the oracle also answers 35.1 for, measured the
    // same way against `select` / `when 1 = 1` / `then: nop` / `end`.
    let error = parse(b"select\nwhen 1 = 1\nthen: nop\nend").expect_err("both reject it");
    assert_eq!((error.code, error.sub), (18, 2));
}

#[test]
fn the_label_colon_rule_fires_for_the_pair_it_was_written_for() {
    assert!(label_colon_deviation(Some((18, 1)), (35, 1)));
    assert!(label_colon_deviation(Some((18, 2)), (35, 1)));
}

#[test]
fn the_label_colon_rule_does_not_fire_for_a_near_miss() {
    // The other direction, which is what keeps the rule from waving through a
    // real regression that happens to involve 18.x or 35.x.
    assert!(!label_colon_deviation(Some((18, 1)), (18, 1)));
    assert!(!label_colon_deviation(Some((18, 1)), (35, 901)));
    assert!(!label_colon_deviation(Some((18, 3)), (35, 1)));
    assert!(!label_colon_deviation(None, (35, 1)));
}

#[test]
fn the_label_colon_rule_covers_exactly_the_two_corpus_rows_it_was_written_for() {
    let cases = cases();
    let count = cases
        .iter()
        .filter(|case| {
            case.class == Class::Translation
                && label_colon_deviation(
                    parse(&case.program).err().map(|e| (e.code, e.sub)),
                    case.error.expect("a translation error has a number"),
                )
        })
        .count();
    assert_eq!(count, 2, "the label-colon deviation grew");
}

#[test]
fn the_eager_scan_deviation_still_deviates() {
    // Task 3.3's recorded deviation, and the only one with no corpus row: this
    // input holds TWO errors, and a row for it would record our answer as the
    // expected one and stop the gate being able to see it.
    //
    // Measured, `say )` on line 1 and `'unclosed` on line 3:
    //     1 *-* say )
    //     Error 37 running <file> line 1:  Unexpected ",", ")", or "]".
    //     Error 37.2:  Unmatched ")" in expression.
    // The interpreter interleaves scanning and parsing and so reports the FIRST
    // error. Task 3.3 scans the whole program up front, so the later scan error
    // is raised before the earlier parse error is ever reached.
    let program = b"say )\n\n'unclosed\n";
    let source = ProgramSource::new(program.to_vec(), SourceKind::Program);
    let error = parse_program(program.to_vec()).expect_err("both reject it");
    assert_eq!(
        (error.code, error.sub),
        (6, 2),
        "ours is the unterminated-literal scan error"
    );
    assert_eq!(
        error.line(&source),
        3,
        "reported against the literal's clause"
    );
}

#[test]
fn the_eager_scan_deviation_needs_both_errors_to_appear() {
    // The other direction: neither half alone deviates, so the deviation is the
    // masking and not either error. `say )` on its own is the oracle's own 37.2
    // on line 1, and the unterminated literal on its own is 6.2 on its own line.
    let error = parse_program(b"say )\n".to_vec()).expect_err("a stray paren is an error");
    assert_eq!((error.code, error.sub), (37, 2));
    let program = b"nop\n\n'unclosed\n";
    let source = ProgramSource::new(program.to_vec(), SourceKind::Program);
    let error = parse_program(program.to_vec()).expect_err("an open literal is an error");
    assert_eq!((error.code, error.sub), (6, 2));
    assert_eq!(error.line(&source), 3);
}

#[test]
fn no_corpus_row_holds_the_eager_scan_shape() {
    // The exclusion, asserted rather than assumed: no corpus program may both
    // fail to scan and hold an earlier parse error. Truncating at the failing
    // clause's own byte leaves every complete clause before it, so if that prefix
    // does not parse either, the program has two errors and does not belong here.
    let cases = cases();
    let scanner_classes = [6u16, 13, 15, 30];
    let mut offenders = Vec::new();
    for case in &cases {
        let Err(error) = parse(&case.program) else {
            continue;
        };
        let scanner_class =
            scanner_classes.contains(&error.code) || (error.code, error.sub) == (99, 943);
        if !scanner_class {
            continue;
        }
        if parse(&case.program[..error.byte]).is_err() {
            offenders.push(case.row);
        }
    }
    assert!(
        offenders.is_empty(),
        "rows holding a scan error that masks an earlier parse error: {offenders:?}"
    );
}

// ---- INTERPRET, which the corpus structurally cannot hold ----

#[test]
fn the_interpret_only_errors_match_the_condition_objects_own_code() {
    // `rexxc` compiles a FILE, so there is no way to point it at an `INTERPRET`
    // string. A `signal on syntax` trap around an `interpret` is what exposes
    // these, and `condition('o')~code` is the number. Measured under
    // `build/bin/rexx`, all seven, one file each:
    //
    //     interpret "expose a"       -> code=99.908  errortext=Translation error.
    //     interpret "guard on"       -> code=99.912  errortext=Translation error.
    //     interpret "use local a"    -> code=99.915  errortext=Translation error.
    //     interpret "forward to 1"   -> code=99.923  errortext=Translation error.
    //     interpret "reply 1"        -> code=99.924  errortext=Translation error.
    //     interpret "::routine r"    -> code=99.914  errortext=Translation error.
    //     interpret "x: nop"         -> code=47.1    errortext=Unexpected label.
    //
    // `condition('o')~position` is 2 for every one of them, the line of the
    // `INTERPRET` instruction itself and not a position inside the fragment,
    // which is why no line is asserted here.
    let cases: &[(&str, (u16, u16))] = &[
        ("expose a", (99, 908)),
        ("guard on", (99, 912)),
        ("use local a", (99, 915)),
        ("forward to 1", (99, 923)),
        ("reply 1", (99, 924)),
        ("::routine r", (99, 914)),
        ("x: nop", (47, 1)),
    ];
    let mut wrong = Vec::new();
    for (fragment, expected) in cases {
        match parse_interpret(fragment.as_bytes().to_vec()) {
            Ok(_) => wrong.push(format!("{fragment:?} parsed, expected {expected:?}")),
            Err(e) if (e.code, e.sub) != *expected => {
                wrong.push(format!("{fragment:?}: expected {expected:?}, got {e}"));
            }
            Err(_) => {}
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn an_interpret_error_with_no_substitution_renders_the_condition_objects_message() {
    // 99.908's sub-message takes no substitution, so `message()` returns it, and
    // it is byte-identical to what the trapped program reads out of
    // `condition('o')~message`. Measured:
    //     code=99.908  message=INTERPRET data must not contain EXPOSE.
    let error = parse_interpret(b"expose a".to_vec()).expect_err("EXPOSE is rejected");
    assert_eq!(error.message(), "INTERPRET data must not contain EXPOSE.");
}

#[test]
fn an_interpret_error_with_a_substitution_renders_the_condition_objects_errortext() {
    // 47.1's sub-message substitutes the label, so `message()` falls back to the
    // major -- which is byte-identical to `condition('o')~errortext`, the other
    // field the condition object carries. Measured:
    //     code=47.1  errortext=Unexpected label.
    //                message=INTERPRET data must not contain labels; found "X".
    //                additional=X
    // That `additional` is the value Phase 4 owes and this phase does not carry.
    let error = parse_interpret(b"x: nop".to_vec()).expect_err("a label is rejected");
    assert_eq!(error.message(), "Unexpected label.");
}

#[test]
fn an_interpret_fragment_that_is_legal_is_still_accepted() {
    // The other direction, so the test above cannot pass by rejecting every
    // fragment: `interpret "say 1"` runs, and `expose`/`guard` are rejected for
    // being INTERPRET-specific rather than for being unparseable.
    parse_interpret(b"say 1".to_vec()).expect("a plain SAY interprets");
    parse_interpret(b"a = 1; say a".to_vec()).expect("two clauses interpret");
}

// ---- completeness over the files that are in the tree already ----

/// Every `.rex` file under `samples/`, found with a recursive walk.
///
/// Not a glob: `samples/*.rex` is only the 36 top-level files, and the criterion
/// is over all 301.
fn sample_programs() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples");
    let mut found = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("samples/ is readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rex") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn every_sample_program_parses() {
    let files = sample_programs();
    // 301 files, and every one gets rc 0 from `build/bin/rexxc` (measured), so
    // "parses" is the whole expectation and there is nothing per-file to record.
    assert!(
        files.len() >= 301,
        "expected at least 301 sample programs, found {}",
        files.len()
    );
    let mut failed = Vec::new();
    for path in &files {
        let text = std::fs::read(path).expect("a readable sample");
        if let Err(e) = parse_program(text) {
            failed.push(format!("{}: {e}", path.display()));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn both_bootstrap_files_parse() {
    // The interpreter's own class library, which is the largest real Rexx in the
    // tree and the only input that exercises the directive grammar at scale.
    for name in ["CoreClasses.orx", "StreamClasses.orx"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../interpreter/RexxClasses")
            .join(name);
        let text = std::fs::read(&path).unwrap_or_else(|e| panic!("{name} is readable: {e}"));
        parse_program(text).unwrap_or_else(|e| panic!("{name} failed to parse: {e}"));
    }
}
