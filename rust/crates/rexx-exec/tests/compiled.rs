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

//! Compiled programs, restored by every route the oracle restores one.
//!
//! Each side's `rexxc` compiles `tests/compiled/programs` into that side's
//! own directory, and each side's `rexx` runs the cases of
//! `tests/compiled/cases` there. The three descriptors must agree, and the
//! recorded block is the oracle's. A case's input is its words, one per
//! line. Images this crate refuses are pinned against this crate alone: the
//! oracle crashes on a truncated image.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CASE_DIR: &str = "tests/compiled/cases";

const PROGRAMS: &str = "tests/compiled/programs";

/// An image the oracle's `rexxc` wrote.
const ORACLE_IMAGE: &str = "tests/compiled/oracle.rxo";

/// The oracle's address-space cap, in KiB.
const ORACLE_MEMORY_LIMIT_KIB: u64 = 1_048_576;

const ORACLE_ROOT: &str = "/home/moritz/dev/repos/ooRexx/build";

/// What every image starts with, on either side.
const IMAGE_PREFIX: &[u8] = b"#!/usr/bin/env rexx\n/**/@REXX";

/// One side: its two binaries, and whether they are the oracle's.
struct Side {
    rexx: PathBuf,
    rexxc: PathBuf,
    oracle: bool,
    dir: PathBuf,
}

impl Side {
    fn oracle() -> Side {
        let bin = Path::new(ORACLE_ROOT).join("bin");
        assert!(
            bin.join("rexx").is_file(),
            "the oracle is missing at {}",
            bin.display()
        );
        Side {
            rexx: bin.join("rexx"),
            rexxc: bin.join("rexxc"),
            oracle: true,
            dir: fixture("oracle"),
        }
    }

    /// This crate's side, in a directory of its own named `name`.
    fn ours(name: &str) -> Side {
        Side {
            rexx: PathBuf::from(env!("CARGO_BIN_EXE_rexx")),
            rexxc: PathBuf::from(env!("CARGO_BIN_EXE_rexxc")),
            oracle: false,
            dir: fixture(name),
        }
    }

    /// A command running `program` from this side's directory under
    /// `timeout`, the oracle's under its address-space cap.
    fn command(&self, program: &Path) -> Command {
        let mut command = Command::new("timeout");
        command.args(["-k", "5", "20"]);
        if self.oracle {
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
            .current_dir(&self.dir)
            .env_remove("REXX_PATH")
            .stdin(Stdio::null());
        command
    }

    /// Runs one case on this side, answering its descriptors as the case
    /// file records them.
    fn run(&self, case: &datadriven::TestCase) -> String {
        let words: Vec<&str> = case.input.lines().collect();
        let program = match case.directive.as_str() {
            "compile" | "rexxc" => &self.rexxc,
            "rexx" => &self.rexx,
            other => panic!("unknown directive {other:?}"),
        };
        let mut command = self.command(program);
        command.args(&words);
        for (key, values) in &case.args {
            assert_eq!(key, "stdin", "unknown argument {key}");
            let file = std::fs::File::open(self.dir.join(&values[0])).expect("the input file");
            command.stdin(file);
        }
        let output = command.output().expect("the binary starts");
        let rendered = render(&self.dir, &output.stdout, &output.stderr, output.status.code());
        if case.directive == "compile" {
            assert_eq!(output.status.code(), Some(0), "{words:?}: {rendered}");
            let image = std::fs::read(self.dir.join(words[1])).expect("the image");
            assert!(
                image.starts_with(IMAGE_PREFIX),
                "{words:?} on {}",
                self.rexxc.display()
            );
        }
        rendered
    }
}

/// A fresh directory below the target's temporary directory.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("compiled-{name}-{}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("an old scratch directory");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory");
    dir
}

/// A fresh directory holding the fixture's programs.
fn fixture(side: &str) -> PathBuf {
    let dir = scratch(side);
    copy_tree(Path::new(PROGRAMS), &dir);
    dir
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("the fixture directory") {
        let entry = entry.expect("a fixture entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("its type").is_dir() {
            std::fs::create_dir_all(&target).expect("a fixture directory");
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("a fixture file");
        }
    }
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

/// The three descriptors as the case file records them, with the side's
/// directory as `DIR`.
fn render(dir: &Path, stdout: &[u8], stderr: &[u8], status: Option<i32>) -> String {
    let dir = escaped(dir.as_os_str().as_encoded_bytes());
    let mut rendered = match status {
        Some(code) => format!("rc> {code}\n"),
        None => "rc> none\n".to_string(),
    };
    for (tag, bytes) in [("out", stdout), ("err", stderr)] {
        for line in escaped(bytes).replace(&dir, "DIR").lines() {
            rendered.push_str(&format!("{tag}> {line}\n"));
        }
    }
    rendered
}

#[test]
fn each_route_runs_a_restored_program_as_the_oracles_does() {
    let (theirs, ours) = (Side::oracle(), Side::ours("crate"));
    let mut cases = 0usize;
    let mut diverged = Vec::new();
    datadriven::walk(CASE_DIR, |file| {
        file.run(|case| {
            cases += 1;
            let oracle = theirs.run(case);
            let crate_side = ours.run(case);
            if crate_side != oracle {
                diverged.push(format!(
                    "{} {}:\n--- oracle\n{oracle}--- crate\n{crate_side}",
                    case.directive,
                    case.input.replace('\n', " ")
                ));
            }
            oracle
        });
    });
    std::fs::remove_dir_all(&theirs.dir).expect("the oracle directory is removed");
    std::fs::remove_dir_all(&ours.dir).expect("the crate directory is removed");
    assert!(cases > 0, "{CASE_DIR} produced no cases");
    assert!(diverged.is_empty(), "{}", diverged.join("\n"));
}

/// The report this crate's `rexx` makes for `image` in `dir`.
fn ours_on(side: &Side, image: &str, bytes: &[u8]) -> String {
    std::fs::write(side.dir.join(image), bytes).expect("the image");
    let output = side
        .command(&side.rexx)
        .arg(image)
        .output()
        .expect("rexx starts");
    render(&side.dir, &output.stdout, &output.stderr, output.status.code())
}

/// The report for an image refused with `sub`: 3.903 or 3.905.
fn refused(image: &str, sub: u16) -> String {
    let message = match sub {
        903 => format!(
            "Error 3.903:  Program \"DIR/{image}\" cannot be run by this version of the REXX \
             interpreter."
        ),
        _ => format!("Error 3.905:  Encoded compiled program \"DIR/{image}\" cannot be decoded."),
    };
    format!("rc> 253\nerr> Error 3:  Failure during initialization.\nerr> {message}\n")
}

/// Review Focus 2: an image this crate does not write, or a damaged one,
/// never panics. Undecodable base64 is 3.905 and every other damage 3.903,
/// rc 253, each measured on the oracle where the oracle survives the image.
#[test]
fn a_foreign_or_damaged_image_is_refused() {
    let side = Side::ours("damaged");
    let oracle_image = std::fs::read(ORACLE_IMAGE).expect("the oracle's image");
    assert!(oracle_image.starts_with(IMAGE_PREFIX));
    assert_eq!(
        ours_on(&side, "oracle.rxo", &oracle_image),
        refused("oracle.rxo", 903)
    );

    let compile = |output: &str, flags: &[&str]| -> Vec<u8> {
        let written = side
            .command(&side.rexxc)
            .args(["fail.rex", output, "-s"])
            .args(flags)
            .output()
            .expect("rexxc starts");
        assert_eq!(written.status.code(), Some(0), "{output}");
        std::fs::read(side.dir.join(output)).expect("the image")
    };
    let binary = compile("b.rxo", &[]);
    let encoded = compile("e.rxo", &["-e"]);
    // The `#!` line, then the 48-byte header, then the source.
    let header = 20 + 48;
    let body = 20 + "/**/@REXX@\n".len();

    let mut damaged: Vec<(&str, Vec<u8>, u16)> = vec![
        ("header.rxo", binary[..header - 8].to_vec(), 903),
        ("source.rxo", binary[..binary.len() - 3].to_vec(), 903),
        // Line ends between quartets: a whole line cut away.
        ("line.rxo", encoded[..body + 73].to_vec(), 903),
        // Inside a quartet.
        ("quartet.rxo", encoded[..body + 6].to_vec(), 903),
    ];
    let mut broken = encoded.clone();
    broken[body + 5] = b'!';
    damaged.push(("broken.rxo", broken, 905));
    // Deviation 41: the encoded tag over base64 of bytes without the binary
    // tag. The oracle runs whatever it decoded in place.
    let mut untagged = b"#!/usr/bin/env rexx\n/**/@REXX@\n".to_vec();
    untagged.extend_from_slice(b"c2F5ICdkZWNvZGVkJwo=\n");
    damaged.push(("untagged.rxo", untagged, 903));
    for (image, bytes, sub) in damaged {
        assert_eq!(ours_on(&side, image, &bytes), refused(image, sub), "{image}");
    }
    std::fs::remove_dir_all(&side.dir).expect("the directory is removed");
}
