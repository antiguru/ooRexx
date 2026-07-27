//! D9 arithmetic dimension: replays `rust/bench-programs/arith.rex` directly
//! against `Number`, rather than through an interpreter.
//!
//! ## What this does and does not establish
//!
//! `interpreter/arith` (the C++ baseline, `docs/superpowers/plans/perf-baseline.md`)
//! times a Rexx program: parsing, instruction dispatch, variable lookup and
//! arithmetic, all together. This benchmark performs only the arithmetic --
//! the same operators, the same operand values, the same `DIGITS` settings,
//! the same iteration count -- with the loop and operand values inlined as
//! Rust rather than interpreted as Rexx source. There is no lexer, no
//! parser, no bytecode dispatch and no variable-lookup-by-name anywhere in
//! this file.
//!
//! A ratio between this number and the C++ baseline is therefore a
//! statement about the arithmetic engine alone, not about end-to-end
//! interpreter speed -- the Rust interpreter does not exist yet (Phase 2 is
//! `rexx-num` and its neighbours; dispatch and lookup are later phases). Do
//! not read a favourable ratio here as evidence the eventual Rust
//! interpreter will match or beat the C++ one on this program; it says
//! nothing about the cost this benchmark does not pay.
//!
//! ## Operation count
//!
//! `arith.rex` runs 500,000 iterations of:
//!
//! ```text
//! numeric digits 9
//! a = i / 3
//! b = a * a - 1
//! numeric digits 20
//! c = i / 7
//! d = c ** 2 // 5
//! total = total + b + d
//! ```
//!
//! i.e. per iteration: 2 divides, 1 multiply, 1 subtract and 1 power at
//! DIGITS 9 or 20 as shown, 1 remainder-divide, and 2 adds into a running
//! accumulator -- 8 `Number` operations per iteration, 4,000,000 total. `i`
//! is parsed fresh from its decimal string each iteration (there is no
//! integer-valued `Number` constructor to reuse instead), matching the fact
//! that the loop variable is a new value every time; the five literal
//! operands (`3`, `7`, `2`, `5`, `1`) are parsed once outside the loop, since
//! they do not change.

use criterion::{Criterion, criterion_group, criterion_main};
use rexx_num::{DivOp, Number};
use std::hint::black_box;
use std::time::Duration;

const ITERATIONS: u64 = 500_000;

fn arith(c: &mut Criterion) {
    let mut group = c.benchmark_group("arith");
    // Matches the settings `rexx-bench`'s `interpreter` benchmarks use
    // (`rust/crates/rexx-bench/benches/interpreter.rs`), and for the same
    // reason recorded in `perf-baseline.md`: criterion's defaults
    // (sample_size 100, 5s measurement) cost minutes at this program's size.
    // Matching them is also what makes this run's numbers comparable with
    // the recorded C++ baseline.
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("500k_mixed_digits", |bencher| {
        bencher.iter(|| {
            let three = Number::parse("3").unwrap();
            let seven = Number::parse("7").unwrap();
            let two = Number::parse("2").unwrap();
            let five = Number::parse("5").unwrap();
            let one = Number::parse("1").unwrap();
            let mut total = Number::zero();

            for i in 1..=ITERATIONS {
                let i_num = Number::parse(&i.to_string()).unwrap();

                // numeric digits 9
                let a = i_num.div(&three, 9, DivOp::Divide).unwrap();
                let b = a.mul(&a, 9).unwrap().sub(&one, 9).unwrap();

                // numeric digits 20
                let c = i_num.div(&seven, 20, DivOp::Divide).unwrap();
                let d = c.pow(&two, 20).unwrap().div(&five, 20, DivOp::Remainder).unwrap();

                total = total.add(&b, 20).unwrap().add(&d, 20).unwrap();
            }

            black_box(total.format(20))
        })
    });

    group.finish();
}

criterion_group!(benches, arith);
criterion_main!(benches);
