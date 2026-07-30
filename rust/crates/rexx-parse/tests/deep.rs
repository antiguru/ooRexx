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
//! point) -- and far shallower still, on a default 2 MiB thread, which is
//! what `a_shallow_paren_nesting_still_parses_on_a_default_stack_thread`
//! below records. `MAX_EXPR_DEPTH` in `expr.rs` now stops the recursion at
//! 50,000 levels, inside the oracle's own reporting range and well below
//! this parser's measured native cliff, raising the same 11.1 the oracle
//! raises rather than a silent crash.
//!
//! Task 3d: nested calls, `say f(f(...'a'...))`, are a **third** recursion,
//! descending through `arg_list` rather than either of the above, and Task
//! 3c left them unguarded. That mattered more than it looked: measured on
//! the 512 MiB thread, they survived 91,948 levels and aborted at 92,337
//! with no message, while the oracle reports `Error 11.1` from somewhere in
//! [34,500, 34,760] onward -- so the **sized** path, the one the executor
//! runs on, died silently where the reference implementation reported a
//! condition. `arg_list` now spends the same `MAX_EXPR_DEPTH` budget the
//! grouping-paren arm does, which is a correctness requirement and not a
//! tidiness one: see `parens_and_calls_share_one_budget_rather_than_one_each`.
//!
//! Two default-thread numbers this file used to state as 337 and 338 were
//! measured before Task 3c's own counter existed. Adding a counter costs
//! stack, so the shipped cliffs are shallower than the ones it was
//! calibrated against, and both were re-measured on the final code: **331
//! parens parse and 332 aborts; 341 nested calls parse and 342 aborts.**
//! Nested calls are therefore *deeper* than parens, not shallower as Task
//! 3c's report and probe both said.

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
///
/// A different recursion from `deep_paren_program`'s, and that is the whole
/// point: this descends through `arg_list`, which for Task 3c was unguarded
/// while the grouping-paren arm was counted.
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
///
/// A `cargo test` thread's own stack is far smaller, and this parser's native
/// cliff on one is around 331 parens: any test asserting what the counter does
/// at tens of thousands of levels has to run on a sized thread or it measures
/// the small stack instead.
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
///
/// Run on an explicit 512 MiB thread, matching the stack D19 gives
/// `rexx-exec`'s public entry point: on a `cargo test` default 2 MiB thread
/// this parser's own native cliff (331 parens, see the sibling test below)
/// sits far below `MAX_EXPR_DEPTH`, so the counter never gets a chance to
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
///
/// The number was 337/338 before Task 3c's counter existed and this comment
/// went on saying so afterwards, which is the trap worth naming: **a counter
/// costs stack, so adding one makes the unprotected case slightly shallower
/// than the measurement that justified it.** Re-measure this pair whenever
/// the guarded functions change, rather than carrying a figure forward.
///
/// This test parses 300, comfortably below the measured 331-332 cliff,
/// precisely so the suite keeps demonstrating the gap is real without
/// crashing on it. The margin is 31 levels, not the 37 the original number
/// implied.
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
///
/// The failure this closes was not a small-stack embedder's problem, which is
/// why it became a task rather than a deferred note. Measured on the 512 MiB
/// thread the executor actually runs on: `f(f(…'a'…))` survived 91,948 levels
/// and aborted at 92,337 with `SIGABRT` and no message, while `build/bin/rexx`
/// reports `Error 11.1` from somewhere in [34,500, 34,760] onward. So above
/// roughly 92,000 the shipped path died silently where the oracle reported a
/// condition, which is precisely what D19 exists to prevent.
///
/// 100,000 is past that cliff and past `MAX_EXPR_DEPTH`. Before Task 3d this
/// aborted the whole test binary; the counter now answers first.
#[test]
fn a_call_nesting_past_the_native_cliff_raises_11_1_instead_of_aborting() {
    let err = parse_on_a_sized_thread(deep_call_program(100_000))
        .expect_err("100,000 nested calls is past MAX_EXPR_DEPTH");
    assert_eq!((err.code, err.sub), (11, 1));
}

/// Parentheses and calls share **one** depth budget, and this is the test that
/// makes that a requirement rather than a preference.
///
/// With two separate budgets of `MAX_EXPR_DEPTH` each, a program alternating
/// the two recursions gets twice the allowance: 50,000 parens plus 50,000
/// calls is 100,000 levels of real stack, which is past both measured native
/// cliffs (about 89,000 for parens, about 92,000 for calls) and would abort
/// exactly as before. One shared counter is what makes the limit mean "total
/// expression nesting", which is the quantity the stack actually cares about.
///
/// Half the budget in each construct, so neither alone would trip a separate
/// counter and only the shared one can answer.
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
