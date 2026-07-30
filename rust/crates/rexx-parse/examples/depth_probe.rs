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
        other => panic!("unknown mode {other}"),
    }
}
