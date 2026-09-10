# Shrinking the hot core

> **For agentic workers:** the four tasks below are ordered. Each ends with a
> measurement and a gate, and each is independently revertible.

**Goal:** cut instructions per clause and the hot code footprint, rather than
shaving individual operations.

**Spec:** `paths-to-parity.md` is the measurement record this argues from.

## Why this and not more micro-work

Every counter says the same thing, and it is not "our operations are slow":

* 1153 instructions per clause against the oracle's 531 -- while our **IPC is
  higher than the oracle's on all ten axes** and our branch misses are lower.
* **L1i misses 202.1M against 58.2M, a 3.47x ratio**, the one counter far out
  of line with the 1.96x instruction ratio.
* The driver is **87% cold** -- 2529 of 2908 instructions never execute -- with
  ~379 hot instructions smeared across 13KB.
* **6.35 ops per clause** (762 ops, 120 clauses in `rexxcps`), at roughly 19
  instructions of dispatch each: ~120 instructions per clause before any work.

The machine runs our code efficiently; there is simply too much of it per
clause. That is also why the last rounds stalled: candidates are worth 1-3%
and **the measured layout band is 4%** on a single axis, so the work was
below its own noise floor. A hot core small enough to sit in L1i makes that
band stop mattering.

## Global constraints

* Byte-identical differential parity on three descriptors. Not negotiable.
* No `unsafe`; the workspace lint is `deny` and per-site approval is Moritz's
  alone. Where a task's cost is attributable to this, say so and stop.
* Commit before gating. Gate is `fmt`, `clippy -D warnings`,
  `cargo test --release --workspace --no-fail-fast`, and
  `REXX_CORPUS_GATE=1 cargo test --workspace --no-fail-fast`.
* **Density work must be measured on `cycles:u` with a pad control**, because
  retired instructions cannot see code that never executes, and a single-axis
  cycle difference under about 4% is not evidence.
* Every task is bounded by an ablation *first*. An ablation that comes back
  small closes the task for a day's work.

---

### Task 1: get `Result`/`?` out of the op loop

**Why:** the 87%-cold driver and the 3.47x L1i ratio are one fact. Every op
returning `Result<Flow, Failure>` creates an unwind edge and drop glue for
every live local; the cold runs are full of `movups` stack shuffling and
`-0x2` drop-flag stores. Nothing about correctness requires the error channel
to be a droppable value threaded through every op.

- [ ] **Bound it.** Build with `panic = "abort"`, which deletes every unwind
  landing pad and nothing else. Record the driver's size, its cold fraction
  from a fresh instruction-sampled profile, and `cycles:u` on the axes.
  This is not a shippable configuration -- `on_interpreter_thread` resumes
  panics across the thread boundary -- it is the ceiling.
- [ ] If the ceiling is small, stop and record it. Otherwise continue.
- [ ] **Design:** ops return a `Copy` `Flow` carrying a `Failed` variant; the
  failure is parked on `Interp` and taken at the boundary that currently
  builds the `Result`. Keep `Result` at the driver's own entry and exit.
- [ ] Convert the hottest arms first, measure, then the rest.
- [ ] Measure `cycles:u` with a pad control, plus `instructions:u`. Gate.

### Task 2: fuse ops

**Why:** 6.35 ops per clause is the tax base, and dispatch is 11.5% of
`rexxcps` self time. Fewer executed ops cuts instructions *and* dispatch, and
compounds with Task 1 rather than competing with it. `ir::compile` already
exists, so this is a compiler change.

- [ ] **Bound it.** Count the dynamic op-pair frequencies over the corpus, not
  the static ones -- the static mix is 762 ops and says nothing about which
  pairs actually run. Take the top pairs and compute the ops removed.
- [ ] Fuse the dominant patterns: `Load`+`Load`+`Binary`+`Store`,
  `Const`+`Binary`, the `TraceRead` pairs.
- [ ] Watch code size: each fused arm adds driver bytes, which Task 1 is
  trying to remove. Measure both.
- [ ] Gate, including `ir_dual` and the golden IR tests, which pin the stream.

### Task 3: stop emitting trace ops defensively

**Why:** **222 of 762 ops in `rexxcps` -- 29% of the stream -- are trace ops**,
for a run that never traces, because the body contains `trace value tracevar`
and `never_retraces` taints the whole body. `chunk_for` already keys on
`ChunkTrace`, so the mechanism exists; the body-wide taint defeats it.

- [ ] **Bound it.** The existing note bounds the direct win at 1.56%, which is
  small -- so the case rests on compounding with Tasks 1 and 2 and on the
  chunk footprint, and that has to be measured rather than assumed.
- [ ] Recompile on a `TRACE` change instead of emitting for both worlds,
  resuming at an instruction boundary (`op_of` provides one).
- [ ] The three named alternatives are in `paths-to-parity.md`; pick with
  measurements, not by preference.

### Task 4: retire the tree-walker

**Why:** now evidence-backed rather than a preference -- the IR beats it on all
ten axes, 0.9% to 85%. Keeping both forces `Op::Generic` to exist as an escape
hatch into the tree executor, doubles the semantic surface, and is why `step`
is a 29KB function next to the driver.

- [ ] **This is a decision, not a task.** `ir_dual` is nearly the only thing
  exercising the tree-walker, and retiring it removes that cross-check. Put
  the trade to Moritz before writing code.
- [ ] If taken: remove `Op::Generic`'s fallback, then `step`, then the engine
  switch, measuring code size at each step.
