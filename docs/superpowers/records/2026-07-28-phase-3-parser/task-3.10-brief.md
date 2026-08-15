## Task 3.10: Parse throughput on the bootstrap files

**Files:**
- Create: `rust/crates/rexx-parse/benches/parse.rs`
- Modify: `rust/crates/rexx-parse/Cargo.toml`

**Interfaces:**
- Consumes: the whole parser.

- [ ] **Step 1: Add criterion**

`criterion = "0.8.2"`, `[[bench]] name = "parse" harness = false`. Match the
settings the existing benchmarks use — `sample_size(10)`, 500 ms warmup, 30 s
ceiling — or the numbers are not comparable with `perf-baseline.md`.

- [ ] **Step 2: Benchmark the real files, not synthetic input**

```rust
// interpreter/RexxClasses/CoreClasses.orx   4,193 lines
// interpreter/RexxClasses/StreamClasses.orx 1,010 lines
```

- [ ] **Step 3: Assert the parse succeeded inside the benchmark**

Phase 2 learned this the hard way: a timing comparison means nothing unless
both sides do the same work. Assert a node count, so a parser that silently
stops early cannot post a good number.

- [ ] **Step 4: Record the number against the cold-start budget**

C++ cold start is 5.1 ms from a memory-mapped image; the budget is ~55 ms
total. Write the measurement into `d10-decision.md` and say plainly whether it fits.

**Report parse time as a component of cold start, not as cold start.** Under D2 the
Rust build parses these 5,203 lines at every interpreter start, so parse time is *in*
the budget — but bootstrap execution, heap setup and class construction are also in
it, and none is measured yet. The parent plan already had to correct D10 for claiming
parser throughput "sets cold-start time directly". So a 1 ms parse does **not** mean
the budget fits; it means one component of it costs 1 ms. State the number, state
what it excludes, and do not draw the conclusion the data cannot support.

- [ ] **Step 5: Commit**

---

