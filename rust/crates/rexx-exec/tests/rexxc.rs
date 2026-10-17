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

//! `rexxc` against the oracle's, and against `corpus/errors/parse-errors.tsv`.
//!
//! Each case of `tests/rexxc/cases` runs both binaries with the same words
//! from a directory of the same programs, and the three descriptors must
//! agree. The recorded block is the oracle's, normalised. A case's input is
//! its words, one per line: `\xHH` is a byte. `out=NAME` reports the first
//! bytes of the file [`output_name`] spells after the run, the part of an
//! image both sides write alike, and removes it.

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rayon::prelude::*;

const CASE_DIR: &str = "tests/rexxc";

/// The oracle's address-space cap, in KiB.
const ORACLE_MEMORY_LIMIT_KIB: u64 = 1_048_576;

const ORACLE_ROOT: &str = "/home/moritz/dev/repos/ooRexx/build";

/// How much of an output file a case reports: the `#!` line and the binary
/// tag with two of its pad bytes, or the encoded tag.
const IMAGE_PREFIX: usize = 31;

/// The programs every case runs among.
fn lay_fixture(dir: &Path) {
    let file = |name: &[u8], text: &str| {
        let path = dir.join(PathBuf::from(OsString::from_vec(name.to_vec())));
        std::fs::write(path, text).expect("a fixture file");
    };
    file(b"ok.rex", "say 1\n");
    file(b"bad.rex", "say (\n");
    file(b"a b.rex", "say 1\n");
    file(b"c\xffd.rex", "say 2\n");
    file(b"e f.rex", "say (\n");
    file(
        b"lib.rex",
        "say 'prolog'\n::routine r external \"LIBRARY x\"\n",
    );
    file(b"cls.rex", "::class a subclass zzz\n");
    file(b"req.rex", "::requires 'nosuch.rex'\n");
    file(
        b"rexxent.rex",
        "::method m external \"LIBRARY REXX nosuchentry\"\n",
    );
    file(b"ro.rxo", "");
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(dir.join("ro.rxo"), std::fs::Permissions::from_mode(0o444))
        .expect("the read-only output");
}

/// The output file a case's `out` argument names: one of the names
/// `datadriven` cannot carry in an argument, or the name itself.
fn output_name(name: &str) -> PathBuf {
    let bytes: &[u8] = match name {
        "blank_rxo" => b"o p.rxo",
        "ff_rxo" => b"q\xffr.rxo",
        other => other.as_bytes(),
    };
    PathBuf::from(OsString::from_vec(bytes.to_vec()))
}

/// `line` with each `\xHH` as its byte.
fn unescape(line: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut rest = line.as_bytes();
    while let Some((&byte, tail)) = rest.split_first() {
        if let Some(hex) = rest.strip_prefix(b"\\x").filter(|hex| hex.len() >= 2) {
            let digits = std::str::from_utf8(&hex[..2]).expect("hex digits");
            bytes.push(u8::from_str_radix(digits, 16).expect("a hex byte"));
            rest = &hex[2..];
        } else {
            bytes.push(byte);
            rest = tail;
        }
    }
    bytes
}

/// `bytes` with every byte outside printable ASCII as `\xHH`.
fn escaped(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &byte in bytes {
        if byte == b'\n' || (0x20..0x7f).contains(&byte) && byte != b'\\' {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("\\x{byte:02X}"));
        }
    }
    out
}

/// The three descriptors as the case file records them, with the fixture's
/// directory as `DIR` and the banner's compile-time stamps masked.
fn render(dir: &Path, stdout: &[u8], stderr: &[u8], status: Option<i32>) -> String {
    let dir = escaped(dir.as_os_str().as_encoded_bytes());
    let mut rendered = match status {
        Some(code) => format!("rc> {code}\n"),
        None => "rc> none\n".to_string(),
    };
    for (tag, bytes) in [("out", stdout), ("err", stderr)] {
        for line in escaped(bytes).replace(&dir, "DIR").lines() {
            let line = match line.strip_prefix("Build date: ") {
                Some(_) => "Build date: <date>".to_string(),
                None => match line.strip_prefix("Copyright (c) 2005-") {
                    Some(rest) => {
                        format!("Copyright (c) 2005-<year>{}", &rest[4.min(rest.len())..])
                    }
                    None => line.to_string(),
                },
            };
            rendered.push_str(&format!("{tag}> {line}\n"));
        }
    }
    rendered
}

/// A command running `program` from `dir` under `timeout`, the oracle's
/// under its address-space cap.
fn command(program: &Path, dir: &Path, oracle: bool) -> Command {
    let mut command = Command::new("timeout");
    command.args(["-k", "5", "20"]);
    if oracle {
        command
            .arg("sh")
            .arg("-c")
            .arg(format!(
                "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" \"$@\""
            ))
            .env("LD_LIBRARY_PATH", Path::new(ORACLE_ROOT).join("lib"));
    }
    command
        .arg(program)
        .current_dir(dir)
        .env_remove("REXX_PATH")
        .stdin(Stdio::null());
    command
}

/// Runs `program` with the case's words from `dir`, and reports and removes
/// the output file the case names.
fn run(program: &Path, dir: &Path, case: &datadriven::TestCase, oracle: bool) -> String {
    let args: Vec<OsString> = case
        .input
        .lines()
        .map(|line| OsString::from_vec(unescape(line)))
        .collect();
    let output = command(program, dir, oracle)
        .args(&args)
        .output()
        .expect("the binary starts");
    let mut rendered = render(dir, &output.stdout, &output.stderr, output.status.code());
    for (key, values) in &case.args {
        assert_eq!(key, "out", "unknown argument {key}");
        let name = values.first().expect("a file name");
        let file = dir.join(output_name(name));
        match std::fs::read(&file) {
            Ok(bytes) => {
                let prefix = &bytes[..bytes.len().min(IMAGE_PREFIX)];
                rendered.push_str(&format!("file> {}\n", escaped(prefix).replace('\n', "\\n")));
                if name != "ro.rxo" {
                    std::fs::remove_file(&file).expect("the output is removed");
                }
            }
            Err(_) => rendered.push_str("file> absent\n"),
        }
    }
    rendered
}

/// A fresh directory below the target's temporary directory.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("rexxc-{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("an old scratch directory");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory");
    dir
}

/// A fresh fixture directory for one side.
fn fixture(side: &str) -> PathBuf {
    let dir = scratch(side);
    lay_fixture(&dir);
    dir
}

fn oracle_bin(name: &str) -> PathBuf {
    let binary = Path::new(ORACLE_ROOT).join("bin").join(name);
    assert!(
        binary.is_file(),
        "the oracle is missing at {}",
        binary.display()
    );
    binary
}

#[test]
fn the_command_line_answers_as_the_oracles_does() {
    let oracle = oracle_bin("rexxc");
    let ours = Path::new(env!("CARGO_BIN_EXE_rexxc"));
    let (theirs_dir, ours_dir) = (fixture("oracle"), fixture("crate"));
    let mut cases = 0usize;
    let mut diverged = Vec::new();
    datadriven::walk(CASE_DIR, |file| {
        file.run(|case| {
            assert_eq!(
                case.directive, "run",
                "unknown directive {:?}",
                case.directive
            );
            cases += 1;
            let theirs = run(&oracle, &theirs_dir, case, true);
            let crate_side = run(ours, &ours_dir, case, false);
            if crate_side != theirs {
                diverged.push(format!(
                    "{}:\n--- oracle\n{theirs}--- crate\n{crate_side}",
                    case.input.replace('\n', " ")
                ));
            }
            theirs
        });
    });
    std::fs::remove_dir_all(&theirs_dir).expect("the oracle fixture is removed");
    std::fs::remove_dir_all(&ours_dir).expect("the crate fixture is removed");
    assert!(cases > 0, "{CASE_DIR} produced no cases");
    assert!(diverged.is_empty(), "{}", diverged.join("\n"));
}

/// One row of `parse-errors.tsv`.
struct Row {
    line: usize,
    class: String,
    expect: String,
    main_line: String,
    message: Vec<u8>,
    program: Vec<u8>,
}

/// A field with the file's escapes decoded: `\\`, `\t`, `\n`, `\r`, `\xNN`.
fn decode(field: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut bytes = field.bytes();
    while let Some(byte) = bytes.next() {
        if byte != b'\\' {
            out.push(byte);
            continue;
        }
        match bytes.next() {
            Some(b'\\') => out.push(b'\\'),
            Some(b't') => out.push(b'\t'),
            Some(b'n') => out.push(b'\n'),
            Some(b'r') => out.push(b'\r'),
            Some(b'x') => {
                let digits: Vec<u8> = bytes.by_ref().take(2).collect();
                let digits = std::str::from_utf8(&digits).expect("hex digits");
                out.push(u8::from_str_radix(digits, 16).expect("a hex byte"));
            }
            other => panic!("unknown escape {other:?} in {field:?}"),
        }
    }
    out
}

fn rows() -> Vec<Row> {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../corpus/errors/parse-errors.tsv"
    ))
    .expect("parse-errors.tsv");
    let mut rows = Vec::new();
    let mut header = true;
    for (index, line) in text.lines().enumerate() {
        if line.starts_with('#') {
            continue;
        }
        if header {
            header = false;
            continue;
        }
        let fields: Vec<&str> = line.splitn(5, '\t').collect();
        let [class, expect, main_line, message, program] = fields[..] else {
            panic!("line {}: five fields", index + 1);
        };
        rows.push(Row {
            line: index + 1,
            class: class.to_string(),
            expect: expect.to_string(),
            main_line: main_line.to_string(),
            message: if message == "-" {
                Vec::new()
            } else {
                decode(message)
            },
            program: decode(program),
        });
    }
    rows
}

/// Whether `program` names `REGISTERED`, in any case: this crate refuses
/// `EXTERNAL "REGISTERED"` with the `Rxapi` reason where the oracle's parser
/// answers through rxapi.
fn names_registered(program: &[u8]) -> bool {
    program
        .windows(b"REGISTERED".len())
        .any(|window| window.eq_ignore_ascii_case(b"REGISTERED"))
}

/// Whether `row` is exempt from the agreement: an `install` row whose
/// program names `REGISTERED`, which the oracle's parser resolves through
/// rxapi. A translation row naming it is checked like any other.
fn exempt(row: &Row) -> bool {
    row.class == "install" && names_registered(&row.program)
}

/// Runs this crate's `rexxc FILE -s` on `program`, from a fresh directory
/// below `root`.
fn compile(root: &Path, name: &str, program: &[u8]) -> std::process::Output {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).expect("a row directory");
    std::fs::write(dir.join("p.rex"), program).expect("the program");
    command(Path::new(env!("CARGO_BIN_EXE_rexxc")), &dir, false)
        .args(["p.rex", "-s"])
        .output()
        .expect("rexxc starts")
}

/// How this crate's `rexxc` answer for a row stands against the recorded one.
enum Answer {
    Agrees,
    /// A licensed Deviation: this crate's error number and the oracle's.
    Deviation(String, String),
    Disagrees(String),
}

/// The `major.sub` of the `Error major.sub:` line in `stderr`.
fn secondary_code(stderr: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stderr).lines().find_map(|line| {
        let code = line.strip_prefix("Error ")?.split_once(':')?.0;
        (code.contains('.')
            && code
                .bytes()
                .all(|byte| byte == b'.' || byte.is_ascii_digit()))
        .then(|| code.to_string())
    })
}

/// Whether this crate answering `ours` where the oracle answers `oracle` is
/// a licensed Deviation: 38, a `TO` or `BY` after a `DO OVER` or `DO WITH`
/// header's `FOR` count, and 39, a `THEN` followed by a colon
/// (`phase-4-exclusions.txt`, pinned by `rexx-parse/tests/errors.rs`).
fn licensed(ours: &str, oracle: &str) -> bool {
    matches!(
        (ours, oracle),
        ("25.927", "49.2") | ("18.1" | "18.2", "35.1")
    )
}

/// What this crate's `rexxc` answers for `row`, against the recorded answer.
fn answer(root: &Path, row: &Row) -> Answer {
    let output = compile(root, &row.line.to_string(), &row.program);
    let status = output.status.code();
    let agrees = if row.expect == "ok" {
        status == Some(0) && output.stderr.is_empty()
    } else {
        let (major, _) = row.expect.split_once('.').expect("major.sub");
        let major: i32 = major.parse().expect("a major number");
        let secondary = [format!("Error {}:  ", row.expect).as_bytes(), &row.message].concat();
        let main = format!(" line {}:  ", row.main_line);
        let stderr = &output.stderr;
        status == Some(256 - major)
            && stderr
                .split(|&byte| byte == b'\n')
                .any(|line| line == secondary.as_slice())
            && stderr
                .windows(main.len())
                .any(|window| window == main.as_bytes())
    };
    if agrees {
        return Answer::Agrees;
    }
    if let Some(ours) = secondary_code(&output.stderr)
        && licensed(&ours, &row.expect)
    {
        return Answer::Deviation(ours, row.expect.clone());
    }
    Answer::Disagrees(format!(
        "line {}: expected {} at {}, got rc {status:?}, stderr {:?}",
        row.line,
        row.expect,
        row.main_line,
        String::from_utf8_lossy(&output.stderr)
    ))
}

#[test]
fn rexxc_agrees_with_every_recorded_parse_error_row() {
    let rows = rows();
    let (exempted, checked): (Vec<&Row>, Vec<&Row>) = rows.iter().partition(|row| exempt(row));
    assert!(!exempted.is_empty(), "the exempt rows are found");
    assert!(
        checked.iter().any(|row| names_registered(&row.program)),
        "a translation row naming REGISTERED is checked"
    );
    // An exempt row is one this crate refuses with the `Rxapi` reason, so an
    // exemption wider than that refusal hides a row that would agree.
    let exempt_root = scratch("tsv-exempt");
    let unrefused: Vec<usize> = exempted
        .par_iter()
        .filter(|row| {
            let output = compile(&exempt_root, &row.line.to_string(), &row.program);
            !String::from_utf8_lossy(&output.stderr).contains("(Phase 10 Rxapi)")
        })
        .map(|row| row.line)
        .collect();
    std::fs::remove_dir_all(&exempt_root).expect("the exempt row directories are removed");
    assert!(
        unrefused.is_empty(),
        "exempt rows not refused with the Rxapi reason, at lines {unrefused:?}"
    );
    assert!(
        checked
            .iter()
            .any(|row| row.message.starts_with(b"Unable to load library")),
        "the library-binding rows are checked"
    );
    let root = scratch("tsv");
    let answers: Vec<(&Row, Answer)> = checked
        .par_iter()
        .map(|row| (*row, answer(&root, row)))
        .collect();
    std::fs::remove_dir_all(&root).expect("the row directories are removed");
    let mut deviations = Vec::new();
    let mut disagreements = Vec::new();
    for (row, answer) in answers {
        match answer {
            Answer::Agrees => {}
            Answer::Deviation(ours, oracle) => {
                deviations.push((
                    String::from_utf8_lossy(&row.program).into_owned(),
                    ours,
                    oracle,
                ));
            }
            Answer::Disagrees(text) => disagreements.push(text),
        }
    }
    deviations.sort();
    let deviation = |program: &str, ours: &str, oracle: &str| {
        (program.to_string(), ours.to_string(), oracle.to_string())
    };
    assert_eq!(
        deviations,
        [
            deviation("do i over x for 1 by 2\nend\n", "25.927", "49.2"),
            deviation("do i over x for 1 to 2\nend", "25.927", "49.2"),
            deviation("do with index i over x for 1 to 2\nend\n", "25.927", "49.2"),
            deviation("if 1 = 1\nthen: nop", "18.1", "35.1"),
            deviation("select\nwhen 1 = 1\nthen: nop\nend", "18.2", "35.1"),
        ],
        "the licensed deviations changed"
    );
    assert!(
        disagreements.is_empty(),
        "{} of {} rows disagree:\n{}",
        disagreements.len(),
        checked.len(),
        disagreements.join("\n")
    );
}

#[test]
fn registered_is_refused_with_the_rxapi_reason() {
    let program = b"::routine r external \"REGISTERED x y\"\n";
    let recorded = rows();
    assert!(
        recorded
            .iter()
            .any(|row| row.program == program && exempt(row)),
        "the witness is a recorded, exempt row"
    );
    let root = scratch("registered");
    let output = compile(&root, "witness", program);
    std::fs::remove_dir_all(&root).expect("the witness directory is removed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(output.status.code(), Some(0), "{stderr}");
    assert!(stderr.contains("(Phase 10 Rxapi)"), "{stderr}");
}

#[test]
fn the_oracle_refuses_this_crates_images_as_another_versions() {
    let dir = scratch("oracle-reads");
    std::fs::write(dir.join("p.rex"), "say 'compiled'\n").expect("the program");
    for (image, flags) in [("b.rxo", &[][..]), ("e.rxo", &["-e"][..])] {
        let written = command(Path::new(env!("CARGO_BIN_EXE_rexxc")), &dir, false)
            .args(["p.rex", image, "-s"])
            .args(flags)
            .output()
            .expect("rexxc starts");
        assert_eq!(written.status.code(), Some(0), "{image}");
        let bytes = std::fs::read(dir.join(image)).expect("the image");
        assert!(bytes.starts_with(b"#!/usr/bin/env rexx\n/**/@REXX"));
        let ran = command(&oracle_bin("rexx"), &dir, true)
            .arg(image)
            .output()
            .expect("the oracle starts");
        assert_eq!(
            render(&dir, &ran.stdout, &ran.stderr, ran.status.code()),
            format!(
                "rc> 253\n\
                 err> Error 3:  Failure during initialization.\n\
                 err> Error 3.903:  Program \"DIR/{image}\" cannot be run by this version of \
                 the REXX interpreter.\n"
            ),
            "{image}"
        );
    }
    std::fs::remove_dir_all(&dir).expect("the directory is removed");
}
