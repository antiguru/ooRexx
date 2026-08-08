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

---

## Phase 2 addendum — arithmetic at 1.22×, recorded as debt (2026-07-28)

Phase 2's exit gate asks for **parity** on the `arith` benchmark. It is not
met. The number is recorded here rather than in the phase plan because the
cause is representation, which is D1's subject.

| | mean | gap |
|---|---|---|
| C++ `interpreter/arith`, re-measured | 1.1546–1.1595 s | — |
| Rust, as first written | 1.9666 s | 1.70× |
| Rust, after the division rewrite | 1.4067 s | **1.22×** |

The division rewrite (`ec5f5626`) replaced repeated-subtraction quotient
digits with the estimating algorithm the interpreter itself uses, which was
already in the tree from the `dividePower` port. That was worth −28.5% on its
own because one function held 48.2% of the time.

**What the remaining 1.22× is made of.** The profile after the rewrite is
flat: `long_divide` 26.3%, `mul_magnitudes` 18.8%, and roughly 15% in
`malloc`/`free`/`memmove`. No single function to fix. Closing an 18% gap from
here means attacking the digit-per-`u8` representation and the fresh `Vec`
per result — the same class of change this document already pre-registers for
strings, and for the same reason: one allocation per value where the C++ has
none.

**Why stopping here is defensible, and where it is not.** The comparison
flatters the Rust side. It times arithmetic alone against a C++ figure that
includes parsing, dispatch and variable lookup, because at Phase 2 there is
no Rust interpreter to measure — the parser is Phase 3 and dispatch is Phase
4. So 1.22× is a **lower bound** on the real gap, and it will get worse, not
better, once the Rust side starts paying those costs. Do not read this entry
as "nearly at parity".

**Re-measure at Phase 4**, alongside the string-representation question above.
That is when a like-for-like comparison first exists, and it is already what
this document schedules for the heap number.

---

## Phase 4 addendum -- the recorded 26.5 ms is not reproducible, and the re-measurement misses the Phase 1 threshold too (2026-08-09, starting from commit `ad7c36f0`)

### Step 1: Why re-running the old bench does not answer the question asked

Commit `a3178cff` (2026-07-30, "Add the executor's value bodies and root-set slot frames") replaced `Body::String(String)` -- the exact variant this document's risk analysis names above -- with `Body::Text { bytes: Vec<u8>, num: Option<Result<Box<Number>, NotNumeric>> }`.
The same commit added two more variants, `Body::Num` and `Body::Stem`.
`heap.rs`'s bench source was mechanically updated in that same commit to keep building against the new variant name, but the recorded 26.5 ms figure was never rerun afterward.
So the number in the table above and the code that produces it have described two different representations since 2026-07-30.

`Body`'s size grew because of this, measured with `std::mem::size_of`:

| commit | `size_of::<Body>()` | `size_of::<Object>()` |
|---|---:|---:|
| `0fa62ca8` (last commit before `a3178cff`) | 32 bytes | -- |
| `ad7c36f0` (this task's starting commit) | 80 bytes | 88 bytes |

The growth is not mostly the `Text`/`String` swap itself.
`Text`'s own payload (`bytes: Vec<u8>` at 24 bytes, `num` measured at 16 bytes) is around 40 bytes against `String`'s 24.
The dominant term is `Body::Stem`, added in the same commit for an unrelated value kind: a stem's name, default, and tail `HashMap`.
A Rust enum's size is its largest variant, so every `Body` value, including one that only ever holds text, now carries the footprint `Stem` needs, whether or not the graph being collected contains a single stem.
That is the trap this task was written around: the bench file already "looks rebuilt" because it compiles and names the current variant, but the per-object payload the collector scans changed under it without the recorded number moving.

So Step 1's conclusion, stated plainly: **the recorded 26.5 ms figure is not reproducible.**
Re-running `cargo bench` today measures a heap whose objects are, on average, materially larger than the ones the 26.5 ms figure describes, for reasons that have nothing to do with the collection algorithm.
`Heap::collect` gained no algorithmic change between `4d14855b`, when D1 was closed, and `ad7c36f0` -- confirmed by reading every commit touching `heap.rs` in between.
A fresh number is measured below, but it is compared to the old one with this caveat attached, not quoted against it as a rerun of the same instrument.

### Step 2: Rebuilding the two arms to be like-for-like

The graph shape `build_graph` builds already matched `heapshape.rex`'s core structure and was left as-is: 1,000 arrays of 1,000 distinct strings built via `format!("e{j}")`, mirroring `"e" || j` and preserved deliberately (see the note under "Reproducing" above about why a bare literal collapses the graph), 10% cross-linked, reachable from one root.

One gap was closed.
`heapshape.rex` also builds `root = .directory~new` and sets `root["K" || i] = a` for each of the 1,000 arrays -- the same arrays `outer` already reaches, but under 1,000 more distinct key strings ("K" concatenated with the index, same reasoning as `"e" || j`).
`rexx-core` has no `Directory` body variant to build the equivalent of `root` with.
`CONDITION('O')`, the one place this crate names the type, says so directly: "CONDITION option \"O\" answers a Directory, which is not implemented" (`rexx-exec/src/builtin/state.rs:1131`).
`build_graph` now approximates it with a plain array holding the 1,000 key strings (`root_keys` in `heap.rs`), which matches the C++ side's object *count* -- 1,001 extra objects, one container plus 1,000 keys, on both sides -- but not its *shape*.
Marking a `Directory` walks a hash table's buckets for those 1,000 entries; marking this array walks a contiguous `Vec`, which is cheaper per entry.
That residual difference cuts in the oracle's favor, understating the true C++ cost rather than overstating it, and it is not closed here.

Measured, this 1,001-object addition (0.1% of a roughly 1,002,002-object graph) has no detectable effect at this benchmark's precision.
A control run of the pre-addition `build_graph` (same commit, same `Body::Text` representation, just missing `root_keys`) read 30.072-30.752 ms, inside the same run-to-run spread as the two post-addition runs recorded below rather than reliably above or below them.

### Step 3: The measurement, against parity and against the Phase 1 threshold

Machine quiet: load average 0.3-0.7 on 32 cores throughout, no competing `cargo`/`criterion`/`rexx` process at any point during the runs below.

**Oracle** (`( ulimit -v 8388608; LD_LIBRARY_PATH=.../lib .../bin/rexx heapshape.rex )`, run from a fresh empty directory, 5 runs, every run exit 0, every stderr empty): `gc_pause_seconds` 0.017488-0.018060 s, median **17.966 ms**.
Binary identity: `bin/rexx` sha256 `bb5bb8cc...a13019`, `lib/librexx.so.4` sha256 `42136c40...6d9b6fb` -- the same binary `phase-4d-retention.md` recorded.
The local, uncommitted oracle patch on record in `phase-4-exclusions.txt` (`NumberStringClass.cpp`'s `copyIfNecessary`) touches number-to-string caching, not the collector, so it has no bearing on this figure.
This number changed negligibly from the recorded 17.8-18.7 ms / median 18.2 ms, as expected: no C++ code changed for this task.

**Rust** (`cargo bench --offline -p rexx-core --bench heap -- collection`, `collection/full_gc_1m_graph`, `iter_batched` excludes `build_graph` from the timing, exit 0 both times, n=10 each): two independent runs read **28.733-29.080 ms** and **28.523-28.901 ms**.
Criterion reports no significant difference between them (p = 0.66).
Combined range **28.5-29.1 ms**, midpoint about 28.8 ms.

| | pause | ratio |
|---|---|---|
| C++ (median of 5) | 17.966 ms | 1.00x |
| Rust (two runs, n=10 each) | 28.5-29.1 ms | 1.58x-1.66x |

**This misses parity, and it now also misses the Phase 1 threshold of 1.5x that the stale 26.5 ms / 1.45x figure was inside.**
The gap widened between Phase 1 and Phase 4 -- from 1.45x to roughly 1.6x -- and Step 1 above accounts for essentially all of it.
The collection algorithm is unchanged, the graph shape is unchanged net of the 0.1%, immaterial addition in Step 2, and the per-object payload the collector scans grew 2.5x, from 32 to 80 bytes, for reasons unconnected to the string-representation risk this document originally named.
Read this as a genuine miss, not a number massaged to clear a bar: the pre-registered fix below was not attempted, and this task does not attempt it either, per this phase's constraint against optimising anything.

**What this adds to the pre-registered fix, for 4d-2.**
The side byte-arena indexed by `(offset, len)` above remains the first thing to try for the double-allocation risk this document originally named, and this task does not build it.
This addendum adds a second, distinct candidate that 4d-2 inherits as live input rather than as a conclusion drawn here: `Body`'s enum size now carries `Stem`'s footprint into every value, including ones that are never a stem.
That sits in tension with the existing note that "boxing the enum variants would make it worse" -- that note is about boxing string bytes specifically, where the byte-arena alternative addresses exactly that without adding a pointer chase to the hot string path, not about whether a large, cold variant like `Stem` should stay inline in the same enum as the hot ones.
Whether shrinking `Body` by boxing only `Stem`'s payload is worth it, or falls into the same trap the existing note warns about, is 4d-2's question, not this task's.

### Reproducing this addendum

```sh
# C++ (from a fresh empty directory, absolute path to the file)
( ulimit -v 8388608; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /path/to/heapshape.rex )

# Rust
cd rust && cargo bench --offline -p rexx-core --bench heap -- collection
```
