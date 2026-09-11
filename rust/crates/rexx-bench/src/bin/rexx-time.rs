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

//! A minimal stand-in for `hyperfine`: run a command a fixed number of times
//! and report min/median/mean wall time.

use rexx_bench::timing::{Capture, Summary, time_once};
use std::process::ExitCode;
use std::time::Duration;

fn main() -> ExitCode {
    let mut warmup = 5usize;
    let mut runs = 20usize;

    let mut args = std::env::args().skip(1);
    let mut command: Vec<String> = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--warmup" => warmup = next_number(&mut args, "--warmup"),
            "--runs" => runs = next_number(&mut args, "--runs"),
            "--" => {
                command.extend(args);
                break;
            }
            other => {
                command.push(other.to_string());
                command.extend(args);
                break;
            }
        }
    }

    let Some((program, program_args)) = command.split_first() else {
        eprintln!("usage: rexx-time [--warmup N] [--runs N] <command> [args...]");
        return ExitCode::from(2);
    };

    for _ in 0..warmup {
        if run_once(program, program_args).is_none() {
            eprintln!("warmup run failed: {program} {program_args:?}");
            return ExitCode::FAILURE;
        }
    }

    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        let Some(wall) = run_once(program, program_args) else {
            eprintln!("run failed: {program} {program_args:?}");
            return ExitCode::FAILURE;
        };
        samples.push(wall);
    }
    // A run that failed to launch at all reports nothing rather than a
    // fabricated zero -- an empty `samples` here would already have
    // returned above, but guard the arithmetic below regardless.
    let Some(summary) = Summary::of(&mut samples) else {
        eprintln!("no runs completed");
        return ExitCode::FAILURE;
    };

    println!("command: {program} {}", program_args.join(" "));
    println!("runs: {runs} ({warmup} warm-up runs discarded)");
    println!("min:    {:>10.3} ms", millis(summary.min));
    println!("median: {:>10.3} ms", millis(summary.median));
    println!("mean:   {:>10.3} ms", millis(summary.mean));
    println!("max:    {:>10.3} ms", millis(summary.max));
    ExitCode::SUCCESS
}

/// Wall time of one successful run, or `None` if it could not be launched or
/// did not exit 0.
fn run_once(program: &str, args: &[String]) -> Option<Duration> {
    let completed = time_once(program, args, &[], Capture::Discard).ok()?;
    completed.succeeded().then_some(completed.wall)
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn next_number(args: &mut impl Iterator<Item = String>, flag: &str) -> usize {
    args.next()
        .unwrap_or_else(|| panic!("{flag} needs a value"))
        .parse()
        .unwrap_or_else(|e| panic!("{flag} value is not a number: {e}"))
}
