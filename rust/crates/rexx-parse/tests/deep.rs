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
//!
//! Task 3c: nested parentheses are a *different* recursion (`subterm`'s
//! descent into `full_subexpression` on `TokenKind::LeftParen`) with its own,
//! much shallower cliff, and here the oracle itself raises a condition
//! rather than crashing -- `build/bin/rexx` starts reporting `Error 11.1`,
//! "Insufficient control stack space", for `say ((((...'a'...))))` somewhere
//! between 39,900 and 39,950 parens. This parser's own recursion, unbounded,
//! aborted with a native stack overflow between 88,800 and 89,000 parens on
//! a 512 MiB thread (the stack size D19 gives `rexx-exec`'s public entry
//! point) -- and far shallower still, around 337 parens, on a default 2 MiB
//! thread, which is what `a_shallow_paren_nesting_still_parses_on_a_default_stack_thread`
//! below records. `MAX_PAREN_DEPTH` in `expr.rs` now stops the recursion at
//! 50,000 levels, inside the oracle's own reporting range and well below
//! this parser's measured native cliff, raising the same 11.1 the oracle
//! raises rather than a silent crash.

use rexx_parse::parse_program;

/// Builds `say (((...('a')...)))` with `depth` levels of parenthesis
/// nesting.
fn deep_paren_program(depth: usize) -> Vec<u8> {
    let mut src = String::from("say ");
    src.push_str(&"(".repeat(depth));
    src.push_str("'a'");
    src.push_str(&")".repeat(depth));
    src.push('\n');
    src.into_bytes()
}

/// 100,000 levels of parenthesis nesting is past both this parser's pre-fix
/// native cliff (measured between 88,800 and 89,000, see this file's header)
/// and `MAX_PAREN_DEPTH` (50,000), so before Task 3c this aborted with a
/// native stack overflow and now raises `11.1` instead -- the same
/// condition, not a coincidence: `build/bin/rexx` raises it too, from
/// somewhere between 39,900 and 39,950 parens onward, so this is parity with
/// the oracle rather than an invented number.
///
/// Run on an explicit 512 MiB thread, matching the stack D19 gives
/// `rexx-exec`'s public entry point: on a `cargo test` default 2 MiB thread
/// this parser's own native cliff (~337 parens, see the sibling test below)
/// sits far below `MAX_PAREN_DEPTH`, so the counter never gets a chance to
/// run and a depth anywhere near 100,000 would abort the whole test binary
/// regardless of whether Task 3c's fix exists.
#[test]
fn a_paren_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let bytes = deep_paren_program(100_000);
    let handle = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || parse_program(bytes))
        .expect("spawns");
    let err = handle.join().expect("the thread itself must not panic");
    let err = err.expect_err("100,000 parens is past MAX_PAREN_DEPTH");
    assert_eq!((err.code, err.sub), (11, 1));
}

/// Documents, rather than fixes, the gap `MAX_PAREN_DEPTH`'s own doc comment
/// warns about: the counter is calibrated against the oracle's cliff and
/// this parser's cliff *on the 512 MiB thread*, not against a `cargo test`
/// default 2 MiB thread's far shallower one. 338 parens already aborts a
/// default-stack thread natively (measured, `examples/depth_probe.rs`'s
/// `paren_default` mode: 337 parses, 338 aborts), thousands of levels below
/// where `MAX_PAREN_DEPTH` would ever raise `11.1`. So a caller on a small
/// thread is not protected by the counter for depths between this parser's
/// own native cliff and `MAX_PAREN_DEPTH` -- only a sized thread is. This
/// test parses 300, comfortably below the measured 337-338 cliff, precisely
/// so the suite keeps demonstrating the gap is real without crashing on it.
#[test]
fn a_shallow_paren_nesting_still_parses_on_a_default_stack_thread() {
    let program = parse_program(deep_paren_program(300)).expect("parses");
    drop(program);
}

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
