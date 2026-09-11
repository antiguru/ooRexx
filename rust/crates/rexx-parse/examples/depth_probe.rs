//! Measures, rather than assumes, which of several candidate recursions in
//! `rexx-parse` is what overflows a default-stack thread on a deep
//! `1 + 1 + ... + 1` chain, and how deep each one tolerates.
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
        // `nested_calls_sized` is the mode Task 3d needed and Task 3c did not
        // have. Measuring nested calls only on a default thread made the gap
        // look like a small-stack embedder's problem; on the 512 MiB thread
        // D19 gives `rexx-exec`'s public entry point, the same construct
        // aborted above roughly 92,000, which is the sized path the executor
        // actually runs on.
        "nested_calls" | "nested_calls_sized" => {
            let mut src = String::from("say ");
            src.push_str(&"f(".repeat(n));
            src.push_str("'a'");
            src.push_str(&")".repeat(n));
            src.push('\n');
            let bytes = src.into_bytes();
            let mut builder = std::thread::Builder::new();
            if mode == "nested_calls_sized" {
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
        // `select_nesting` exists because of a probe that measured nothing
        // while appearing to pass. A `select` whose next clause is
        // `otherwise` is **error 7.1, "WHEN or OTHERWISE expected"**, raised
        // at the first `select` before any nesting is built, so a probe using
        // that shape reports a clean parse error at every depth and looks
        // like a pass. `when 1=1 then` is the clause that actually nests.
        // Measured with this mode: 100,000 levels parse cleanly on a default
        // 2 MiB thread, which is what confirms `translate_block`'s `Vec`
        // covers `SELECT` as well as `DO`.
        "select_nesting" => {
            let mut src = String::new();
            for _ in 0..n {
                src.push_str("select\nwhen 1=1 then\n");
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
