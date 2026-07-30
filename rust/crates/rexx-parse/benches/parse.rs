//! Task 3.10: parse throughput on the two `.orx` files the Rust build ships
//! and parses at every interpreter start (D2).
//!
//! ## What this measures, and what it does not
//!
//! This times `parse_program` over real files, not synthetic input, plus the
//! cheap node-count check described below that rides along in the same timed
//! closure. It is a component of cold start, not cold start itself: bootstrap
//! execution, heap setup and class construction are not measured anywhere
//! yet, so this number cannot be compared against the ~55 ms cold-start
//! budget on its own. `docs/superpowers/plans/d10-decision.md` records where
//! this figure sits next to the D10 spike's numbers, in the
//! same-methodology sense only -- matching `sample_size`, warm-up and
//! measurement settings, not a row in `perf-baseline.md`, which has none for
//! `rexx-parse` or either `.orx` file -- and why the two are not the same
//! measurement: the spike timed a `chumsky` prototype against a hand-written
//! one on a shared expression grammar over partial input; this times the
//! shipped instruction-and-directive parser over whole files.
//!
//! ## The clone
//!
//! `parse_program` takes `Vec<u8>` by value, because `Program::source` retains
//! the bytes for every node's span. Each timed call therefore needs its own
//! owned buffer. The file is read once, outside every timed region; per
//! sample, `iter_batched` clones that buffer in an UNTIMED setup closure, so
//! the routine closure's timed work is the `parse_program` call plus the
//! node-count check. Measured separately, the clone itself costs about 1 us
//! on the 141,049-byte file, under 0.1% of the parse -- small enough that
//! including it would barely move the number. It is excluded anyway, on
//! principle rather than because it would distort the result: the
//! interpreter's own cold-start path reads a file's bytes once and parses
//! them once, and never pays for a clone at all.
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
//! counts together every sample: the main body's instructions, the directive
//! count, and the total instructions nested inside every directive's body.
//!
//! Their provenance differs, and the difference matters more than it looks.
//! `src/directive/tests.rs`'s `core_classes_parses` independently pins
//! `directives.len() == 347` for `CoreClasses.orx`, and
//! `the_other_shipped_packages_parse` pins `StreamClasses.orx`'s per-kind
//! counts (7 class, 139 method, 5 attribute, 2 constant), which sum to the
//! 153 used here, though 153 itself is never written there. `main_instructions`
//! (41, 7) and `nested_instructions` (2390, 610) have no such acceptance
//! test anywhere in the tree: they were measured for the first time by this
//! benchmark, against today's parser, and hardcoded here as a change
//! detector rather than an independent pin. A future regression this
//! benchmark and `directive/tests.rs` both happened to miss in the same way
//! would still pass both.
//!
//! ## What the triple does not observe
//!
//! All three counts are flat lengths -- `Program::instructions`, the
//! `Directive` count, and the sum of each `CodeBody::instructions` inside a
//! directive's body -- and `If`, `Do` and `Select` carry target *indices*
//! into these vectors rather than owning nested vectors, so dropping any
//! clause anywhere changes a count. That is what makes the triple worth
//! asserting, and it is also its limit: it cannot see corrupted control-flow
//! wiring (a jump index pointing at the wrong instruction, every count
//! unchanged), a body-boundary bug that moves a clause from one directive's
//! body into the adjacent one (the sum across directives still holds), or
//! anything inside an `Expr`, which no count here inspects. That limit is
//! shared with `directive/tests.rs`'s own acceptance tests rather than
//! introduced by this benchmark, and a benchmark is the wrong place to grow a
//! structural checksum that would close it.

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
///
/// `directives` is cross-checked against `src/directive/tests.rs`'s own
/// acceptance tests (see the module doc). `main_instructions` and
/// `nested_instructions` are not: they are this benchmark's own baseline,
/// first measured here rather than pinned anywhere else.
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
                        (program.main.instructions.len(), program.directives.len(), nested,),
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
