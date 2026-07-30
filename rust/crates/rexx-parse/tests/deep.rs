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

//! Task 3b: a deep `1 + 1 + ... + 1` chain must parse and drop on a normal,
//! default-stack test thread, not just on the wide thread `rexx-run` uses.
//!
//! Before this task, three recursions in this crate scaled with the depth of
//! such a chain rather than with any bound the language imposes: `block.rs`'s
//! `visit_expr` (a hand-written walk run at parse time), `Expr`'s
//! compiler-generated recursive `Drop`, and the gate test harness's own
//! `each_expr` walk (`tests/gate_walk/mod.rs`, shared by `tiling.rs` and
//! `variants.rs`). Measured cliffs on a default 2 MiB thread, before any of
//! the three was fixed: parsing (which ran `visit_expr`) aborted from 2450
//! terms; a tree of the same shape built directly (bypassing the parser, so
//! only `Drop` was exercised) survived to somewhere between 10,000 and
//! 20,000. `rust/corpus/lang/deep_nested_expr.rex`, a 3000-term chain that
//! `build/bin/rexx` evaluates without complaint, sat past the first cliff
//! and is what caught this.
//!
//! `parse_program` itself has no depth limit of its own now: this test's
//! second case, 100,000 terms, is the depth `build/bin/rexx` still answers
//! (it exits 139 at 150,000, which is the oracle's own stack, not a language
//! limit, so this crate is not required to go further and does not claim to).

use rexx_parse::parse_program;

/// Builds `total = 1 + 1 + ... + 1` (`terms` copies of `1`) then `say total`,
/// as one flat, left-associative chain.
fn deep_sum_program(terms: usize) -> Vec<u8> {
    let mut src = String::from("total = 1");
    for _ in 0..terms {
        src.push_str(" + 1");
    }
    src.push_str("\nsay total\n");
    src.into_bytes()
}

/// The exact depth this crate's own throwaway measurement tool,
/// `examples/depth_probe.rs`, found the pre-fix cliff at: 2449 terms parsed
/// and dropped, 2450 aborted. This is comfortably past that on the same
/// default-stack test thread, so a regression in any of the three fixed
/// recursions reintroduces a hang rather than a hard-to-notice slowdown.
#[test]
fn a_chain_past_the_pre_fix_cliff_parses_and_drops() {
    let program = parse_program(deep_sum_program(5_000)).expect("parses");
    drop(program);
}

/// The depth the oracle itself still answers (see this file's own header):
/// `build/bin/rexx` prints a result for a 100,000-term chain and only exits
/// 139 (its own stack overflow) at 150,000. Nothing in this crate should be
/// shallower than the language it reproduces.
#[test]
fn the_depth_the_oracle_still_answers_also_parses_and_drops() {
    let program = parse_program(deep_sum_program(100_000)).expect("parses");
    drop(program);
}
