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

//! The `rexx` and `rexxc` command lines' library half: option scanning, the
//! program search, and the texts the launchers print themselves
//! (`utilities/rexx/platform/unix/rexx.cpp`,
//! `utilities/rexxc/platform/unix/RexxCompiler.cpp`).

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use rexx_core::ObjRef;

use crate::error::{ClauseSite, Raised};
use crate::{Failure, Interp, Outcome, require, version};

/// What a command line asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// The usage text on standard error, status 255.
    Usage,
    /// The version banner on standard output, status 0.
    Version,
    /// A program to run.
    Run(Run),
}

/// A program and what the command line hands it.
#[derive(Debug, PartialEq, Eq)]
pub struct Run {
    /// The program.
    pub program: Program,
    /// The words joined into the argument string.
    pub words: Vec<Vec<u8>>,
    /// `.SYSCARGS`' words.
    pub syscargs: Vec<Vec<u8>>,
    /// The index `.SYSCARGS` stores its first word at.
    pub syscargs_base: usize,
    /// The `::OPTIONS` directive and override count `-o` and `-od` give.
    pub default_options: Option<(Vec<u8>, i64)>,
}

/// Where a program's text comes from.
#[derive(Debug, PartialEq, Eq)]
pub enum Program {
    /// A name, searched for as [`resolve_program`] searches.
    File(Vec<u8>),
    /// `-e`'s text, run as `INSTORE`.
    Instore(Vec<u8>),
}

/// The usage text, written to standard error.
pub const USAGE: &[u8] = b"\nSyntax is \"rexx [-o[d] \"options\"] filename [arguments]\"\n\
or        \"rexx -e program_string [arguments]\"\n\
or        \"rexx -v\".\n";

/// The status the usage text exits with: `main`'s `return -1`.
pub const USAGE_STATUS: u8 = 255;

/// The longest `::OPTIONS` string `-o` keeps: `overrideString`'s 512 bytes
/// less the terminator.
const OVERRIDE_LIMIT: usize = 511;

/// `main`'s scan of `args`, the words after the binary's own name.
pub fn parse(args: &[Vec<u8>]) -> Command {
    let argc = args.len() + 1;
    let mut argc_base = 2;
    let mut program = None;
    let mut real_argument = true;
    let mut words = Vec::new();
    let mut default_options = None;
    let mut i = 1;
    while i < argc {
        let arg = &args[i - 1];
        if program.is_none() && arg.first() == Some(&b'-') {
            match arg.get(1) {
                Some(b'e' | b'E') => {
                    if argc != i + 1 {
                        program = Some(Program::Instore(args[i].clone()));
                        real_argument = false;
                    }
                }
                Some(b'o' | b'O') => {
                    if argc != i + 1 {
                        let count = match arg.get(2) {
                            Some(b'd' | b'D') => -1,
                            _ => 1,
                        };
                        argc_base += 2;
                        i += 1;
                        default_options = Some((options_directive(&args[i - 1..]), count));
                        real_argument = false;
                    }
                }
                Some(b'v' | b'V') => return Command::Version,
                _ => argc_base += 1,
            }
        } else {
            if program.is_none() {
                program = Some(Program::File(arg.clone()));
            } else if real_argument {
                words.push(arg.clone());
            }
            real_argument = true;
        }
        i += 1;
    }
    let Some(program) = program else {
        return Command::Usage;
    };
    let syscargs = args.get(argc_base - 1..).unwrap_or_default().to_vec();
    Command::Run(Run {
        program,
        words,
        syscargs,
        syscargs_base: argc_base - 1,
        default_options,
    })
}

/// The directive `-o`'s string `rest[0]` gives: the string itself when the
/// byte after its first non-blank is a colon, `::OPTIONS ` and the string
/// otherwise, cut to [`OVERRIDE_LIMIT`].
///
/// **The byte looked at can lie past the string.** `rexx.cpp:116` reads it
/// out of `argv`'s memory, where an empty or all-blank string is followed by
/// its terminator and the next word: measured, `-o '' ':x'` sends the empty
/// string (93.900, argument 2 must not be empty).
fn options_directive(rest: &[Vec<u8>]) -> Vec<u8> {
    let text = &rest[0];
    let first = text
        .iter()
        .position(|byte| *byte != b' ')
        .unwrap_or(text.len());
    let looked_at = rest
        .iter()
        .flat_map(|word| word.iter().copied().chain([0]))
        .nth(first + 1);
    let mut directive = if looked_at == Some(b':') {
        text.clone()
    } else {
        [b"::OPTIONS ".as_slice(), text].concat()
    };
    directive.truncate(OVERRIDE_LIMIT);
    directive
}

/// The program `name` names: `SysFileSystem::resolveProgramName` for a
/// program with no caller, so the current directory, then `REXX_PATH`, then
/// `PATH`, each with the default extensions.
pub fn resolve_program(
    name: &OsStr,
    cwd: &Path,
    rexx_path: Option<&OsStr>,
    path: Option<&OsStr>,
) -> Option<PathBuf> {
    require::search(
        None,
        name.as_bytes(),
        rexx_path.map(OsStr::as_bytes),
        path.map(OsStr::as_bytes),
        cwd.as_os_str().as_bytes(),
        false,
    )
    .map(|found| PathBuf::from(OsString::from_vec(found)))
}

/// The program `name` names and its text, or the report and status for one
/// that cannot be found or read.
pub fn load(name: &[u8]) -> Result<(PathBuf, Vec<u8>), (Vec<u8>, i32)> {
    let found = std::env::current_dir().ok().and_then(|cwd| {
        resolve_program(
            OsStr::from_bytes(name),
            &cwd,
            std::env::var_os("REXX_PATH").as_deref(),
            std::env::var_os("PATH").as_deref(),
        )
    });
    let Some(found) = found else {
        return Err(program_not_found(name));
    };
    match std::fs::read(&found) {
        Ok(text) => Ok((found, text)),
        Err(_) => Err(program_unreadable(found.as_os_str().as_bytes())),
    }
}

/// Translates the program at `path` without running it, reporting a refusal
/// exactly as `rexx` would.
pub fn translate(path: &Path, text: Vec<u8>) -> Outcome {
    crate::translate_program(path.as_os_str().as_bytes(), text)
}

/// What a `rexxc` command line asks for.
#[derive(Debug, PartialEq, Eq)]
pub struct Compile {
    /// The program's name.
    pub input: Vec<u8>,
    /// The image's file name, `None` for a syntax check.
    pub output: Option<Vec<u8>>,
    /// `-s`: no banner.
    pub silent: bool,
    /// `-e`: the encoded image.
    pub encode: bool,
}

/// The status `rexxc` exits with after its usage text: `exit(-1)`.
pub const COMPILE_USAGE_STATUS: u8 = 255;

/// The status `rexxc` exits with when the output names the input: `exit(-2)`.
pub const COMPILE_SAME_FILE_STATUS: u8 = 254;

/// A `rexxc` message by its catalogue code, as `displayError` prints it to
/// standard output.
fn rexxc_message(sub: u16) -> Vec<u8> {
    let message = rexx_inventory::errors::lookup(999, sub).expect("a catalogued rexxc message");
    format!("{}\n", message.text).into_bytes()
}

/// `RexxCompiler.cpp`'s `main` scan of `args`, the words after the binary's
/// own name: the command, or the usage text for standard output.
pub fn parse_compile(args: &[Vec<u8>]) -> Result<Compile, Vec<u8>> {
    let usage = |incorrect: bool| {
        let mut text = if incorrect {
            rexxc_message(12)
        } else {
            Vec::new()
        };
        text.extend(rexxc_message(14));
        text.extend(rexxc_message(15));
        text
    };
    let Some((input, rest)) = args.split_first() else {
        return Err(usage(false));
    };
    let (output, flags) = match rest.split_first() {
        Some((output, flags)) if output.first() != Some(&b'-') => (Some(output.clone()), flags),
        _ => (None, rest),
    };
    let mut compile = Compile {
        input: input.clone(),
        output,
        silent: false,
        encode: false,
    };
    for flag in flags {
        match (flag.first(), flag.get(1)) {
            (Some(b'-'), Some(b's' | b'S')) => compile.silent = true,
            (Some(b'-'), Some(b'e' | b'E')) => compile.encode = true,
            _ => return Err(usage(true)),
        }
    }
    Ok(compile)
}

/// `rexxc`'s refusal when the output and input names are the same bytes.
pub fn same_file_message() -> Vec<u8> {
    rexxc_message(13)
}

/// The report and status for an image that cannot be written to `name`:
/// 3.902, displayed with no position.
pub fn output_unwritable(name: &[u8]) -> (Vec<u8>, i32) {
    positionless(Raised::syntax(3, 902, vec![name.to_vec()]))
}

/// `RexxGetVersionInformation`'s text (`api/InterpreterAPI.cpp:250-278`),
/// without the line end `rexx -v` writes after it.
pub fn version_banner() -> Vec<u8> {
    let date = std::str::from_utf8(version::BUILD_DATE).expect("an ASCII build date");
    let mut fields = date.split(' ');
    let (day, month, year) = match (fields.next(), fields.next(), fields.next()) {
        (Some(day), Some(month), Some(year)) => (day, month, year),
        _ => unreachable!("BUILD_DATE is day, month and year"),
    };
    let text = std::str::from_utf8(version::VERSION).expect("an ASCII version");
    let bits = text
        .split_once(")_")
        .and_then(|(_, rest)| rest.split_once("-bit"))
        .map_or("64", |(bits, _)| bits);
    format!(
        "Open Object Rexx Version {} r0\n\
         Build date: {month} {day:>2} {year}\n\
         Addressing mode: {bits}\n\
         Copyright (c) 1995, 2004 IBM Corporation. All rights reserved.\n\
         Copyright (c) 2005-{year} Rexx Language Association. All rights reserved.\n\
         This program and the accompanying materials are made available under the terms\n\
         of the Common Public License v1.0 which accompanies this distribution or at\n\
         https://www.oorexx.org/license.html",
        String::from_utf8_lossy(version::VERSION_NUMBER)
    )
    .into_bytes()
}

/// The report and status for a program `name` the search did not find:
/// 3.901, displayed with no position.
pub fn program_not_found(name: &[u8]) -> (Vec<u8>, i32) {
    positionless(Raised::syntax(3, 901, vec![name.to_vec()]))
}

/// The report and status for a program found at `path` that cannot be read.
pub fn program_unreadable(path: &[u8]) -> (Vec<u8>, i32) {
    positionless(Raised::syntax(3, 1, vec![path.to_vec()]))
}

/// The report `Activity::display` writes for `raised` with no program
/// position, and the status `main` returns for it.
pub(crate) fn positionless(mut raised: Raised) -> (Vec<u8>, i32) {
    raised.delivery.positionless = true;
    let report = raised.report(&ClauseSite {
        path: b"",
        sites: &[],
    });
    (report, raised.exit_code())
}

/// The status `RexxStart` hands `-e`'s caller: its `short` return code, so a
/// result outside that range is 0 (`RexxStartDispatcher.cpp:174-179`).
pub fn instore_status(exit_code: i32) -> i32 {
    i16::try_from(exit_code).map_or(0, i32::from)
}

impl Interp {
    /// `rexx.cpp:205-216`: `.Package~defaultOptions('D', directive)`, then
    /// `('C', count)`, sent from outside any activation.
    pub(crate) fn send_default_options(
        &mut self,
        directive: &[u8],
        count: i64,
    ) -> Result<(), Failure> {
        let Some(package) = self.system_symbol(b"PACKAGE") else {
            return Err(crate::Loud::environment_symbol(b".PACKAGE", None).into());
        };
        let define = self.text(b"D");
        self.roots.activity_mut().push_temp(define);
        let directive = self.text(directive);
        self.roots.activity_mut().push_temp(directive);
        let caller = self.caller();
        self.send_message(
            package,
            b"DEFAULTOPTIONS",
            None,
            &[Some(define), Some(directive)],
            caller,
        )
        .map_err(|failure| self.displayed_without_position(failure))?;
        let overrides = self.text(b"C");
        self.roots.activity_mut().push_temp(overrides);
        let count =
            ObjRef::small_int(count).unwrap_or_else(|| self.text(count.to_string().as_bytes()));
        self.roots.activity_mut().push_temp(count);
        self.send_message(
            package,
            b"DEFAULTOPTIONS",
            None,
            &[Some(overrides), Some(count)],
            caller,
        )?;
        Ok(())
    }

    /// `failure` as `Activity::display` shows a condition no clause of a
    /// program raised: with no position when no recorded site has a line.
    fn displayed_without_position(&self, failure: Failure) -> Failure {
        let mut sites = self
            .activity
            .failure_sites
            .iter()
            .chain(&self.activity.failure_site);
        match failure {
            Failure::Raised(mut raised) if !sites.any(|site| site.line().is_some()) => {
                raised.delivery.positionless = true;
                Failure::Raised(raised)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(args: &[&str]) -> Vec<Vec<u8>> {
        args.iter().map(|arg| arg.as_bytes().to_vec()).collect()
    }

    #[test]
    fn an_ignored_option_and_an_options_string_shift_syscargs() {
        let Command::Run(run) = parse(&words(&["-x", "-o", "digits 12", "p", "a", "b"])) else {
            panic!("a program to run");
        };
        assert_eq!(run.program, Program::File(b"p".to_vec()));
        assert_eq!(run.words, words(&["a", "b"]));
        assert_eq!(run.syscargs, words(&["a", "b"]));
        assert_eq!(run.syscargs_base, 4);
        assert_eq!(
            run.default_options,
            Some((b"::OPTIONS digits 12".to_vec(), 1))
        );
    }

    #[test]
    fn the_options_colon_is_the_byte_after_the_first_non_blank() {
        let directive = |rest: &[&str]| options_directive(&words(rest));
        assert_eq!(directive(&["::options x", "p"]), b"::options x");
        assert_eq!(directive(&["  ::options x", "p"]), b"  ::options x");
        assert_eq!(directive(&["x:options", "p"]), b"x:options");
        assert_eq!(directive(&[":options", "p"]), b"::OPTIONS :options");
        assert_eq!(directive(&["", ":x"]), b"");
        assert_eq!(directive(&["", "x"]), b"::OPTIONS ");
        assert_eq!(directive(&[""]), b"::OPTIONS ");
        assert_eq!(directive(&[&"d".repeat(600)]).len(), OVERRIDE_LIMIT);
    }

    #[test]
    fn instore_text_is_not_an_argument_and_a_trailing_option_is_usage() {
        let Command::Run(run) = parse(&words(&["-e", "say 1", "-e", "a"])) else {
            panic!("a program to run");
        };
        assert_eq!(run.program, Program::Instore(b"say 1".to_vec()));
        assert_eq!(run.words, words(&["-e", "a"]));
        for args in [&["-e"][..], &["-o"], &["-od"], &["-x"], &["-"], &[]] {
            assert_eq!(parse(&words(args)), Command::Usage, "{args:?}");
        }
        assert_eq!(parse(&words(&["-?", "-v"])), Command::Version);
        assert_eq!(parse(&words(&["-vx"])), Command::Version);
    }

    #[test]
    fn the_banner_carries_the_parse_version_date() {
        let banner = String::from_utf8(version_banner()).expect("ASCII");
        assert!(banner.starts_with("Open Object Rexx Version 5.3.0 r0\nBuild date: Jul 30 2026\n"));
        assert!(!banner.ends_with('\n'));
    }

    #[test]
    fn rexxc_reads_an_output_then_flags_by_their_second_byte() {
        let compile = |args: &[&str]| parse_compile(&words(args));
        assert_eq!(
            compile(&["a", "b", "-Sx", "-e"]),
            Ok(Compile {
                input: b"a".to_vec(),
                output: Some(b"b".to_vec()),
                silent: true,
                encode: true,
            })
        );
        assert_eq!(
            compile(&["-x"]).map(|compile| compile.input),
            Ok(b"-x".to_vec())
        );
        assert_eq!(
            compile(&["a", "-s"]).map(|compile| compile.output),
            Ok(None)
        );
        let usage = b"SYNTAX: rexxc inputfile [outputfile] [-s] [-e]\n\
                      Without outputfile rexxc only performs a syntax check.\n";
        assert_eq!(compile(&[]), Err(usage.to_vec()));
        for args in [
            &["a", "-x"][..],
            &["a", "-"],
            &["a", "b", "c"],
            &["a", "-s", "b"],
        ] {
            assert_eq!(
                compile(args),
                Err([&b"The rexxc command parameters are incorrect.\n"[..], usage].concat()),
                "{args:?}"
            );
        }
    }

    #[test]
    fn an_instore_status_outside_a_short_is_zero() {
        assert_eq!(instore_status(-3), -3);
        assert_eq!(instore_status(70000), 0);
        assert_eq!(instore_status(32767), 32767);
    }
}
