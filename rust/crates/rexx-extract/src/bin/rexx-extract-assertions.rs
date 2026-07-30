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

//! Walks a checked-out ooTest suite and reports what `extract_assertions`
//! makes of every `.testGroup` file's `self~assertSame` calls: how many
//! became rows, how many were blocked and why, per group and in total.
//!
//! This is a reporting tool, not a data-file generator: the row set is
//! meant to be produced by calling `rexx_extract::extract_assertions`
//! directly against the `.testGroup` sources at the point of use (they are
//! already checked into the tree, so there is nothing to serialise), not by
//! reading a pre-baked file this binary would write. Building that consumer
//! needs a real evaluator and is a later task's job.

use rexx_extract::{extract_assertions, find_test_groups};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut suite = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--suite" => suite = args.next().map(PathBuf::from),
            other => {
                eprintln!("unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(suite) = suite else {
        eprintln!("usage: rexx-extract-assertions --suite <dir>");
        return ExitCode::from(2);
    };

    let mut groups: Vec<PathBuf> = find_test_groups(&suite);
    groups.sort();
    if groups.is_empty() {
        eprintln!("no .testGroup files under {}", suite.display());
        return ExitCode::from(2);
    }

    let mut total_calls = 0usize;
    let mut total_rows = 0usize;
    let mut total_dropped = 0usize;
    println!("| Group | assertSame calls | Rows | Dropped |");
    println!("|---|---|---|---|");
    for path in &groups {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("cannot read {}: {e}", path.display());
                return ExitCode::from(2);
            }
        };
        let source = String::from_utf8_lossy(&bytes);
        let group_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        let extraction = extract_assertions(group_name, &source);
        let calls = count_assert_same(&source);
        let rows = extraction.rows.len();
        let dropped: usize = extraction.blocked.iter().map(|b| b.dropped).sum();
        // The invariant this whole mode exists to preserve: every call is
        // either a row or an accounted-for drop, never neither.
        assert_eq!(
            rows + dropped,
            calls,
            "{}: {rows} rows + {dropped} dropped != {calls} assertSame calls",
            path.display()
        );
        println!("| {group_name} | {calls} | {rows} | {dropped} |");
        total_calls += calls;
        total_rows += rows;
        total_dropped += dropped;

        for blocked in &extraction.blocked {
            println!(
                "    blocked: {}::{} -- {} ({} dropped)",
                group_name, blocked.method, blocked.reason, blocked.dropped
            );
        }
    }
    println!("| **Total** | **{total_calls}** | **{total_rows}** | **{total_dropped}** |");
    println!(
        "{} groups, {total_calls} assertSame calls, {total_rows} rows, {total_dropped} dropped",
        groups.len()
    );
    ExitCode::SUCCESS
}

/// An independent count of `self~assertSame` occurrences, case-insensitive,
/// used only to cross-check `extract_assertions`'s own accounting -- this
/// does not need to understand Rexx, it only needs to count a substring.
fn count_assert_same(source: &str) -> usize {
    let lower = source.to_ascii_lowercase();
    lower.matches("self~assertsame").count()
}
