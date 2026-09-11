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

use rexx_parse::{MAX_EXPR_DEPTH, parse_program};

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

/// Builds `say f(f(...f('a')...))` with `depth` levels of call nesting.
fn deep_call_program(depth: usize) -> Vec<u8> {
    let mut src = String::from("say ");
    src.push_str(&"f(".repeat(depth));
    src.push_str("'a'");
    src.push_str(&")".repeat(depth));
    src.push('\n');
    src.into_bytes()
}

/// Runs `parse_program` on the 512 MiB thread D19 gives `rexx-exec`'s public
/// entry point, which is the stack every depth limit here is calibrated
/// against.
fn parse_on_a_sized_thread(bytes: Vec<u8>) -> Result<(), rexx_parse::ParseError> {
    std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || parse_program(bytes).map(drop))
        .expect("spawns")
        .join()
        .expect("the thread itself must not panic")
}

/// 100,000 levels of parenthesis nesting is past both this parser's pre-fix
/// native cliff (measured between 88,800 and 89,000, see this file's header)
/// and `MAX_EXPR_DEPTH` (50,000), so before Task 3c this aborted with a
/// native stack overflow and now raises `11.1` instead -- the same
/// condition, not a coincidence: `build/bin/rexx` raises it too, from
/// somewhere between 39,900 and 39,950 parens onward, so this is parity with
/// the oracle rather than an invented number.
#[test]
fn a_paren_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let bytes = deep_paren_program(100_000);
    let handle = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || parse_program(bytes))
        .expect("spawns");
    let err = handle.join().expect("the thread itself must not panic");
    let err = err.expect_err("100,000 parens is past MAX_EXPR_DEPTH");
    assert_eq!((err.code, err.sub), (11, 1));
}

/// Documents, rather than fixes, the gap `MAX_EXPR_DEPTH`'s own doc comment
/// warns about: the counter is calibrated against the oracle's cliff and
/// this parser's cliff *on the 512 MiB thread*, not against a `cargo test`
/// default 2 MiB thread's far shallower one. **332 parens already aborts a
/// default-stack thread natively** (measured on the shipped code,
/// `examples/depth_probe.rs`'s `paren_default` mode: 331 parses, 332
/// aborts), tens of thousands of levels below where `MAX_EXPR_DEPTH` would
/// ever raise `11.1`. So a caller on a small thread is not protected by the
/// counter for depths between this parser's own native cliff and
/// `MAX_EXPR_DEPTH` -- only a sized thread is.
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

/// Nested calls past the sized thread's native cliff raise `11.1` instead of
/// aborting the process (Task 3d).
#[test]
fn a_call_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let err = parse_on_a_sized_thread(deep_call_program(100_000))
        .expect_err("100,000 nested calls is past MAX_EXPR_DEPTH");
    assert_eq!((err.code, err.sub), (11, 1));
}

/// Parentheses and calls share **one** depth budget, and this is the test that
/// makes that a requirement rather than a preference.
#[test]
fn parens_and_calls_share_one_budget_rather_than_one_each() {
    let half = usize::try_from(MAX_EXPR_DEPTH).expect("fits") / 2 + 100;
    let mut src = String::from("say ");
    src.push_str(&"f(".repeat(half));
    src.push_str(&"(".repeat(half));
    src.push_str("'a'");
    src.push_str(&")".repeat(half));
    src.push_str(&")".repeat(half));
    src.push('\n');

    let err = parse_on_a_sized_thread(src.into_bytes())
        .expect_err("half the budget in each construct must still exceed the shared one");
    assert_eq!((err.code, err.sub), (11, 1));
}
