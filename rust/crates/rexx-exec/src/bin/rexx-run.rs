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
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: rexx-run FILE");
        return ExitCode::from(2);
    };

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
    let outcome = rexx_exec::run_program(&reported.to_string_lossy(), text);

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
