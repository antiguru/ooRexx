//! Task 3.10: parse throughput on the two `.orx` files the Rust build ships
//! and parses at every interpreter start (D2).

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rexx_parse::{DirectiveKind, parse_program};
use std::time::Duration;

/// Total instructions inside a directive's own body, 0 for a directive with
/// none (an external method, an abstract method, `::OPTIONS`, ...).
fn body_len(kind: &DirectiveKind) -> usize {
    match kind {
        DirectiveKind::Method(m) => m.body.as_ref().map_or(0, |b| b.instructions.len()),
        DirectiveKind::Attribute(a) => a.body.as_ref().map_or(0, |b| b.instructions.len()),
        DirectiveKind::Routine(r) => r.body.as_ref().map_or(0, |b| b.instructions.len()),
        _ => 0,
    }
}

/// One file to parse and the node counts that must come out of it.
struct Case {
    name: &'static str,
    text: &'static [u8],
    main_instructions: usize,
    directives: usize,
    nested_instructions: usize,
}

const CASES: &[Case] = &[
    Case {
        name: "CoreClasses.orx",
        // 4,193 lines (Task 3.10 brief).
        text: include_bytes!("../../../../interpreter/RexxClasses/CoreClasses.orx"),
        main_instructions: 41,
        directives: 347,
        nested_instructions: 2390,
    },
    Case {
        name: "StreamClasses.orx",
        // 1,010 lines (Task 3.10 brief).
        text: include_bytes!("../../../../interpreter/RexxClasses/StreamClasses.orx"),
        main_instructions: 7,
        directives: 153,
        nested_instructions: 610,
    },
];

fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    // Matches the settings `rexx-bench` and `rexx-num` use (`rust/crates
    // /rexx-bench/benches/interpreter.rs`, `rust/crates/rexx-num/benches
    // /arith.rs`), and for the same reason: criterion's defaults (sample_size
    // 100, 5s measurement) cost more time than this measurement is worth.
    // Matching them keeps this run's methodology comparable with the other
    // benchmarks' -- `perf-baseline.md` has no row for `rexx-parse` or either
    // `.orx` file, so "comparable" means the settings agree, not that this
    // number is checked against a value recorded there.
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(30));

    for case in CASES {
        group.throughput(criterion::Throughput::Bytes(case.text.len() as u64));
        group.bench_function(case.name, |bencher| {
            bencher.iter_batched(
                // Untimed: the owned buffer `parse_program` needs, cloned
                // here rather than inside the timed closure. See the module
                // doc comment for why the clone is deliberately excluded.
                || case.text.to_vec(),
                |text| {
                    let program = parse_program(text).unwrap_or_else(|e| {
                        panic!("{} failed to parse: {e:?}", case.name);
                    });
                    let nested: usize = program.directives.iter().map(|d| body_len(&d.kind)).sum();
                    assert_eq!(
                        (
                            program.main.instructions.len(),
                            program.directives.len(),
                            nested,
                        ),
                        (
                            case.main_instructions,
                            case.directives,
                            case.nested_instructions,
                        ),
                        "{} parsed to a (main, directives, nested) count different \
                         from this benchmark's pinned baseline; directives.len() is \
                         cross-checked against src/directive/tests.rs, but \
                         main_instructions and nested_instructions are this \
                         benchmark's own first measurement, so check which of the \
                         three moved before assuming which side is wrong",
                        case.name,
                    );
                    program
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, parse);
criterion_main!(benches);
