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

//! Prints the compiled op stream of a program: `rexx-ir FILE [TRACE-SETTING]`.
//!
//! **A view of the compiler, and nothing is run.** `rexx_exec::render_ir` owns
//! what that means and what it cannot show; this file reads a path, hands the
//! bytes over, and writes the answer.
//!
//! The optional second word is the `TRACE` setting the chunk is compiled
//! under, spelled as a `TRACE` instruction spells it (`n`, `r`, `i`, `a`, ...).
//! It is an input to compilation rather than a display option, which is why it
//! is an argument here at all: under a setting that echoes, every promoted
//! clause carries an `Op::TraceClause` that the same body compiled untraced
//! does not have. It defaults to `n`, the setting a program runs under with no
//! `TRACE` instruction in it.

use std::io::Write;
use std::process::ExitCode;

/// The `TRACE` setting a chunk is compiled under when the caller names none.
///
/// `N`, because that is what an activation's setting is before any `TRACE`
/// instruction runs, so this is the stream a program's first pass through each
/// body actually uses.
const DEFAULT_SETTING: &[u8] = b"n";

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: rexx-ir FILE [TRACE-SETTING]");
        return ExitCode::from(2);
    };
    let setting = match args.next() {
        Some(word) => word.into_encoded_bytes(),
        None => DEFAULT_SETTING.to_vec(),
    };

    let text = match std::fs::read(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("rexx-ir: {}: {error}", path.to_string_lossy());
            return ExitCode::from(2);
        }
    };

    // On a thread with the stack the interpreter compiles bodies on, for the
    // reason `render_ir`'s own doc gives: the expression walk takes a frame per
    // operator, and a stack overflow aborts the process rather than reporting
    // anything.
    let rendered = std::thread::Builder::new()
        .stack_size(rexx_exec::INTERPRETER_STACK_BYTES)
        .spawn(move || rexx_exec::render_ir(text, &setting))
        .expect("spawning the compiler thread")
        .join();

    match rendered {
        Ok(Ok(text)) => {
            let _ = std::io::stdout().write_all(text.as_bytes());
            let _ = std::io::stdout().flush();
            ExitCode::SUCCESS
        }
        Ok(Err(report)) => {
            eprintln!("rexx-ir: {}: {report}", path.to_string_lossy());
            ExitCode::from(1)
        }
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
