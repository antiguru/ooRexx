# Item 2: promote an observed TRACE setting into the pool, behind a guard

BASE `7fbadfecb`. Tree clean. Item 2 of
`.superpowers/sdd/queued/2026-09-20-performance-todo.md`; read that file's items 1
and 2, its Kildall section, and everything under "Item 1 landed 2026-09-21",
including the corrections. Item 1's report sections are in that file rather than
on disk.

## What item 1 built, and what it left

Item 1 replaced a per-body `bool` with a per-clause forward dataflow answer over a
`ChunkTrace` lattice. It works and reaches its ceiling exactly where the setting
is a top-level literal: **-40,721,661 ops, -31.91%** on such a program.

**On `rexxcps` it banks nothing**, because a `trace value <expr>` is bottom to any
static analysis. The two facts that matter:

* `rexxcps`' events are at lines 38 (`trace value tracevar`), 76 (`trace value
  trace()`) and 86 (`trace off`), and the timed 1000-clause body is lines **41 to
  83**. The hot body sits after one `trace value` and contains another.
* So no edge-set improvement reaches it. **Only this item can.**

## The sizing, measured rather than inherited

The 3.89% figure in the to-do came from **deleting** the `TRACE` instructions,
which measures what the feature costs and not what a fix recovers. This item's own
sizing was taken separately, and it says the full amount is reachable **on this
program**:

* `rexxcps` writes **zero bytes on stderr** over the whole run (`rc=0`, stdout 252
  bytes), so every one of the 38,461,666 trace ops dispatched was a no-op.
* `tracevar` is assigned once, at line 14, `'Off'`, and only read after. So line
  38 yields `Off` on each of its executions.
* `trace()` under `trace off` answers `O`, and `trace value trace()` leaves it at
  `O` -- measured against the oracle, rc 0, empty stderr. Line 76 re-installs the
  setting already in force.

**All three events yield the same setting every time they run**, so a guarded
promotion makes the whole body `Known` and banks essentially all of it. A program
whose `trace value` answered differently at different times would deoptimise and
bank less; that is the design working, not a failure.

## What to build

Extend item 1's pool so that the value after a `trace value <expr>` can be
`Known(observed)` rather than bottom, with a guard that keeps it true.

* the chunk's key is currently `(BodyKey, ChunkTrace)` on the **entry** setting;
  it has to extend to the observed outcomes the chunk was compiled under;
* `drive.rs:576` already compares `chunk.trace().clause_echoes()` against the
  setting in force and moves the clause echo back to a run-time gate when they
  differ. That is the deoptimisation mechanism; this item gives it more to check;
* the clause echo was deliberately left on the entry setting by item 1. Say in the
  report whether that still holds and why.

## The guard obligation is the whole of the risk

A guard that fails to fire is **a program that silently stops tracing**. Nothing
about that looks like a crash and no gate catches it by accident. So:

**Assert the guard at its accessor, then invert it to prove the assertion is
live.** Under `debug_assertions`, recompute the setting the promotion assumed and
assert it equals the one in force at every promoted clause. Then deliberately
break the promotion and show that assertion reddens. An assertion nobody has
watched fail is not evidence.

Item 1 built exactly this shape for its own analysis: a debug-only per-clause
table the driver checks against the setting actually in force. Extend it rather
than inventing a second mechanism.

## Three things already measured -- do not re-derive them, do not contradict them

1. **`INTERPRET` propagates.** `trace off` then `interpret "trace i"`: the oracle
   traces every clause after it in the enclosing body. `Interpret` drives the pool
   to bottom.
2. **An internal routine does not.** `call zsub` where `zsub` does `trace i`: the
   oracle's whole stderr is `9 *-*   return`.
3. **An instruction can change the setting part-way through its own clause.**
   Under `trace off`, `zg = trace('i') 'tail'` echoes `>L> "tail"` and `>O> " " =>
   "O tail"` under the `I` the call just installed. The analysis answers bottom at
   any event for this reason; a promotion must not undo it.

If you find any of these is wrong, say so with the probe that shows it -- they
were measured against the oracle, not reasoned.

## How it is judged

**The oracle differential is the arbiter.** Trace output moves all three
descriptors and the corpus has programs that trace. Run them.

The gates as `rust/CLAUDE.md` defines them, with the split item 1 found:

    cargo build --workspace --all-targets --release                              # outside the cap
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast # under it
    cargo build --workspace --all-targets                                        # outside
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast           # under it

**`memcap 8G` kills a cold release compile** -- `rustc` peaks over the cap, rc 137,
tests never start, and the last log line is a `Compiling`. A status-only reading
calls that a red gate. Build outside the cap, test under it.

Measurement on the pinned `rust/bench-rexxcps/rexxcps.rex`:

* BASE: 127,886,118 ops dispatched, 21,147,250,696 instructions, 6.39 ops/clause.
* the ceiling: 86,884,257 ops, 20,325,640,274 instructions, 4.34 ops/clause.
* count ops by summing execution counts at the dispatch `jmp *` addresses inside
  every `Interp::run_ops_from` instantiation; the method and the expected
  per-address rows are in `2026-09-20-instructions-per-op.md` and reproduced
  independently by item 1.
* measure the noise band rather than assuming one; item 1 measured 0.0047%.

## Report

Write to `.superpowers/sdd/2026-09-21-trace-quickening-report.md`; if your harness
refuses, return it as text and say so.

Return only: status, commits, the measurement against both BASE and the ceiling
with the fraction reached, gate results, whether the guard's assertion was proved
live by inversion, and any concern.

Quote the command beside every figure and give the exit status you observed. If a
run is still going when you report, say so rather than describing what it will
say. Never amend a commit; a follow-up commit is the answer.
