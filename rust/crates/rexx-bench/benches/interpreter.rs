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

//! Times each `bench-programs/*.rex` file under the interpreter named by
//! `REXX_BENCH_BINARY`. This is Task 0.7: it asserts nothing and gates
//! nothing -- it only establishes the numbers that every later phase's
//! performance gate (D9, Global Constraints) compares against.
//!
//! Run against the C++ oracle with:
//! ```sh
//! REXX_BENCH_BINARY=../build/bin/rexx cargo bench -p rexx-bench -- --save-baseline cpp-linux
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use rexx_bench::{PROGRAMS, interpreter_under_test, program_path, programs_dir};
use std::time::Duration;

fn bench_interpreter(c: &mut Criterion) {
    let interpreter = interpreter_under_test();
    let cwd = programs_dir();

    let mut group = c.benchmark_group("interpreter");
    // Every sample here is a whole process launch costing 0.5-2s (the sizing
    // target from Task 0.7 step 1), so criterion's default sample_size=100
    // would take minutes per benchmark. 10 is criterion's floor; it is still
    // enough for a point estimate and a confidence interval on a
    // launch-dominated measurement.
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(30));

    for name in PROGRAMS {
        let program = program_path(name);
        group.bench_function(*name, |b| {
            b.iter(|| interpreter.run(&program, &[], &cwd).expect("interpreter runs"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_interpreter);
criterion_main!(benches);
