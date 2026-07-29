//! Task 3.10: parse throughput on the two `.orx` files the Rust build ships
//! and parses at every interpreter start (D2).
//!
//! ## What this measures, and what it does not
//!
//! This times `parse_program` alone, over real files, not synthetic input.
//! It is a component of cold start, not cold start itself: bootstrap
//! execution, heap setup and class construction are not measured anywhere
//! yet, so this number cannot be compared against the ~55 ms cold-start
//! budget on its own. `docs/superpowers/plans/d10-decision.md` records where
//! this figure sits next to the D10 spike's numbers and why the two are not
//! the same measurement -- the spike timed a `chumsky` prototype against a
//! hand-written one on a shared expression grammar over partial input; this
//! times the shipped instruction-and-directive parser over whole files.
//!
//! ## The clone
//!
//! `parse_program` takes `Vec<u8>` by value, because `Program::source` retains
//! the bytes for every node's span. Each timed call therefore needs its own
//! owned buffer. The file is read once, outside every timed region; per
//! sample, `iter_batched` clones that buffer in an UNTIMED setup closure and
//! only the `parse_program` call itself is timed. This is deliberate: the
//! interpreter's own cold-start path reads a file's bytes once and parses
//! them once, so it never pays for a clone at all, and charging one to this
//! benchmark would overstate the cost this measurement is trying to isolate.
//!
//! ## The assertion
//!
//! Phase 2 learned this the hard way: a timing comparison means nothing
//! unless both sides do the same work, and a parser that silently stops early
//! can post a good number. `Program::instructions` is a flat `Vec`, but for
//! both files below nearly all of the content lives inside `::METHOD`,
//! `::ATTRIBUTE` and `::ROUTINE` bodies rather than the main body -- measured,
//! `CoreClasses.orx`'s main body holds 41 instructions against 2,390 nested
//! inside its 347 directives' bodies. Asserting `instructions.len()` alone
//! would let a parser that stopped after the main body, or that dropped every
//! directive body, still post a passing count. So this asserts three node
//! counts together: the main body's instructions, the directive count, and
//! the total instructions nested inside every directive's body -- the same
//! counts `src/directive/tests.rs`'s `core_classes_parses` and
//! `the_other_shipped_packages_parse` pin, reused here rather than
//! re-derived.

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

/// One file to parse and the node counts that must come out of it, pinned
/// against `src/directive/tests.rs` rather than re-derived here.
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
    // 100, 5s measurement) cost more time than this measurement is worth, and
    // matching them is what keeps the numbers comparable with
    // `perf-baseline.md`.
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
                        (program.instructions.len(), program.directives.len(), nested,),
                        (
                            case.main_instructions,
                            case.directives,
                            case.nested_instructions,
                        ),
                        "{} parsed to a different node count; the benchmark and \
                         the acceptance test in src/directive/tests.rs have \
                         diverged, or the parser stopped early",
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
