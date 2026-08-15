# Task 5 report: D1's Phase 4 re-measurement -- the GC arm, rebuilt

Commit: `c445154c` (parent `ad7c36f0`, branch `plan/rust-rewrite`).
Date: 2026-08-09.
Files touched: `rust/crates/rexx-core/benches/heap.rs`, `docs/superpowers/plans/d1-decision.md`.

## Step 1: the recorded 26.5 ms figure is not reproducible

Commit `a3178cff` (2026-07-30) replaced `Body::String(String)` -- the exact variant D1's risk analysis names -- with `Body::Text { bytes: Vec<u8>, num: Option<Result<Box<Number>, NotNumeric>> }`, and added two more variants, `Body::Num` and `Body::Stem`, in the same commit.
`heap.rs`'s bench source was mechanically updated in that commit to keep compiling against the new variant name, but the recorded 26.5 ms figure was never rerun afterward.
So the bench file already "looks rebuilt" (it compiles, it names the current variant) while measuring a different payload than the one that produced 26.5 ms.

Measured with `std::mem::size_of` (a throwaway test file, added and removed, never committed):

| commit | `size_of::<Body>()` | `size_of::<Object>()` |
|---|---:|---:|
| `0fa62ca8` (last commit before `a3178cff`) | 32 bytes | -- |
| `ad7c36f0` (this task's starting commit) | 80 bytes | 88 bytes |

The growth is mostly `Body::Stem` (name, default, tail `HashMap`), not the `Text`/`String` swap itself -- `Text`'s own payload is around 40 bytes against `String`'s 24. A Rust enum's size is its largest variant, so every `Body` value pays `Stem`'s footprint whether or not the graph contains a single stem. `Heap::collect` itself gained no algorithmic change between D1's close (`4d14855b`) and `ad7c36f0` -- confirmed by reading every intervening commit that touches `heap.rs`.

## Step 2: rebuilding the two arms

`build_graph`'s core shape (1,000 arrays of 1,000 distinct strings via `format!("e{j}")`, mirroring `"e" || j`, 10% cross-linked, one root) already matched `heapshape.rex` and was left as-is.

One gap was closed: `heapshape.rex` also builds a `root` directory duplicating references to the same 1,000 arrays under 1,000 more distinct `"K" || i` key strings. `rexx-core` has no `Directory` body variant (`CONDITION('O')` says so explicitly: "not implemented", `rexx-exec/src/builtin/state.rs:1131`), so `build_graph` now approximates it with a plain array holding 1,000 key strings -- matches the C++ side's object *count* (1,001 extra objects on each side) but not its *shape* (a hash directory's bucket walk vs. a flat `Vec` scan). That residual gap cuts in the oracle's favor and is documented, not closed. Measured control: this addition changes nothing detectable at this benchmark's precision (0.1% of the graph).

## Step 3: the measurement

**Oracle** (`( ulimit -v 8388608; LD_LIBRARY_PATH=.../lib .../bin/rexx heapshape.rex )`, fresh directory, 5 runs, all exit 0, all stderr empty): 17.488-18.060 ms, median **17.966 ms**. Same binary as `phase-4d-retention.md` (sha256-verified); the oracle's local uncommitted patch (`NumberStringClass.cpp::copyIfNecessary`) is unrelated to the collector.

**Rust** (`cargo bench --offline -p rexx-core --bench heap -- collection`, two independent runs, n=10 each, machine quiet throughout): 28.733-29.080 ms and 28.523-28.901 ms (no significant difference between the two, p = 0.66). Combined: **28.5-29.1 ms**.

| | pause | ratio |
|---|---|---|
| C++ (median of 5) | 17.966 ms | 1.00x |
| Rust (two runs, n=10 each) | 28.5-29.1 ms | 1.58x-1.66x |

## Result against parity

**Misses parity, and now also misses the Phase 1 debt threshold of 1.5x** that the stale 26.5 ms / 1.45x figure was inside. The gap widened from 1.45x to roughly 1.6x between Phase 1 and Phase 4. Nothing in the collection algorithm or the graph shape explains this; the per-object payload the collector scans grew 2.5x (32 to 80 bytes) for reasons unconnected to the string-representation risk D1 originally named. This is recorded as a genuine miss -- no fix was attempted, per this phase's constraint against optimising anything.

`d1-decision.md`'s pre-registered side byte-arena remains the first thing for 4d-2 to try. This task adds a second candidate to that inheritance: `Body`'s enum size now carries `Stem`'s footprint into every value, in tension with the document's existing note that "boxing the enum variants would make it worse" (that note is about boxing string bytes specifically, not about a cold, large variant like `Stem` sharing an enum with hot ones). Both are left as 4d-2's question.

## Verification

* `cargo fmt --all --check`: exit 0.
* `cargo clippy --workspace --all-targets -- -D warnings`, from a freshly deleted `target/`: exit 0.
* `cargo test --offline --workspace`: all suites green, no failures, exit 0.
* Working tree clean before commit; committed exactly the two intended files.
