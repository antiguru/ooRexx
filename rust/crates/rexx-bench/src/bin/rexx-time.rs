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
//!
//! `hyperfine` is not installed in this environment and cannot be installed
//! (no network), so Task 0.7's cold-start measurement (D2's gate) needs its
//! own timer. Runs the command N times after a warm-up and reports
//! min/median/mean, plus max for context.
//!
//! Usage: `rexx-time [--warmup N] [--runs N] <command> [args...]`
//! (a leading `--` before the command is accepted and skipped, for
//! readability at call sites).

use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

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
        if !run_once(program, program_args) {
            eprintln!("warmup run failed: {program} {program_args:?}");
            return ExitCode::FAILURE;
        }
    }

    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        if !run_once(program, program_args) {
            eprintln!("run failed: {program} {program_args:?}");
            return ExitCode::FAILURE;
        }
        samples.push(start.elapsed());
    }
    // A run that failed to launch at all reports nothing rather than a
    // fabricated zero -- an empty `samples` here would already have
    // returned above, but guard the arithmetic below regardless.
    if samples.is_empty() {
        eprintln!("no runs completed");
        return ExitCode::FAILURE;
    }

    samples.sort();
    let min = samples[0];
    let max = samples[samples.len() - 1];
    let median = samples[samples.len() / 2];
    let mean = samples.iter().sum::<Duration>() / samples.len() as u32;

    println!("command: {program} {}", program_args.join(" "));
    println!("runs: {runs} ({warmup} warm-up runs discarded)");
    println!("min:    {:>10.3} ms", min.as_secs_f64() * 1000.0);
    println!("median: {:>10.3} ms", median.as_secs_f64() * 1000.0);
    println!("mean:   {:>10.3} ms", mean.as_secs_f64() * 1000.0);
    println!("max:    {:>10.3} ms", max.as_secs_f64() * 1000.0);
    ExitCode::SUCCESS
}

fn run_once(program: &str, args: &[String]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn next_number(args: &mut impl Iterator<Item = String>, flag: &str) -> usize {
    args.next()
        .unwrap_or_else(|| panic!("{flag} needs a value"))
        .parse()
        .unwrap_or_else(|e| panic!("{flag} value is not a number: {e}"))
}
