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

    let outcome = rexx_exec::run_program(text);

    // Written in the order the program produced them relative to each other,
    // which is no order at all: they are separate descriptors, and D17 records
    // that their interleaving is not observable.
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);

    // `ExitCode::from` takes a `u8`, which is the whole range a process exit
    // status carries anyway. Task 12 owns the `256 - major` mapping that makes
    // this range meaningful.
    //
    // **This saturates and the oracle wraps, so it is wrong for any code
    // outside 0..=255, and it is Task 9's to fix** together with `EXIT expr`,
    // which is the only thing that can produce such a code. Measured:
    //
    //     exit 256   ->  rc 0        exit 257  ->  rc 1
    //     exit -1    ->  rc 255      exit 255  ->  rc 255
    //
    // so the oracle takes the value modulo 256, while `unwrap_or(u8::MAX)`
    // answers 255 for every one of them. Nothing 4a's spike can emit reaches
    // here out of range, since `NOT_IMPLEMENTED_EXIT` and 0 and 2 are the only
    // values, which is why this is a comment rather than an arithmetic change
    // made blind: the correct conversion needs the whole `EXIT` result rule,
    // including what a non-numeric or fractional result does, and none of that
    // is measured yet.
    ExitCode::from(u8::try_from(outcome.exit_code).unwrap_or(u8::MAX))
}
