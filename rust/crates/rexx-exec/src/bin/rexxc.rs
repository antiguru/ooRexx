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

//! `rexxc`, the command line `utilities/rexxc/platform/unix/RexxCompiler.cpp`
//! reads.

use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::process::ExitCode;

use rexx_exec::compiled;
use rexx_exec::launcher;

fn main() -> ExitCode {
    rexx_exec::install_signal_handlers();
    let args: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(OsStringExt::into_vec)
        .collect();
    let compile = match launcher::parse_compile(&args) {
        Ok(compile) => compile,
        Err(usage) => {
            let _ = std::io::stdout().write_all(&usage);
            let _ = std::io::stdout().flush();
            return ExitCode::from(launcher::COMPILE_USAGE_STATUS);
        }
    };
    if !compile.silent {
        let mut banner = launcher::version_banner();
        banner.push(b'\n');
        let _ = std::io::stdout().write_all(&banner);
    }
    if compile.output.as_ref() == Some(&compile.input) {
        let _ = std::io::stdout().write_all(&launcher::same_file_message());
        let _ = std::io::stdout().flush();
        return ExitCode::from(launcher::COMPILE_SAME_FILE_STATUS);
    }
    let _ = std::io::stdout().flush();
    let (found, text) = match launcher::load(&compile.input) {
        Ok(loaded) => loaded,
        Err(failure) => return report(failure),
    };
    let image = compile
        .output
        .as_ref()
        .map(|_| compiled::write_image(&text, compile.encode));
    let outcome = launcher::translate(&found, text);
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);
    if outcome.exit_code != 0 {
        // A process exit status is one byte.
        return ExitCode::from(outcome.exit_code as u8);
    }
    if let (Some(output), Some(image)) = (&compile.output, image)
        && write(output, &image).is_err()
    {
        return report(launcher::output_unwritable(output));
    }
    ExitCode::SUCCESS
}

/// Writes `image` to the file `name`, removing what a failed write left.
fn write(name: &[u8], image: &[u8]) -> std::io::Result<()> {
    let path = OsStr::from_bytes(name);
    let mut file = std::fs::File::create(path)?;
    file.write_all(image).inspect_err(|_| {
        let _ = std::fs::remove_file(path);
    })
}

/// Writes a report the launcher makes itself and exits with its status.
fn report((text, status): (Vec<u8>, i32)) -> ExitCode {
    let _ = std::io::stderr().write_all(&text);
    ExitCode::from(status as u8)
}
