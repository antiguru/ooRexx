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

//! `rexx`, the command line `utilities/rexx/platform/unix/rexx.cpp` reads.

use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::process::ExitCode;

use rexx_exec::launcher::{self, Command, Program};

fn main() -> ExitCode {
    rexx_exec::install_signal_handlers();
    let args: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(OsStringExt::into_vec)
        .collect();
    let run = match launcher::parse(&args) {
        Command::Usage => {
            let _ = std::io::stderr().write_all(launcher::USAGE);
            return ExitCode::from(launcher::USAGE_STATUS);
        }
        Command::Version => {
            let mut banner = launcher::version_banner();
            banner.push(b'\n');
            let _ = std::io::stdout().write_all(&banner);
            let _ = std::io::stdout().flush();
            return ExitCode::SUCCESS;
        }
        Command::Run(run) => run,
    };

    let mut invocation = rexx_exec::join_command_line(&run.words)
        .with_input(rexx_exec::ProgramInput::Stdin)
        .with_standard_transient(std::array::from_fn(|fd| {
            use std::os::unix::fs::FileTypeExt;
            std::fs::metadata(format!("/proc/self/fd/{fd}"))
                .is_ok_and(|meta| meta.file_type().is_char_device() || meta.file_type().is_fifo())
        }))
        // A prompt has to be written before the read that waits for its
        // answer, and the interpreter only appends to a buffer.
        .with_sinks(rexx_exec::Sinks {
            stdout: Box::new(|bytes| {
                let _ = std::io::stdout().write_all(bytes);
                let _ = std::io::stdout().flush();
            }),
            stderr: Box::new(|bytes| {
                let _ = std::io::stderr().write_all(bytes);
                let _ = std::io::stderr().flush();
            }),
        });

    // `REXX_SWITCH_MODE` is the deterministic switch mode, for tests:
    // `every`, `at:K` for the program's K-th clause, or the simulation mode,
    // `sim`, `sim:SEED[,...]` or `sim:replay=FILE[,trace=FILE]`.
    invocation = match std::env::var("REXX_SWITCH_MODE").as_deref() {
        Ok("every") => invocation.with_switch_mode(rexx_exec::SwitchMode::EveryOpportunity),
        Ok(mode) if mode == "sim" || mode.starts_with("sim:") => {
            match rexx_exec::SimConfig::parse(mode) {
                Ok(config) => {
                    if mode == "sim" {
                        eprintln!("rexx-sim: seed={}", config.seed);
                    }
                    invocation.with_switch_mode(rexx_exec::SwitchMode::Sim(config))
                }
                Err(error) => {
                    eprintln!("rexx: REXX_SWITCH_MODE: {error}");
                    return ExitCode::from(2);
                }
            }
        }
        Ok(mode) => match mode.strip_prefix("at:").map(str::parse) {
            Some(Ok(clause)) => {
                invocation.with_switch_mode(rexx_exec::SwitchMode::AtClause(clause))
            }
            _ => {
                eprintln!(
                    "rexx: REXX_SWITCH_MODE is `every`, `at:K` or `sim[:SEED,...]`, not `{mode}`"
                );
                return ExitCode::from(2);
            }
        },
        Err(_) => invocation,
    };

    let (outcome, instore) = match run.program {
        Program::Instore(text) => (
            rexx_exec::run_program_named(b"INSTORE", text, invocation.without_syscargs()),
            true,
        ),
        Program::File(name) => {
            invocation = invocation.with_syscargs(run.syscargs, run.syscargs_base);
            let options = run.default_options.is_some();
            if let Some((directive, count)) = run.default_options {
                invocation = invocation.with_default_options(directive, count);
            }
            match load(&name) {
                Ok((found, text)) => (
                    rexx_exec::run_program_named(found.as_os_str().as_bytes(), text, invocation),
                    false,
                ),
                // The oracle sends `-o`'s options before it looks for the
                // program, so their failure is the one reported. An empty
                // program runs nothing but that send.
                Err(failure) if options => {
                    let outcome = rexx_exec::run_program_named(b"", Vec::new(), invocation);
                    if outcome.exit_code == 0 {
                        return report(failure);
                    }
                    (outcome, false)
                }
                Err(failure) => return report(failure),
            }
        }
    };

    // What the sinks did not already take.
    let _ = std::io::stdout().write_all(&outcome.stdout);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&outcome.stderr);
    if let Some(sim) = outcome
        .sim
        .filter(|_| std::env::var_os(rexx_exec::SIM_CHILD_ENV).is_none())
    {
        let trace = sim
            .trace_hash
            .map(|hash| format!(" trace={hash:016x}"))
            .unwrap_or_default();
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        eprintln!(
            "rexx-sim: seed={} policy={} steps={} contended={} switches={}{trace} profile={profile} \
             stack={}",
            sim.seed,
            sim.policy,
            sim.steps,
            sim.contended,
            sim.switches,
            rexx_exec::INTERPRETER_STACK_BYTES
        );
    }

    let status = if instore {
        launcher::instore_status(outcome.exit_code)
    } else {
        outcome.exit_code
    };
    // A process exit status is one byte.
    ExitCode::from(status as u8)
}

/// The program `name` names and its text, or the report and status for one
/// that cannot be found or read.
fn load(name: &[u8]) -> Result<(PathBuf, Vec<u8>), (Vec<u8>, i32)> {
    let found = std::env::current_dir().ok().and_then(|cwd| {
        launcher::resolve_program(
            OsStr::from_bytes(name),
            &cwd,
            std::env::var_os("REXX_PATH").as_deref(),
            std::env::var_os("PATH").as_deref(),
        )
    });
    let Some(found) = found else {
        return Err(launcher::program_not_found(name));
    };
    match std::fs::read(&found) {
        Ok(text) => Ok((found, text)),
        Err(_) => Err(launcher::program_unreadable(found.as_os_str().as_bytes())),
    }
}

/// Writes a report the launcher makes itself and exits with its status.
fn report((text, status): (Vec<u8>, i32)) -> ExitCode {
    let _ = std::io::stderr().write_all(&text);
    ExitCode::from(status as u8)
}
