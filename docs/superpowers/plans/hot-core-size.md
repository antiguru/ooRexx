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

- [x] **Bounded, and the task is CLOSED on the goal axis.** See below.
- [x] The ceiling is small for `rexxcps`. Stopped. The `Flow`/`Failed`
  redesign is not built.

**Two ablations, because the first bounded the wrong mechanism.**

`panic = "abort"` deletes every unwind landing pad. It shrank the *driver* by
**241 bytes of 15,278**, which falsified the premise this task was written on:
the driver's cold bulk is not unwind machinery. The disassembly has **8**
`_Unwind_Resume` edges against **70 of 163 calls being error-shaped** -- it is
ordinary `?` returns, their argument setup and their early-return cleanup.

The second ablation is the real ceiling: every `Loud` constructor made
divergent (`#[inline(always)] -> Loud { std::process::abort() }`), so LLVM
kills each caller's argument setup and collapses every error path in every arm
to one call. Nothing a real implementation does can beat that. Measured
against the band:

| axis | ins | cyc | L1i | cycles vs band |
| --- | ---: | ---: | ---: | --- |
| `rexxcps` | 0.9908 | 0.9854 | 0.9112 | **inside -- not established** |
| `dispatch` | 0.9802 | 0.8519 | 0.1148 | outside by 11.2 pp |
| `emptyloop` | 1.0212 | 1.0146 | 1.0205 | inside |

**0.9% instructions on the goal axis, and nothing established on cycles.**
The send axis has a real 14.8% behind it, but `dispatch` is already 1.27x and
is not the goal.

**`panic = "abort"` as a candidate in its own right: withdrawn.** It first
read `rexxcps` -5.2% cycles and `dispatch` -11.5%, which looked like a
one-line win (Cargo forces `unwind` for test profiles, so the gate is
unaffected). But it removes ~200KB, five times what the layout band sampled,
so it was re-measured with a pad restoring the binary to 2,784,501 bytes
against the original 2,782,832:

| | `rexxcps` cyc | `dispatch` cyc | `emptyloop` ins |
| --- | ---: | ---: | ---: |
| `abort`, smaller binary | 0.9478 | 0.8853 | 1.0423 |
| `abort`, **padded to size** | 0.9882 | 0.8839 | 1.0423 |
| band | 0.9749-1.0000 | 0.9642-1.0231 | -- |

`rexxcps`'s -5.2% was **total-size layout** and is withdrawn. `dispatch`'s win
survives the control unchanged -- it is the removal of landing pads
*interleaved near hot code*, not the binary being smaller -- and `emptyloop`'s
**+4.23% instructions** is robust across both arms and unexplained.

**The finding that outlives the task:** the footprint that matters is the
whole binary's, not one function's. `panic = "abort"` moved the driver 241
bytes and won -11.5% on `dispatch` anyway, because `step` lost 2,722 bytes and
the text lost 199,587.

### Task 2: fuse ops

**Why:** 6.35 ops per clause is the tax base, and dispatch is 11.5% of
`rexxcps` self time. Fewer executed ops cuts instructions *and* dispatch, and
compounds with Task 1 rather than competing with it. `ir::compile` already
exists, so this is a compiler change.

- [x] **Bounded, and the task is CLOSED. Op count is not the tax base.**

Measured with per-pc execution counters (`rexxcps` compiles to one chunk --
`subroutine:` and `novalue:` are labels in the main body -- so counts are exact
and adjacency comes free from the stream order): **1,273,858 ops for 168,910
promoted clauses, 7.30 ops per clause.**

The dominant adjacency is real: `Condition` and `JumpUnless` execute 64,601
times each and are always adjacent, as are `TraceRead`/`Load` at 86,853 and
`TraceArgument`/`PushArg` at 47,611. Fusing the best of them removes 5.1% of
executed ops.

**But Task 3's ablation shows what an op is worth.** Removing **32.8% of all
executed ops bought 1.73% of instructions** -- about 8.6 instructions per
trace op. At that rate the best fusion available is worth roughly 0.3%, and
each fused arm *adds* driver bytes. Not built.

**The instrument needed fixing twice, which is the transferable part.** The
first per-pc counter reported "1.22 ops per clause, 82% `Clause` ops" --
impossible for a register machine. `Op::Clause` names a *region* whose ops run
in an inner `ops_in` loop that the counter never saw, so `Load`, `Binary` and
`Store` all read as zero. A probe that runs, exits 0, and cannot see its
subject.

### Task 3: stop emitting trace ops defensively

**Why:** **222 of 762 ops in `rexxcps` -- 29% of the stream -- are trace ops**,
for a run that never traces, because the body contains `trace value tracevar`
and `never_retraces` taints the whole body. `chunk_for` already keys on
`ChunkTrace`, so the mechanism exists; the body-wide taint defeats it.

- [x] **Bounded at 1.73% instructions on `rexxcps`.** The older note's 1.56%
  was right and the suspicion that it understated things was wrong.

Ablated by dropping the defensive half of the gate, `echoes_values =
trace.intermediates()`. `rexxcps` output byte-identical, as it must be: the
program sets `trace value tracevar` with `tracevar='Off'` and never traces.

| axis | ins | cyc |
| --- | ---: | ---: |
| `rexxcps` | **0.9827** | 0.9516 |
| `dispatch` | 1.0000 | 0.9478 |
| `strings` | 1.0000 | 0.9576 |
| `emptyloop` | 1.0000 | 1.0016 |
| `varlookup` | 1.0000 | 1.0028 |

**Four axes at exactly 1.0000 instructions are the control**: none contains a
`TRACE` instruction, so the ablation cannot reach them, and it does not.

**The cycle column is not evidence, and it shows the band is per-change-site.**
`dispatch` and `strings` move -5.2% and -4.2% on cycles with instruction counts
*exactly* unchanged -- no mechanism reaches them, so that is layout, and it
lands outside the band measured with pads in `drive.rs`. A one-line edit in
`compile.rs` moves axes further than 39KB of dead code did. **A pad control
bounds layout for the site it was measured at, not for the binary.**

- [ ] Whether 1.73% on the goal axis justifies recompiling on a `TRACE` change
  is Moritz's call; the mechanism and the three alternatives are in
  `paths-to-parity.md`.

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
