# D1 — heap representation: the measurement

**Verdict: D1(a) holds. Arena with tagged, generation-checked index handles.
Proceed to Phase 2.**

Measured 2026-07-27 on Linux (see `perf-baseline.md` for the machine).

## The one number that is like-for-like

Full-GC pause over a ~1M-object graph of identical shape — 1,000 arrays of
1,000 distinct strings, 10% cross-linked so it is not a pure tree, all reachable
from one root.

| | Pause | Ratio |
|---|---|---|
| C++ (`GC('F')`, 5 runs) | 17.8 – 18.7 ms, median **18.2 ms** | 1.00× |
| Rust (`Heap::collect`, criterion, n=10) | **26.5 ms** [26.3, 26.6] | **1.45×** |

**Inside the Phase 1 threshold of 1.5×, and outside parity.** Per Global
Constraints this exits Phase 1 as a *recorded debt*, not a clean pass: the
parity gate applies from Phase 2 onward, and this must be re-measured at Phase 4
when a real interpreter exists to measure on equal footing.

This comparison is genuinely like-for-like. `GC('F')` calls
`memoryObject.collectAndUninit` (`BuiltinFunctions.cpp:3031`), a real full
collection, and the Rust side times only `collect` — `iter_batched` keeps graph
construction out of the measurement.

## The numbers that are not like-for-like

| Rust | |
|---|---|
| 1M `Body::String` allocations | 84.3 ms [84.1, 84.5] |
| 1M `Body::Array` of 4 handles | 74.8 ms [74.7, 75.0] |
| C++ building the same 1M strings + 1,000 arrays | 254 ms |

**Do not read 254 vs 84 as a 3× Rust win.** The C++ figure is an interpreted
Rexx loop and pays for parsing, dispatch, and variable lookup that the Rust
microbenchmark never touches. The Rust figure is also not purely arena cost —
`format!("e{j}")` per iteration is a meaningful share of it. These are recorded
for shape, not for adjudication.

## What this does not settle

The pause is 1.45× and the string-representation risk pre-registered in D1
remains **untested**: `Body::String(String)` is a fixed slot plus a separate heap
buffer, where C++ stores bytes inline with the header via `char stringData[4]`.
Two allocations against one, for the most common object in a string-dominated
language.

That is the obvious candidate for the 45%, and the fix is already specified — a
side byte-arena indexed by `(offset, len)`, which works because `RexxString` is
immutable. **It was not built, because the gate did not require it.** If the
Phase 4 re-measurement misses parity, this is the first thing to try, and the
plan says explicitly that boxing the enum variants would make it *worse*.

## Reproducing

```sh
# C++
build/bin/rexx rust/bench-programs/heapshape.rex

# Rust
cd rust && cargo bench --offline -p rexx-core --bench heap
```

Note that `heapshape.rex` builds its strings with `"e" || j` rather than a bare
literal. A literal is a single interned object shared by all 1M slots, which
collapses the graph to ~1,001 objects and reports a 0.6 ms pause. The first
version of this measurement did exactly that and had to be redone.
