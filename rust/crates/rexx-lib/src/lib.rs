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

//! The Rexx-written part of the interpreter's own library: the `.orx` sources
//! the C++ interpreter runs to finish building its image, embedded at build
//! time from this repository's tracked copies.
//!
//! **Sources and their pins only.** Running them needs an `Interp`, which
//! lives in `rexx-exec`, and `rexx-exec` reads this crate -- so a driver here
//! would close a dependency cycle. The driver is `rexx_exec`'s
//! `Interp::bootstrap_library`.
//!
//! `build.rs` records each file's sha256 and refuses to build if it moves, so
//! an edit to a read-only upstream source is a build failure rather than a
//! bootstrap that silently installs something else.
//!
//! **The `unix` platform file is the one embedded.** `build.rs`'s `Source`
//! doc has the whole of that decision; the windows file is neither read nor
//! embedded.

/// One embedded program.
pub struct Program {
    /// The name the C++ resolves it by, which is what a `CALL` inside the
    /// bootstrap writes: `call 'StreamClasses.orx' rexxPackage`
    /// (`CoreClasses.orx:122`). Compared case-insensitively by
    /// [`lookup`], because the oracle's own program-name resolution is.
    pub name: &'static str,
    /// The file's bytes, exactly as tracked.
    pub source: &'static [u8],
    /// The sha256 `build.rs` checked `source` against, lower-case hex.
    pub sha256: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/programs.rs"));

/// The program the bootstrap starts at -- `BASEIMAGELOAD`, which
/// `memory/Setup.cpp:1786` resolves and `:1795` runs with `TheRexxPackage` as
/// its one argument.
pub const ENTRY: &str = "CoreClasses.orx";

/// The embedded program a `CALL` target names, or `None` for a name that is
/// not one of these.
///
/// Case-insensitive: a Rexx `CALL` target that came from a quoted string
/// keeps its own case, and the C++ resolves a program name through the file
/// system, which on the platforms this crate targets is not the same
/// question. Matching case-insensitively here is the wider of the two and is
/// what keeps `call 'streamclasses.orx'` inside the bootstrap from falling
/// out to the external search.
pub fn lookup(name: &str) -> Option<&'static Program> {
    PROGRAMS
        .iter()
        .find(|program| program.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every embedded source is non-empty and reachable by its own name.
    #[test]
    fn every_program_is_reachable_by_name() {
        for program in PROGRAMS {
            assert!(
                !program.source.is_empty(),
                "{} embedded as zero bytes",
                program.name
            );
            let found = lookup(program.name).expect("a program is reachable by its own name");
            assert_eq!(found.sha256, program.sha256);
        }
        assert!(lookup("nosuch.orx").is_none());
    }

    /// The entry point names one of the embedded programs. Without this,
    /// renaming a row leaves [`ENTRY`] pointing at nothing and the bootstrap
    /// fails at run time instead of at test time.
    #[test]
    fn the_entry_point_is_one_of_the_embedded_programs() {
        assert!(lookup(ENTRY).is_some(), "{ENTRY} is not embedded");
    }

    /// Every quoted `CALL` target in `program`, in source order.
    fn quoted_call_targets(program: &Program) -> Vec<String> {
        let text = String::from_utf8(program.source.to_vec()).expect("the source is UTF-8");
        let mut called = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix("call ") else {
                continue;
            };
            let Some(quoted) = rest.trim_start().strip_prefix('\'') else {
                continue;
            };
            let Some(end) = quoted.find('\'') else {
                continue;
            };
            called.push(quoted[..end].to_string());
        }
        called
    }

    /// `CoreClasses.orx`'s two `CALL`s name the other two rows, and neither
    /// of those two calls anything embedded.
    ///
    /// Derived from the embedded bytes rather than written down, so a file
    /// whose `CALL` target changed upstream reddens here instead of leaving
    /// the bootstrap to answer 43.1 at run time.
    ///
    /// **The second half is what makes the run-time recursion guard
    /// unnecessary rather than forgotten**: `Interp::enter_library_program`
    /// takes no activation-depth check, and this is what says the embedded
    /// set has no cycle to need one. Measured, neither non-entry file
    /// contains the word `call` at all.
    #[test]
    fn the_entry_program_calls_exactly_the_other_embedded_programs() {
        let entry = lookup(ENTRY).expect("the entry point is embedded");
        let mut called = quoted_call_targets(entry);
        called.sort();
        let mut expected: Vec<String> = PROGRAMS
            .iter()
            .map(|p| p.name.to_string())
            .filter(|name| name != ENTRY)
            .collect();
        expected.sort();
        assert_eq!(called, expected);

        for program in PROGRAMS.iter().filter(|p| p.name != ENTRY) {
            for target in quoted_call_targets(program) {
                assert!(
                    lookup(&target).is_none(),
                    "{} calls the embedded {target}, so the bootstrap can recurse and \
                     `enter_library_program` needs a depth guard it does not have",
                    program.name
                );
            }
        }
    }
}
