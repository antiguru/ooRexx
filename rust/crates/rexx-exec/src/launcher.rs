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

//! The `rexx` command line's library half: option scanning, the program
//! search, and the texts the launcher prints itself
//! (`utilities/rexx/platform/unix/rexx.cpp`).

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use rexx_core::ObjRef;

use crate::error::{ClauseSite, Raised};
use crate::{Failure, Interp, require, version};

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
    /// `.SYSCARGS`' words, and the index the first is stored at.
    pub syscargs: (Vec<Vec<u8>>, usize),
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
        syscargs: (syscargs, argc_base - 1),
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
    let entries = require::search_entries(
        None,
        rexx_path
            .map(|path| require::widen(path.as_bytes()))
            .as_deref(),
        path.map(|path| require::widen(path.as_bytes())).as_deref(),
    );
    let cwd = cwd.as_os_str().as_bytes();
    let groups = require::candidates(&require::widen(name.as_bytes()), &entries, None, false);
    let located = |candidate: &str| {
        PathBuf::from(OsString::from_vec(crate::paths::normalize_bytes(
            &require::narrow(candidate),
            cwd,
        )))
    };
    let hit = require::first_regular(&groups, |candidate| {
        match std::fs::metadata(located(candidate)) {
            Ok(meta) if meta.is_file() => require::Found::Regular,
            Ok(_) => require::Found::Other,
            Err(_) => require::Found::Missing,
        }
    })?;
    Some(located(&hit))
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
fn positionless(mut raised: Raised) -> (Vec<u8>, i32) {
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
        assert_eq!(run.syscargs, (words(&["a", "b"]), 4));
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
    fn widening_a_name_round_trips_every_byte() {
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(require::narrow(&require::widen(&all)), all);
    }

    #[test]
    fn an_instore_status_outside_a_short_is_zero() {
        assert_eq!(instore_status(-3), -3);
        assert_eq!(instore_status(70000), 0);
        assert_eq!(instore_status(32767), 32767);
    }
}
