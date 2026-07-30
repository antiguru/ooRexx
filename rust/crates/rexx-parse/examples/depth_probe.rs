//! Measures, rather than assumes, which of several candidate recursions in
//! `rexx-parse` is what overflows a default-stack thread on a deep
//! `1 + 1 + ... + 1` chain, and how deep each one tolerates.
//!
//! Built for Task 3b, where the brief's own diagnosis (a compiler-generated
//! recursive `Drop`) turned out to be one of three separate causes rather
//! than the one that actually blocked the corpus test. `mode` isolates one
//! candidate at a time: `parse`/`parse_leak` run the real parser, the
//! `_leak` spelling `std::mem::forget`-ing the result to rule `Drop` in or
//! out; `build_drop`/`build_leak` build an identically-shaped tree directly
//! with `Expr::binary` in a plain loop, with no parser involved, to measure
//! `Drop` alone; `debug`/`eq`/`clone` build the same way and then exercise
//! one derive, to check the neighbours `Drop`'s fix does not touch.
//!
//! Cliffs measured on a default 2 MiB thread, before this task's fixes:
//!
//! | recursion (candidate) | cliff |
//! |---|---|
//! | `block.rs`'s `visit_expr`, run from `add_clause` at parse time | 2,450 terms |
//! | `Expr`'s then-derived, compiler-generated recursive `Drop` | between 10,000 and 20,000 |
//! | `Debug` (`{e:?}`) | between 2,000 and 2,050 |
//! | `PartialEq`/`Clone` (`e == e.clone()`) | between 2,100 and 2,200 |
//!
//! `visit_expr` was the one actually reached first by `parse_program`, found
//! by running the failing test under `gdb` rather than by inference: leaking
//! instead of dropping did not move the cliff, which a `Drop`-only cause
//! would predict, and a hand-built tree survived `Drop` alone to 6-8x that
//! depth. After the fix (`visit_expr` and `Drop` both made iterative;
//! `Debug`/`PartialEq`/`Clone` are unchanged, tracked as a known limit),
//! `parse` on this same probe handles 500,000 terms, well past the 100,000
//! `build/bin/rexx` itself still answers.
//!
//! Extended for Task 3c to measure the *other* candidate D19 named: nested
//! parentheses recurse through `subterm`/`full_subexpression` in `expr.rs`,
//! a different recursion from the flat-chain ones above. `paren_default`/
//! `paren_sized` parse `say (((...('a')...)))` with `n` levels of nesting,
//! on a thread with no explicit `stack_size` (`paren_default`, what
//! `cargo test` gives a test) or an explicit 512 MiB one (`paren_sized`,
//! what `rexx-exec`'s public entry point creates, per D19). Measured before
//! Task 3c's counter existed, this build and machine: `paren_sized` aborted
//! natively between 88,800 and 89,000 parens; `paren_default` aborted far
//! shallower, between 337 and 338. `MAX_PAREN_DEPTH` (`expr.rs`) now stops
//! the recursion at 50,000, inside the oracle's own reporting range
//! (`build/bin/rexx` starts raising `11.1` for the same program between
//! 39,900 and 39,950 parens) and well below the sized cliff -- but not
//! below the default one, which is thousands of levels shallower than the
//! counter itself: a `paren_default` run at any depth past 338 still aborts
//! natively today, counter or not, because there is not enough stack left
//! to reach the check. See Task 3c's report for the full reasoning.
//!
//! `prefix_chain`, `nested_calls` and `nested_do` are Task 3c Step 4's check
//! of other per-source-construct recursions, on a default 2 MiB thread:
//! `nested_do` (`do` nested `n` deep before one `nop` and `n` matching
//! `end`s) parses cleanly to at least 100,000, because `translate_block`
//! tracks open blocks on a heap-allocated `Vec`, not the call stack, and
//! never recurses per nesting level. `prefix_chain` (`- - - - ...1`, unary
//! minus repeated, recursing in `message_subterm`) aborted between 1,150 and
//! 1,200. `nested_calls` (`f(f(f(...'a'...)))`, recursing through `subterm`'s
//! `arg_list` call rather than through the grouping-paren branch `expr.rs`'s
//! counter guards) aborted between 350 and 360 -- shallower than plain
//! parens. Neither is fixed by Task 3c, which is scoped to the grouping-paren
//! recursion D19 measured; both are reported as known gaps of the same
//! shape.
use rexx_parse::{Expr, ExprKind, Operator, parse_program};

fn main() {
    let n: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let mode = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "parse".to_string());

    match mode.as_str() {
        "parse" | "parse_leak" => {
            let mut src = String::from("total = 1");
            for _ in 0..n {
                src.push_str(" + 1");
            }
            src.push_str("\nsay total\n");
            let bytes = src.into_bytes();
            let leak = mode == "parse_leak";
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let p = parse_program(bytes).expect("parses");
                    if leak {
                        std::mem::forget(p);
                    } else {
                        drop(p);
                    }
                    println!("ok: depth {n} {mode}");
                })
                .unwrap();
            handle.join().unwrap();
        }
        // Builds a left-leaning Binary chain directly, with a plain
        // iterative loop and no parser involved at all, to isolate the cost
        // of dropping the tree from the cost of parsing it.
        "build_drop" | "build_leak" => {
            let leak = mode == "build_leak";
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let mut e = Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1);
                    for _ in 0..n {
                        let rhs = Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1);
                        e = Expr::binary(Operator::Plus, e, rhs);
                    }
                    if leak {
                        std::mem::forget(e);
                    } else {
                        drop(e);
                    }
                    println!("ok: depth {n} {mode}");
                })
                .unwrap();
            handle.join().unwrap();
        }
        // Builds one tree the same way `build_drop` does, then exercises one
        // derive on it, to find each derive's own cliff independently of
        // parsing and independently of `Drop` (which is now iterative and
        // is not what these three exercise).
        "debug" | "eq" | "clone" => {
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let mut e = Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1);
                    for _ in 0..n {
                        let rhs = Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1);
                        e = Expr::binary(Operator::Plus, e, rhs);
                    }
                    match mode.as_str() {
                        "debug" => {
                            let text = format!("{e:?}");
                            println!("ok: depth {n} debug, {} chars", text.len());
                        }
                        "eq" => {
                            let clone_for_eq = e.clone();
                            println!("ok: depth {n} eq, equal={}", e == clone_for_eq);
                        }
                        "clone" => {
                            let cloned = e.clone();
                            drop(cloned);
                            println!("ok: depth {n} clone");
                        }
                        _ => unreachable!(),
                    }
                })
                .unwrap();
            handle.join().unwrap();
        }
        // `say (((...('a')...)))` with `n` levels of nesting, to measure the
        // parenthesis-descent recursion in `subterm`/`full_subexpression`
        // rather than the flat-chain recursions above. `paren_sized` uses
        // the 512 MiB stack D19 gives `rexx-exec`'s public entry point;
        // `paren_default` uses no explicit `stack_size`, i.e. what a
        // `cargo test` thread gets, which is the number that decides
        // whether any of this crate's own tests are near the cliff.
        "paren_default" | "paren_sized" => {
            let mut src = String::from("say ");
            src.push_str(&"(".repeat(n));
            src.push_str("'a'");
            src.push_str(&")".repeat(n));
            src.push('\n');
            let bytes = src.into_bytes();
            let mut builder = std::thread::Builder::new();
            if mode == "paren_sized" {
                builder = builder.stack_size(512 * 1024 * 1024);
            }
            let handle = builder
                .spawn(move || match parse_program(bytes) {
                    Ok(p) => {
                        drop(p);
                        println!("ok: depth {n} {mode}, parsed");
                    }
                    Err(e) => println!("ok: depth {n} {mode}, error {e}"),
                })
                .unwrap();
            handle.join().unwrap();
        }
        // Task 3c Step 4 checks: other per-construct recursive descents,
        // measured but not fixed by this task -- see this file's own doc
        // comment and the task's report for the numbers and why they stay
        // unfixed.
        "prefix_chain" => {
            let mut src = String::from("say ");
            src.push_str(&"- ".repeat(n));
            src.push_str("1\n");
            let bytes = src.into_bytes();
            let handle = std::thread::Builder::new()
                .spawn(move || match parse_program(bytes) {
                    Ok(p) => {
                        drop(p);
                        println!("ok: depth {n} {mode}, parsed");
                    }
                    Err(e) => println!("ok: depth {n} {mode}, error {e}"),
                })
                .unwrap();
            handle.join().unwrap();
        }
        "nested_calls" => {
            let mut src = String::from("say ");
            src.push_str(&"f(".repeat(n));
            src.push_str("'a'");
            src.push_str(&")".repeat(n));
            src.push('\n');
            let bytes = src.into_bytes();
            let handle = std::thread::Builder::new()
                .spawn(move || match parse_program(bytes) {
                    Ok(p) => {
                        drop(p);
                        println!("ok: depth {n} {mode}, parsed");
                    }
                    Err(e) => println!("ok: depth {n} {mode}, error {e}"),
                })
                .unwrap();
            handle.join().unwrap();
        }
        "nested_do" => {
            let mut src = String::new();
            for _ in 0..n {
                src.push_str("do\n");
            }
            src.push_str("nop\n");
            for _ in 0..n {
                src.push_str("end\n");
            }
            let bytes = src.into_bytes();
            let handle = std::thread::Builder::new()
                .spawn(move || match parse_program(bytes) {
                    Ok(p) => {
                        drop(p);
                        println!("ok: depth {n} {mode}, parsed");
                    }
                    Err(e) => println!("ok: depth {n} {mode}, error {e}"),
                })
                .unwrap();
            handle.join().unwrap();
        }
        other => panic!("unknown mode {other}"),
    }
}
