//! D1 measurement: allocation throughput and full-GC pause.
//!
//! The graph shape mirrors `rust/bench-programs/heapshape.rex` so the pause
//! figure is comparable with the C++ one: 1,000 arrays of 1,000 distinct
//! strings, 10% cross-linked so the graph is not a pure tree, all reachable
//! from one root.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rexx_core::{Body, Heap, ObjRef, RootSet};
use std::hint::black_box;

const OUTER: usize = 1_000;
const INNER: usize = 1_000;

/// Builds the same graph the Rexx program builds.
fn build_graph() -> (Heap, RootSet) {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let mut outer = Vec::with_capacity(OUTER);

    for _ in 0..OUTER {
        let mut elems = Vec::with_capacity(INNER);
        for j in 0..INNER {
            // A distinct string per slot, as in the Rexx side. A shared
            // constant would collapse the graph to ~1,001 objects and make
            // the pause meaningless.
            elems.push(heap.alloc(Body::String(format!("e{j}"))));
        }
        outer.push(heap.alloc(Body::Array(elems)));
    }

    for i in 0..(OUTER / 10) {
        let target = outer[OUTER - 1 - i];
        if let Some(obj) = heap.get_mut(outer[i])
            && let Body::Array(items) = &mut obj.body
        {
            items[0] = target;
        }
    }

    let root = heap.alloc(Body::Array(outer));
    roots.add_global(".ROOT", root);
    (heap, roots)
}

fn allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation");
    group.sample_size(10);

    group.bench_function("1m_strings", |b| {
        b.iter(|| {
            let mut heap = Heap::new();
            for j in 0..1_000_000usize {
                heap.alloc(Body::String(format!("e{j}")));
            }
            black_box(heap.live_count())
        })
    });

    group.bench_function("1m_arrays_of_4", |b| {
        b.iter(|| {
            let mut heap = Heap::new();
            for _ in 0..1_000_000usize {
                heap.alloc(Body::Array(vec![ObjRef::NIL; 4]));
            }
            black_box(heap.live_count())
        })
    });

    group.finish();
}

fn collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("collection");
    group.sample_size(10);

    // Setup is excluded from the timing: only `collect` is measured, which is
    // what the C++ `GC('F')` figure measures too.
    group.bench_function("full_gc_1m_graph", |b| {
        b.iter_batched(
            build_graph,
            |(mut heap, roots)| {
                let stats = heap.collect(&roots);
                black_box(stats.live)
            },
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

criterion_group!(benches, allocation, collection);
criterion_main!(benches);
