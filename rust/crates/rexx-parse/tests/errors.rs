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

use std::path::{Path, PathBuf};

use rexx_parse::{
    ParseError, ProgramSource, SourceKind, parse_interpret, parse_program, program_from,
};

/// What kind of answer the oracle gave, as the corpus records it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Class {
    /// `rexxc` refused to translate. This parser must reproduce the number, the
    /// sub-number and the line.
    Translation,
    /// `rexxc` rejected it for something outside the program text, a library or
    /// an external routine it could not bind. This parser must accept it.
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
    /// The oracle's secondary message with its inserts filled, or `None` where
    /// it printed none.
    message: Option<Vec<u8>>,
    /// The `program` field as the corpus file spells it, escapes and all.
    source: String,
    /// 1-based row number in the corpus file, for a failure message that can be
    /// looked up.
    row: usize,
}

fn corpus_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/errors/parse-errors.tsv")
}

/// Reverses the corpus file's escaping: `\\`, `\t`, `\n`, `\r` and `\xNN`.
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
        let mut fields = line.splitn(5, '\t');
        let mut field = |what: &str| {
            fields
                .next()
                .unwrap_or_else(|| panic!("row {row}: no {what} field"))
                .to_string()
        };
        let class = field("class");
        let expect = field("expect");
        let at = field("line");
        let message = field("message");
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
            message: (message != "-").then(|| unescape(&message, row)),
            source: program,
            row,
        });
    }
    cases
}

/// Parses on a 512 MiB thread, as `tests/deep.rs` does, so that the nesting
/// row reaches `MAX_EXPR_DEPTH` before the native stack ends.
fn parse(program: &[u8]) -> Result<(), ParseError> {
    let program = program.to_vec();
    std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || parse_program(program).map(|_| ()))
        .expect("spawns")
        .join()
        .expect("the parse does not panic")
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
    // Exact, and the only count here that is: every install-time row is
    // hand-classified, so a new one has to be argued, not absorbed.
    assert_eq!(install, 12, "the install-time classification grew");
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
        let ours = parse(&case.program).err().map(|e| (e.code(), e.sub()));
        if label_colon_deviation(ours, expected) || do_subkey_deviation(ours, expected) {
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
        if (error.code(), error.sub()) != expected {
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
fn a_rejection_that_is_not_a_translation_error_is_accepted_or_bound_first() {
    // None of these is a translation error: the failure is that a library or a
    // registered routine could not be bound, which depends on the machine and
    // never on the program text. Both codes are raised from inside the
    // oracle's parser, and `rexxc`, which never installs anything, reports
    // them. So the parser accepts the row, or rejects it having recorded the
    // directive the oracle names as one it resolves before the rejection. The
    // classification is the corpus's, per row, so this loop reads it rather
    // than inferring it from 98 or 90.
    let cases = cases();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for case in &cases {
        if case.class != Class::Install {
            continue;
        }
        checked += 1;
        let source = ProgramSource::new(case.program.clone(), SourceKind::Program);
        let Err(rejected) = program_from(source) else {
            continue;
        };
        let bound_line = rejected
            .binds
            .first()
            .map(|directive| rejected.source.line_of(directive.clause_span.start));
        if bound_line != case.line {
            wrong.push(format!(
                "row {}: {:?} was rejected with {} and binds line {bound_line:?} first",
                case.row,
                String::from_utf8_lossy(&case.program),
                rejected.error
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(
        checked, 12,
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

/// The translation rows whose rendered message is not the oracle's, keyed by
/// their `program` field, each with what it renders. The recorded deviations
/// are left out, and their own tests pin what they render.
fn message_mismatches() -> std::collections::BTreeMap<String, String> {
    let mut wrong = std::collections::BTreeMap::new();
    for case in cases() {
        if case.class != Class::Translation {
            continue;
        }
        let want = case
            .message
            .as_deref()
            .unwrap_or_else(|| panic!("row {}: a translation row needs a message", case.row));
        let pair = case.error.expect("a translation row has a number");
        let got = match parse(&case.program) {
            Ok(()) => b"<accepted>".to_vec(),
            Err(error) if do_subkey_deviation(Some((error.code(), error.sub())), pair) => continue,
            Err(error) if label_colon_deviation(Some((error.code(), error.sub())), pair) => {
                continue;
            }
            Err(error) => error.message(),
        };
        if got != want {
            wrong.entry(case.source.clone()).or_insert_with(|| {
                format!(
                    "row {}: {}.{} renders {:?}, the oracle {:?}",
                    case.row,
                    pair.0,
                    pair.1,
                    String::from_utf8_lossy(&got),
                    String::from_utf8_lossy(want)
                )
            });
        }
    }
    wrong
}

#[test]
fn every_secondary_message_matches_the_oracle() {
    let wrong = message_mismatches();
    assert!(
        wrong.is_empty(),
        "{}",
        wrong
            .values()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---- the recorded deviations, each pinned in both directions ----

/// Whether `(ours, oracle)` is the label-colon deviation, Deviation 39 in
/// `phase-4-exclusions.txt`.
fn label_colon_deviation(ours: Option<(u16, u16)>, oracle: (u16, u16)) -> bool {
    matches!((ours, oracle), (Some((18, 1 | 2)), (35, 1)))
}

#[test]
fn a_then_label_is_a_missing_then_here_and_a_bad_expression_there() {
    // Structural, not a near miss: Task 3.4 splits `then:` into a label, so the
    // clause the oracle finds an invalid expression in does not exist by the time
    // the grammar runs. Both reject, which is the part that matters.
    let error = parse(b"if 1 = 1\nthen: nop").expect_err("both reject it");
    assert_eq!((error.code(), error.sub()), (18, 1));
}

#[test]
fn a_when_label_is_a_missing_then_here_and_a_bad_expression_there() {
    // The WHEN spelling, which the oracle also answers 35.1 for, measured the
    // same way against `select` / `when 1 = 1` / `then: nop` / `end`.
    let error = parse(b"select\nwhen 1 = 1\nthen: nop\nend").expect_err("both reject it");
    assert_eq!((error.code(), error.sub()), (18, 2));
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
    // Deviation 39; see `label_colon_deviation`.
    let mut covered = Vec::new();
    for case in cases() {
        if case.class != Class::Translation {
            continue;
        }
        let oracle = case.error.expect("a translation error has a number");
        let Err(error) = parse(&case.program) else {
            continue;
        };
        if !label_colon_deviation(Some((error.code(), error.sub())), oracle) {
            continue;
        }
        // Ours names the IF or WHEN, the oracle the leftover colon.
        assert_ne!(
            Some(error.message()),
            case.message,
            "row {}: the oracle now agrees",
            case.row
        );
        assert_eq!(
            case.message.as_deref(),
            Some(&br#"Incorrect expression detected at ":"."#[..]),
            "row {}",
            case.row
        );
        covered.push((case.source, error.message()));
    }
    assert_eq!(
        covered,
        [
            (
                r#"select\nwhen 1 = 1\nthen: nop\nend"#.to_string(),
                b"WHEN instruction on line 2 requires matching THEN clause.".to_vec()
            ),
            (
                r#"if 1 = 1\nthen: nop"#.to_string(),
                b"IF instruction on line 1 requires matching THEN clause.".to_vec()
            ),
        ],
        "the label-colon deviation changed"
    );
}

/// Whether `(ours, oracle)` is Deviation 38: a `TO` or `BY` after a `DO OVER`
/// or `DO WITH` header's FOR count.
fn do_subkey_deviation(ours: Option<(u16, u16)>, oracle: (u16, u16)) -> bool {
    matches!((ours, oracle), (Some((25, 927)), (49, 2)))
}

#[test]
fn the_do_subkey_rule_fires_for_the_pair_it_was_written_for() {
    assert!(do_subkey_deviation(Some((25, 927)), (49, 2)));
}

#[test]
fn the_do_subkey_rule_does_not_fire_for_a_near_miss() {
    assert!(!do_subkey_deviation(Some((49, 2)), (49, 2)));
    assert!(!do_subkey_deviation(Some((25, 927)), (25, 927)));
    assert!(!do_subkey_deviation(Some((27, 901)), (49, 2)));
    assert!(!do_subkey_deviation(Some((25, 927)), (49, 1)));
    assert!(!do_subkey_deviation(None, (49, 2)));
}

#[test]
fn the_do_subkey_rows_differ_from_the_oracle_as_deviation_38_says() {
    let mut covered = Vec::new();
    for case in cases() {
        if case.class != Class::Translation {
            continue;
        }
        let oracle = case.error.expect("a translation error has a number");
        let Err(error) = parse(&case.program) else {
            continue;
        };
        if !do_subkey_deviation(Some((error.code(), error.sub())), oracle) {
            continue;
        }
        let want = if case.source.contains(" by ") {
            &b"Unknown keyword following \"FOR\"; found \"BY\"."[..]
        } else {
            &b"Unknown keyword following \"FOR\"; found \"TO\"."[..]
        };
        assert_eq!(
            String::from_utf8_lossy(&error.message()),
            String::from_utf8_lossy(want),
            "row {}",
            case.row
        );
        assert_ne!(
            Some(error.message()),
            case.message,
            "row {}: the oracle now agrees",
            case.row
        );
        let source = ProgramSource::new(case.program.clone(), SourceKind::Program);
        assert_eq!(Some(error.line(&source)), case.line, "row {}", case.row);
        covered.push(case.source);
    }
    assert_eq!(
        covered,
        [
            r#"do i over x for 1 to 2\nend"#,
            r#"do i over x for 1 by 2\nend\n"#,
            r#"do with index i over x for 1 to 2\nend\n"#,
        ]
    );
}

#[test]
fn a_scan_error_waits_for_the_parse_to_reach_its_clause() {
    // The oracle scans one clause at a time (`LanguageParser::nextClause`), so
    // the earlier parse error wins: measured, 37.2 on line 1.
    let program = b"say )\n\n'unclosed\n";
    let source = ProgramSource::new(program.to_vec(), SourceKind::Program);
    let error = parse_program(program.to_vec()).expect_err("both reject it");
    assert_eq!((error.code(), error.sub()), (37, 2));
    assert_eq!(error.line(&source), 1);
}

#[test]
fn a_scan_error_alone_is_reported_against_its_own_clause() {
    let error = parse_program(b"say )\n".to_vec()).expect_err("a stray paren is an error");
    assert_eq!((error.code(), error.sub()), (37, 2));
    let program = b"nop\n\n'unclosed\n";
    let source = ProgramSource::new(program.to_vec(), SourceKind::Program);
    let error = parse_program(program.to_vec()).expect_err("an open literal is an error");
    assert_eq!((error.code(), error.sub()), (6, 2));
    assert_eq!(error.line(&source), 3);
}

// ---- INTERPRET, which the corpus structurally cannot hold ----

/// The errors only `INTERPRET` text raises, each with a fragment that raises
/// it. `rexxc` compiles a FILE, so there is no way to point it at an
/// `INTERPRET` string. A `signal on syntax` trap around an `interpret` is what
/// exposes these, and `condition('o')~code` is the number. Measured under
/// `build/bin/rexx`, one file each.
const INTERPRET_ONLY: &[(&str, (u16, u16))] = &[
    ("expose a", (99, 908)),
    ("guard on", (99, 912)),
    ("use local a", (99, 915)),
    ("forward to 1", (99, 923)),
    ("reply 1", (99, 924)),
    ("::routine r", (99, 914)),
    ("x: nop", (47, 1)),
];

#[test]
fn the_interpret_only_errors_match_the_condition_objects_own_code() {
    let mut wrong = Vec::new();
    for (fragment, expected) in INTERPRET_ONLY {
        match parse_interpret(fragment.as_bytes().to_vec()) {
            Ok(_) => wrong.push(format!("{fragment:?} parsed, expected {expected:?}")),
            Err(e) if (e.code(), e.sub()) != *expected => {
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
    assert_eq!(error.message(), b"INTERPRET data must not contain EXPOSE.");
}

#[test]
fn an_interpret_label_error_names_the_label() {
    // 47.1's sub-message substitutes the label. Measured on the oracle:
    //     code=47.1  message=INTERPRET data must not contain labels; found "X".
    let error = parse_interpret(b"x: nop".to_vec()).expect_err("a label is rejected");
    assert_eq!(
        error.message(),
        br#"INTERPRET data must not contain labels; found "X"."#
    );
}

#[test]
fn an_interpret_fragment_that_is_legal_is_still_accepted() {
    // The other direction, so the test above cannot pass by rejecting every
    // fragment: `interpret "say 1"` runs, and `expose`/`guard` are rejected for
    // being INTERPRET-specific rather than for being unparseable.
    parse_interpret(b"say 1".to_vec()).expect("a plain SAY interprets");
    parse_interpret(b"a = 1; say a".to_vec()).expect("two clauses interpret");
}

// ---- every message the parser raises, reached ----

/// Why no corpus row reaches a message the parser names.
#[derive(Debug)]
enum Unreached {
    /// Only `INTERPRET` text raises it, so it is in [`INTERPRET_ONLY`].
    InterpretOnly,
    /// No program reaches it. `oracle_site` is the oracle's raise site as
    /// `path:line` under `interpreter/`, and `probes` the programs tried.
    NoProgram {
        oracle_site: &'static str,
        probes: &'static [&'static [u8]],
    },
}

/// The messages the parser names that no corpus row raises, with why.
const UNREACHED_BY_FILE: &[((u16, u16), Unreached)] = &[
    ((47, 1), Unreached::InterpretOnly),
    ((99, 908), Unreached::InterpretOnly),
    ((99, 912), Unreached::InterpretOnly),
    ((99, 914), Unreached::InterpretOnly),
    ((99, 915), Unreached::InterpretOnly),
    ((99, 923), Unreached::InterpretOnly),
    ((99, 924), Unreached::InterpretOnly),
    (
        (27, 903),
        Unreached::NoProgram {
            oracle_site: "parser/InstructionParser.cpp:1650",
            probes: &[
                b"do with\nend\n",
                b"do with index\nend\n",
                b"do with item i\nend\n",
                b"do with over x\nend\n",
                b"do with x over y\nend\n",
            ],
        },
    ),
    (
        (37, 1),
        Unreached::NoProgram {
            oracle_site: "parser/LanguageParser.cpp:3882",
            probes: &[
                b"x = ,\n",
                b"x = 1 + ,\n",
                b"x = (1,)\n",
                b"x = (,1)\n",
                b"say f(1,,2)\n",
                b"x = a[,]\n",
            ],
        },
    ),
    (
        (49, 2),
        Unreached::NoProgram {
            oracle_site: "parser/InstructionParser.cpp:453",
            probes: &[
                b"do i = 1 to 2 by 1 to 3\nend\n",
                b"do i = 1 for 2 while 1 until 2\nend\n",
                b"do forever while 1 until 2\nend\n",
                b"loop 3 while 1 until 2\nend\n",
                b"then nop\n",
                b"else nop\n",
                b"otherwise nop\n",
                b"when 1 then nop\n",
                b"end\n",
            ],
        },
    ),
    (
        (89, 2),
        Unreached::NoProgram {
            oracle_site: "parser/InstructionParser.cpp:3421",
            probes: &[
                b"parse arg .a\n",
                b"parse arg 5\n",
                b"parse var x .a\n",
                b"parse value 1 with .\n",
                b"parse arg a.b. c\n",
            ],
        },
    ),
];

/// Every `E{major}_{sub}` constant the parser's own source names, outside its
/// tests, with the first file naming it.
fn messages_named_by_the_parser() -> std::collections::BTreeMap<(u16, u16), PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    let mut test_modules = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src/ is readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    // A `#[cfg(test)] mod name;` makes `name.rs` beside its parent's stem a
    // test file, whatever it is called.
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a readable source file");
        let lines: Vec<&str> = text.lines().collect();
        for pair in lines.windows(2) {
            if pair[0].trim() == "#[cfg(test)]"
                && let Some(name) = pair[1]
                    .trim()
                    .strip_prefix("mod ")
                    .and_then(|rest| rest.strip_suffix(';'))
            {
                test_modules.push(path.with_extension("").join(format!("{name}.rs")));
            }
        }
    }
    let mut named = std::collections::BTreeMap::new();
    for path in &files {
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs")
            || test_modules.contains(path)
        {
            continue;
        }
        let text = std::fs::read_to_string(path).expect("a readable source file");
        let mut skip_until: Option<String> = None;
        let mut after_cfg_test = false;
        for line in text.lines() {
            if let Some(close) = &skip_until {
                if line == close {
                    skip_until = None;
                }
                continue;
            }
            // An inline `#[cfg(test)] mod name {` runs to the `}` at its own
            // indentation.
            if after_cfg_test && line.trim_start().starts_with("mod ") && line.ends_with('{') {
                let indent = line.len() - line.trim_start().len();
                skip_until = Some(format!("{}}}", " ".repeat(indent)));
                after_cfg_test = false;
                continue;
            }
            after_cfg_test = line.trim() == "#[cfg(test)]";
            let code = line.split("//").next().unwrap_or("");
            let bytes = code.as_bytes();
            for (at, _) in code.match_indices('E') {
                if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_') {
                    continue;
                }
                let rest = &code[at + 1..];
                let major_len = rest.bytes().take_while(u8::is_ascii_digit).count();
                if major_len == 0 || rest.as_bytes().get(major_len) != Some(&b'_') {
                    continue;
                }
                let tail = &rest[major_len + 1..];
                let sub_len = tail.bytes().take_while(u8::is_ascii_digit).count();
                let follows = tail.as_bytes().get(sub_len);
                if sub_len == 0 || follows.is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                {
                    continue;
                }
                let pair = (
                    rest[..major_len].parse().expect("a major number"),
                    tail[..sub_len].parse().expect("a sub-number"),
                );
                named.entry(pair).or_insert_with(|| path.clone());
            }
        }
    }
    named
}

#[test]
fn the_scan_finds_the_messages_it_was_written_against() {
    let named = messages_named_by_the_parser();
    for pair in [(13, 1), (35, 1), (14, 3), (25, 901), (99, 914), (36, 902)] {
        assert!(named.contains_key(&pair), "{pair:?} not found");
    }
    assert!(
        !named.contains_key(&(99, 943)) || named[&(99, 943)].ends_with("directive.rs"),
        "a constant is attributed to a test file"
    );
}

#[test]
fn every_message_the_parser_names_is_raised_by_a_corpus_row_or_recorded_as_unreached() {
    let named = messages_named_by_the_parser();
    let mut raised = std::collections::BTreeSet::new();
    for case in cases() {
        if case.class != Class::Translation {
            continue;
        }
        if let Err(error) = parse(&case.program)
            && Some((error.code(), error.sub())) == case.error
        {
            raised.insert((error.code(), error.sub()));
        }
    }
    let mut missing = Vec::new();
    for (pair, path) in &named {
        let recorded = UNREACHED_BY_FILE.iter().any(|(p, _)| p == pair);
        if raised.contains(pair) {
            assert!(
                !recorded,
                "{pair:?} is raised by a corpus row and also recorded"
            );
        } else if !recorded {
            missing.push(format!("{}.{} ({})", pair.0, pair.1, path.display()));
        }
    }
    assert!(
        missing.is_empty(),
        "named by the parser, raised by no corpus row, and not recorded:\n{}",
        missing.join("\n")
    );
    for (pair, _) in UNREACHED_BY_FILE {
        assert!(
            named.contains_key(pair),
            "{pair:?} is recorded but the parser no longer names it"
        );
    }
}

/// The oracle's symbolic name for `pair`, from `RexxErrorCodes.h`.
fn oracle_name(pair: (u16, u16)) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../interpreter/messages/RexxErrorCodes.h");
    let text = std::fs::read_to_string(&path).expect("RexxErrorCodes.h is readable");
    let number = format!("= {},", u32::from(pair.0) * 1000 + u32::from(pair.1));
    text.lines()
        .find(|line| line.trim_start().starts_with("Error_") && line.contains(&number))
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or_else(|| panic!("{pair:?} has no symbolic name"))
        .to_string()
}

#[test]
fn every_unreached_reason_still_holds() {
    let interpreter = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../interpreter");
    let mut seen = std::collections::BTreeSet::new();
    for (pair, reason) in UNREACHED_BY_FILE {
        assert!(seen.insert(*pair), "{pair:?} is recorded twice");
        match reason {
            Unreached::InterpretOnly => assert!(
                INTERPRET_ONLY.iter().any(|(_, p)| p == pair),
                "{pair:?} is recorded as INTERPRET-only but INTERPRET_ONLY does not hold it"
            ),
            Unreached::NoProgram {
                oracle_site,
                probes,
            } => {
                let (file, line) = oracle_site
                    .rsplit_once(':')
                    .unwrap_or_else(|| panic!("{pair:?}: {oracle_site:?} is not path:line"));
                let line: usize = line
                    .parse()
                    .unwrap_or_else(|e| panic!("{pair:?}: {oracle_site:?}: {e}"));
                let text = std::fs::read_to_string(interpreter.join(file))
                    .unwrap_or_else(|e| panic!("{pair:?}: {file}: {e}"));
                let cited = text
                    .lines()
                    .nth(line - 1)
                    .unwrap_or_else(|| panic!("{pair:?}: {file} has no line {line}"));
                let name = oracle_name(*pair);
                assert!(
                    cited.contains(&name),
                    "{pair:?}: {oracle_site} does not raise {name}: {cited:?}"
                );
                assert!(!probes.is_empty(), "{pair:?}: no probe was recorded");
                for probe in *probes {
                    if let Err(error) = parse(probe) {
                        assert_ne!(
                            (error.code(), error.sub()),
                            *pair,
                            "{pair:?} is reached by {:?}",
                            String::from_utf8_lossy(probe)
                        );
                    }
                }
            }
        }
    }
}

// ---- completeness over the files that are in the tree already ----

/// Every `.rex` file under `samples/`, found with a recursive walk.
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
