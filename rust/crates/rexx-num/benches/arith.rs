//! D9 arithmetic dimension: replays `rust/bench-programs/arith.rex` directly
//! against `Number`, rather than through an interpreter.
//! ```text
//! numeric digits 9
//! a = i / 3
//! b = a * a - 1
//! numeric digits 20
//! c = i / 7
//! d = c ** 2 // 5
//! total = total + b + d
//! ```

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
                let d = c
                    .pow(&two, 20)
                    .unwrap()
                    .div(&five, 20, DivOp::Remainder)
                    .unwrap();

                total = total.add(&b, 20).unwrap().add(&d, 20).unwrap();
            }

            // A timing comparison between two implementations is worth
            // nothing unless they compute the same thing. `arith.rex` under
            // `build/bin/rexx` prints exactly this, so a divergence here
            // means the benchmark has stopped replaying the program it
            // claims to and its number should not be compared with the C++
            // baseline. The check costs one string compare per sample.
            let result = total.format(20);
            assert_eq!(
                result, "4629643519330627.7808",
                "benchmark no longer matches arith.rex"
            );
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(benches, arith);
criterion_main!(benches);
