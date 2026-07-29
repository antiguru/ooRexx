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
//! anything. **Soundness**: every program the oracle rejects as a translation
//! error, this parser rejects with the same number and sub-number on the same
//! line. **Completeness**: every program the oracle accepts, this parser
//! accepts -- a parser that rejected nothing would pass the first half and fail
//! the second, and one that rejected everything the reverse.
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

use std::path::{Path, PathBuf};

use rexx_parse::{ParseError, ProgramSource, SourceKind, parse_program};

/// One row of the corpus: a program and what the oracle answered for it.
struct Case {
    program: Vec<u8>,
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
        if line.starts_with('#') || line.starts_with("expect\t") {
            continue;
        }
        let mut fields = line.split('\t');
        let expect = fields
            .next()
            .unwrap_or_else(|| panic!("row {row}: no expect"));
        let at = fields
            .next()
            .unwrap_or_else(|| panic!("row {row}: no line"));
        let program = fields
            .next()
            .unwrap_or_else(|| panic!("row {row}: no program"));
        assert!(
            fields.next().is_none(),
            "row {row}: a program field must not contain a tab; \\t is the escape"
        );
        let error = if expect == "ok" {
            assert_eq!(at, "-", "row {row}: an accepted program has no line");
            None
        } else {
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
            program: unescape(program, row),
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

/// The name of every deviation this phase records, so that a case landing in
/// one is visibly a deviation rather than quietly a pass.
///
/// Both are in the plan's exit gate. Neither is a bug to be fixed here: the
/// tests further down pin each in both directions, and any case that stops
/// deviating fails the count assertions.
#[derive(PartialEq, Eq, Debug)]
enum Deviation {
    /// A label clause spelled `then:` or `when:`. Task 3.4 splits the label's
    /// colon off, so nothing is left that can produce the oracle's 35.1 and the
    /// missing-THEN check fires instead. Both reject; the number differs.
    LabelColonInsteadOfBadExpression,
    /// The oracle rejects it, but not with a *translation* error: the directive
    /// translated and then failed to bind to something outside the program.
    /// This parser must accept it, because loading a library is not parsing.
    NotATranslationError,
}

/// Which deviation, if any, explains `(ours, oracle)`.
///
/// A rule rather than a list of programs, so a new program of the same shape is
/// classified instead of failing, while a new *shape* still fails. The counts
/// asserted below are what stops the rules from silently absorbing more than
/// the nine and two cases they were written for.
fn deviation(ours: Option<(u16, u16)>, oracle: Option<(u16, u16)>) -> Option<Deviation> {
    match (ours, oracle) {
        // 98.9xx is `Execution error`, raised when a `::ROUTINE`/`::METHOD`
        // with `EXTERNAL "LIBRARY x"` cannot load `x`; 90.999 is `External name
        // not found`, the same failure for the `REGISTERED` spelling. Both come
        // from installing the directive after translation finished, which is why
        // `rexxc` reports them at all and why they are not this parser's to
        // raise.
        (None, Some((98 | 90, _))) => Some(Deviation::NotATranslationError),
        (Some((18, 1 | 2)), Some((35, 1))) => Some(Deviation::LabelColonInsteadOfBadExpression),
        _ => None,
    }
}

#[test]
fn the_corpus_file_is_shaped_the_way_the_tests_below_assume() {
    let cases = cases();
    // A floor rather than an exact count, because the corpus may grow; a file
    // that silently emptied would make every loop below vacuously pass.
    assert!(
        cases.len() >= 995,
        "expected at least 995 corpus rows, found {}",
        cases.len()
    );
    let errors = cases.iter().filter(|c| c.error.is_some()).count();
    let accepted = cases.len() - errors;
    assert!(
        errors >= 551,
        "expected at least 551 rejected programs, found {errors}"
    );
    assert!(
        accepted >= 444,
        "expected at least 444 accepted programs, found {accepted}"
    );
    // Duplicate rows would inflate every count without adding a case.
    let mut programs: Vec<&[u8]> = cases.iter().map(|c| c.program.as_slice()).collect();
    programs.sort_unstable();
    let before = programs.len();
    programs.dedup();
    assert_eq!(
        before,
        programs.len(),
        "the corpus holds a duplicate program"
    );
    // Every rejected row names a line, and it is a line the program has.
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

#[test]
fn every_program_the_oracle_rejects_this_parser_rejects_with_the_same_number() {
    let cases = cases();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for case in &cases {
        let Some(expected) = case.error else { continue };
        let ours = parse(&case.program).err().map(|e| (e.code, e.sub));
        if deviation(ours, Some(expected)).is_some() {
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
        checked >= 540,
        "only {checked} rejections were checked; the deviation rules are absorbing cases"
    );
}

#[test]
fn every_program_the_oracle_accepts_this_parser_accepts() {
    let cases = cases();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for case in &cases {
        if case.error.is_some() {
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
fn the_reported_line_matches_the_oracles_main_message() {
    let cases = cases();
    let mut wrong = Vec::new();
    // Line 1 is the answer for most of the corpus, so a check that only saw
    // one-line programs would pass without discriminating. Count the rows that
    // report past line 1 and require enough of them.
    let mut past_first_line = 0;
    for case in &cases {
        let Some(expected) = case.error else { continue };
        let Err(error) = parse(&case.program) else {
            continue;
        };
        if (error.code, error.sub) != expected {
            continue;
        }
        let source = ProgramSource::new(case.program.clone(), SourceKind::Program);
        let ours = error.line(&source);
        let Some(theirs) = case.line else { continue };
        if theirs > 1 {
            past_first_line += 1;
        }
        if ours != theirs {
            wrong.push(format!(
                "row {}: error {}.{} reported on line {ours}, oracle says {theirs}",
                case.row, expected.0, expected.1
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(
        past_first_line >= 100,
        "only {past_first_line} corpus errors are reported past line 1, so this \
         check is not discriminating between line conventions"
    );
}

#[test]
fn every_error_the_corpus_raises_renders_a_message_a_user_can_read() {
    let cases = cases();
    let mut checked = 0;
    for case in &cases {
        if case.error.is_none() {
            continue;
        }
        let Err(error) = parse(&case.program) else {
            continue;
        };
        checked += 1;
        let message = error.message();
        assert!(!message.is_empty(), "row {}: empty message", case.row);
        // The one property the scope decision does not relax: whatever the
        // message says, it must not say `&1`. An unfilled placeholder reaching a
        // user would be worse than the generic text that replaces it. Any
        // position, not just the three the table happens to use today.
        assert!(
            !holds_placeholder(&message),
            "row {}: {}.{} renders {message:?}",
            case.row,
            error.code,
            error.sub
        );
    }
    assert!(checked >= 542, "only {checked} messages were rendered");
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

// ---- the two recorded deviations, both directions each ----

#[test]
fn a_then_or_when_label_is_a_missing_then_here_and_a_bad_expression_there() {
    // Structural, not a near miss: Task 3.4 splits `then:` into a label, so the
    // clause the oracle finds an invalid expression in does not exist by the
    // time the grammar runs. Both reject, which is the part that matters.
    //
    // Measured, `if 1 = 1` then `then: nop`:
    //     Error 35 running <file> line 2:  Invalid expression.
    //     Error 35.1:  Incorrect expression detected at ":".
    let error = parse(b"if 1 = 1\nthen: nop").expect_err("both reject it");
    assert_eq!((error.code, error.sub), (18, 1));
    // And the WHEN spelling, which the oracle also answers 35.1 for.
    let error = parse(b"select\nwhen 1 = 1\nthen: nop\nend").expect_err("both reject it");
    assert_eq!((error.code, error.sub), (18, 2));
    // The other direction of the rule: 18.1 against any other oracle number is
    // NOT this deviation and must not be waved through.
    assert_eq!(
        deviation(Some((18, 1)), Some((35, 1))),
        Some(Deviation::LabelColonInsteadOfBadExpression)
    );
    assert_eq!(deviation(Some((18, 1)), Some((18, 1))), None);
    assert_eq!(deviation(Some((18, 1)), Some((35, 901))), None);
    assert_eq!(deviation(Some((18, 3)), Some((35, 1))), None);
}

#[test]
fn a_load_failure_is_not_a_parse_error_and_the_program_is_accepted() {
    // `rexxc` rejects all nine of these, and none of the nine is a translation
    // error: the directive parsed, and then the interpreter could not find the
    // library or the registered routine. Measured for the first,
    //     Error 98 running <file> line 1:  Execution error.
    //     Error 98.903:  Unable to load library "x".
    // and for the last,
    //     Error 90 running <file> line 1:  External name not found.
    //     Error 90.999:  Unable to find external routine "R".
    for program in [
        &b"::routine r external \"LIBRARY x\"\n"[..],
        &b"::method m external \"LIBRARY foo\"\n"[..],
        &b"::routine r external \"registered x\"\n"[..],
    ] {
        parse(program).unwrap_or_else(|e| {
            panic!(
                "{:?} is not a translation error but was rejected with {e}",
                String::from_utf8_lossy(program)
            )
        });
    }
    // The rule's other direction: only a 98 or 90 from the oracle excuses our
    // accepting a program, and only when we accepted it.
    assert_eq!(
        deviation(None, Some((98, 903))),
        Some(Deviation::NotATranslationError)
    );
    assert_eq!(
        deviation(None, Some((90, 999))),
        Some(Deviation::NotATranslationError)
    );
    assert_eq!(deviation(None, Some((35, 1))), None);
    assert_eq!(deviation(None, Some((99, 903))), None);
    assert_eq!(deviation(Some((98, 903)), Some((98, 903))), None);
}

#[test]
fn each_deviation_covers_the_number_of_corpus_cases_it_was_written_for() {
    // The rules above are shapes, so without this they could grow to cover a
    // real regression. Exact counts, not floors: a tenth load failure or a third
    // label-colon case has to be looked at and this line re-argued.
    let cases = cases();
    let mut counts = (0, 0);
    for case in &cases {
        let ours = parse(&case.program).err().map(|e| (e.code, e.sub));
        match deviation(ours, case.error) {
            Some(Deviation::NotATranslationError) => counts.0 += 1,
            Some(Deviation::LabelColonInsteadOfBadExpression) => counts.1 += 1,
            None => {}
        }
    }
    assert_eq!(counts, (9, 2), "(not-a-translation-error, label-colon)");
}

// ---- completeness over the files that are in the tree already ----

/// Every `.rex` file under `samples/`, found with a recursive walk.
///
/// Not a glob: `samples/*.rex` is only the 36 top-level files, and the
/// criterion is over all 301.
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
