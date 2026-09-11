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

use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::process::ExitCode;

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
    let invocation = rexx_exec::join_command_line(args.map(|arg| arg.as_bytes().to_vec()))
        .with_input(rexx_exec::ProgramInput::Stdin);

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
    ExitCode::from(outcome.exit_code as u8)
}
