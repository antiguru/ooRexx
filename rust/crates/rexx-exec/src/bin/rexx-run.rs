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

//! The runner the L0 differential tests drive: `rexx-run FILE`.
//!
//! Deliberately thin. **The sized interpreter thread is not here**, it is in
//! `rexx_exec::run_program`, because the L0 harness and the assertion-table
//! harness both call that function in process rather than through this binary,
//! and a `cargo test` thread's stack is far smaller than the one D19's depth
//! limit is calibrated against. Everything this file does is read bytes, hand
//! them over, and write back what came out.

use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::process::ExitCode;

/// Which engine this run uses, from `REXX_ENGINE`.
///
/// **An environment variable and not a command-line option**, for the reason
/// `main`'s own comment gives for having no option parsing at all: every word
/// after the program path is the program's argument, and a leading `-x` that
/// this binary swallowed would silently change what a program is given.
///
/// It exists because the two engines are the same build selected through
/// `Invocation`, so a benchmark comparison between them has to interleave
/// between arms of one binary within one sitting -- and nothing else in this
/// binary can reach `Invocation::with_engine`.
///
/// **Set-but-unrecognised is rejected, and that includes a value this platform
/// will not decode.** Only an unset variable defaults, to the same engine
/// `Invocation::none` picks. A typo, or a byte string that is not UTF-8, would
/// otherwise run whichever engine the default names while the caller believed
/// it had asked for the other -- which is worse than either running or
/// refusing, because a benchmark would attribute the result to the wrong arm
/// and read a comparison of one arm against itself as no movement.
fn engine_from_environment() -> rexx_exec::Engine {
    engine_from(std::env::var("REXX_ENGINE"))
}

/// [`engine_from_environment`]'s decision, over a value rather than over the
/// process.
///
/// **Split out so it can be tested at all.** The environment is process-wide,
/// `std::env::remove_var` is `unsafe` and this workspace forbids `unsafe`, and
/// libtest runs its cases on threads of one process -- so a test that unset
/// the variable would be both unwritable here and a race with every other
/// test if it were written. Over a `Result` there is nothing to unset.
fn engine_from(value: Result<String, std::env::VarError>) -> rexx_exec::Engine {
    use std::env::VarError;

    match value {
        Err(VarError::NotPresent) => rexx_exec::Engine::DEFAULT,
        Ok(value) => match value.as_str() {
            "ir" => rexx_exec::Engine::Ir,
            "tree-walker" => rexx_exec::Engine::TreeWalker,
            other => reject_engine(&format!("`{other}`")),
        },
        // Not decodable as UTF-8, so there is nothing to compare against
        // either spelling and nothing to print back either. Rejected rather
        // than defaulted for the reason above: it is set, so the caller asked
        // for something.
        Err(VarError::NotUnicode(_)) => reject_engine("a value that is not UTF-8"),
    }
}

/// Reports an unusable `REXX_ENGINE` and stops, with the status `main` uses
/// for a request it cannot carry out at all.
fn reject_engine(described: &str) -> ! {
    eprintln!("rexx-run: REXX_ENGINE is {described}: expected `ir` or `tree-walker`");
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: rexx-run FILE [arguments]");
        return ExitCode::from(2);
    };

    // Everything after the program path is the program's argument, joined into
    // the single string a Rexx program can see. `rexx_exec::join_command_line`
    // owns the joining rule and the measurements behind it; what belongs here
    // is only the decision to hand it every remaining word, because that
    // matches where the oracle does the same thing -- its own launcher
    // (`utilities/rexx/platform/unix/rexx.cpp`'s `main`) builds `arg_buffer`
    // from `argv` and the interpreter never sees the separate words.
    //
    // Bytes, not `String`: a command-line word is not required to be UTF-8 on
    // this platform and a Rexx string is a byte string, so lossy conversion
    // here would silently change the argument a program is given.
    //
    // No option parsing of any kind, which is the one place this binary is not
    // a thin wrapper over that launcher. `rexx` accepts `-e`, `-o`/`-od` and
    // `-v` before the program name; none of them is in Phase 4's scope, and
    // treating a leading `-x` as an option here would mean silently dropping a
    // word that would otherwise reach the program as part of its argument
    // string. Every word after the path is an argument.
    // This process's own standard input is what `.input` reads, and this is the
    // only place in the tree that asks for it: `ProgramInput`'s own doc has why
    // the in-process callers must not, and why the default is not this.
    let invocation = rexx_exec::join_command_line(args.map(|arg| arg.as_bytes().to_vec()))
        .with_input(rexx_exec::ProgramInput::Stdin)
        .with_engine(engine_from_environment());

    let text = match std::fs::read(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("rexx-run: {}: {error}", path.to_string_lossy());
            return ExitCode::from(2);
        }
    };

    // The oracle names the program in its error reports by the absolute,
    // dot-normalised path, whatever was typed on the command line: measured,
    // `./sub/../sub/rel.rex` and a bare `rel.rex` run from the directory both
    // report the same canonical path. `canonicalize` is that normalisation and
    // also resolves symlinks, which is one step further than the oracle has
    // been measured to go; nothing in the corpus runs through a symlink, so
    // that difference is unobserved rather than known to agree.
    //
    // A failure here falls back to the path as given rather than aborting: the
    // file has already been read successfully by this point, so a canonicalise
    // failure is a race or a permission quirk on the directory, and reporting
    // the program under the name the caller used beats refusing to run it.
    let reported = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone().into());
    let outcome = rexx_exec::run_program(&reported.to_string_lossy(), text, invocation);

    // Written in the order the program produced them relative to each other,
    // which is no order at all: they are separate descriptors, and D17 records
    // that their interleaving is not observable.
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);

    // `ExitCode::from` takes a `u8`, which is the whole range a process exit
    // status carries anyway. What makes that range meaningful is
    // `Raised::exit_code`'s `256 - major`, which `execute` applies, and now
    // `EXIT expr`'s own result, which `Interp::exit_code_for` (`lib.rs`)
    // converts into an `i32` that can be negative or wider than a byte.
    //
    // **A truncating cast, not `u8::try_from`, because the oracle wraps and
    // the old conversion saturated.** Measured:
    //
    //     exit 256   ->  rc 0        exit 257  ->  rc 1
    //     exit -1    ->  rc 255      exit 255  ->  rc 255
    //
    // so the oracle keeps only the low 8 bits of the value, which is exactly
    // what `as u8` does on an `i32` (defined, not implementation-specific:
    // Rust's numeric `as` narrows by truncating the two's-complement bit
    // pattern) -- `-1i32 as u8` is 255, `256i32 as u8` is 0, `257i32 as u8` is
    // 1, matching all four rows above. `u8::try_from(-1)` or `(256)` would
    // instead fail and fall back to 255 for every one of them, indistinguishable
    // from `exit -1` alone, which is the bug this replaces.
    ExitCode::from(outcome.exit_code as u8)
}

#[cfg(test)]
mod tests {
    use super::engine_from;
    use rexx_exec::Engine;
    use std::env::VarError;

    /// An unset `REXX_ENGINE` gives the same engine a caller who built an
    /// `Invocation` and chose nothing gets.
    ///
    /// **This binary is the one place naming an engine that the library's own
    /// test cannot see.** `Invocation::into_parts` is `pub(crate)`, so
    /// `invocation.rs` can pin the library half and nothing there reaches
    /// this half; the two are pinned to one `Engine::DEFAULT` from opposite
    /// sides instead. Without this, a flip applied to one and not the other
    /// would leave `rexx-run` running an engine the library says is not the
    /// default, and every gate would stay green.
    #[test]
    fn an_unset_variable_gives_the_librarys_own_default() {
        assert_eq!(engine_from(Err(VarError::NotPresent)), Engine::DEFAULT);
    }

    /// Both recognised spellings still select what they name, so the default
    /// above cannot be satisfied by a function that answers it for everything.
    ///
    /// The two rejecting arms are absent on purpose and not by oversight:
    /// `reject_engine` ends the process, which libtest cannot survive to
    /// assert on. What they refuse is stated where they are.
    #[test]
    fn each_spelling_selects_the_engine_it_names() {
        assert_eq!(engine_from(Ok("ir".to_string())), Engine::Ir);
        assert_eq!(
            engine_from(Ok("tree-walker".to_string())),
            Engine::TreeWalker
        );
    }
}
