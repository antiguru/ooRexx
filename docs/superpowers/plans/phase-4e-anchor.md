# Phase 4e anchor -- the tree-walker's benchmark figures, before the IR exists

Task 1 of Phase 4e (`docs/superpowers/plans/2026-08-09-phase-4e-ir.md`).
The `spike/bytecode-vm` figures this phase's tasks used to cite are withdrawn: no bench-results
file, no harness script, no build identity, and no empty-loop program ever existed in
`rust/bench-programs/`.
Every task from Task 4 onward states a predicted axis movement before measuring, and a prediction
is not falsifiable against a number nobody can reproduce.
This document is that number.

**These are the tree-walker's own figures, and only the tree-walker's.**
At the commit measured below there is exactly one engine in `rexx-run`: `Engine::TreeWalker`, the
only variant that exists before Task 3 lands `Engine::Ir`.
Once Task 3 lands, `rexx-run` carries two arms, and a table that says only "the ratio" without
naming which arm stops having a referent -- Task 3 onward must say tree-walker-vs-oracle,
IR-vs-oracle, or IR-vs-tree-walker explicitly, because after this document all three exist.
Every ratio below is tree-walker-vs-oracle.

**No optimisation was attempted and none is proposed here.**
This task's entire content is building the anchor: one new benchmark axis (`emptyloop.rex`) and one
measurement of it alongside the six existing loop axes, run once, interleaved, on the unmodified
tree-walker.

## What's new here versus the existing baseline

`docs/superpowers/plans/perf-baseline.md` already holds Phase 4d-1 measurements of `alloc4c`,
`arith`, `compound`, `strings` and `varlookup` against the oracle, most recently re-measured at
`d233d1e9`.
This document adds exactly one axis those did not have: `emptyloop`, "the clause-dispatch floor: a
loop whose body does nothing" (`rust/bench-programs/emptyloop.rex`'s own header).
Task 4 is the first task that promotes anything (`Do`/`Loop`), and its own text names `emptyloop`
and `varlookup` as the two axes that promotion touches, so `emptyloop` not existing was a gap this
plan's own next task would have hit immediately.

This document is also a fresh measurement of the other five axes rather than a copy of
`perf-baseline.md`'s numbers, because "benchmark comparisons interleave between arms within one
sitting" (Global Constraints) and this task's `emptyloop` sample has to sit in the same sitting as
the axes it is compared against for spread and ratio purposes.
The five pre-existing axes' numbers here differ from `perf-baseline.md`'s own `d233d1e9` figures by
varying amounts.
Two of the five moved enough that this document cannot wave the difference away as "run-to-run
variation" without checking; "Movement against `perf-baseline.md`'s `d233d1e9` figures" below
states what moved and what this document can and cannot say about why.

## The new axis: `emptyloop.rex`

```rexx
/* The clause-dispatch floor: a loop whose body does nothing, so the
   measurement is the per-clause cost and almost nothing else. */
n = 25000000
do i = 1 to n
  nop
end
say 'done'
```

**Sizing.**
`n = 3000000`, the plan's own example bound, ran the oracle in 0.119 s from a fresh
directory under the standard benchmark cap -- far short of the 0.5-2 s window every other axis in
this corpus is sized to (`rust/bench-programs/README.md`, "Sizing").
`n = 25000000` was chosen after that probe: 0.904 s on a single direct timing, landing inside the
window.
The harness run below measured it at a 0.8955 s oracle median, confirming the choice.
Registered as `Role::Loop` in `rexx-bench-suite.rs`'s `AXES` and added to `rexx_bench::PROGRAMS`, so
both the interleaved suite and the criterion harness (`benches/interpreter.rs`) cover it.
This document reports the interleaved suite's own figures, not criterion's.
The interleaved suite is what every ratio and interval below comes from, and criterion's own cost
for this axis was checked rather than run, because checking is enough to know it needs no
accommodation: at a ~0.9 s oracle / ~3.0 s Rust program size and criterion's existing
`sample_size(10)` / 30 s measurement-time ceiling for this benchmark group, ten samples of the
slower side costs about 30 s -- inside the ceiling.
Verified deterministic and side-effect-free before being committed: two runs from two different
fresh directories produced byte-identical stdout (`done`), exit 0, empty stderr.

## `emptyloop` is fold-fragile, and an optimiser must not be allowed to hide it

Recorded 2026-08-10, after Moritz observed that const folding would make this axis's cost "go away".

**It would, and that is the problem rather than the fix.**

```rexx
n = 25000000
do i = 1 to n
  nop
end
```

The body is empty and the loop has exactly one observable effect: `i` is `25000001` afterwards. A folder strong enough to prove that collapses the whole construct to an assignment. Three consequences, and the axis exists because of the third:

* The ratio against the oracle stops being a dispatch measurement and becomes "we deleted the work, they did it" -- the oracle does not fold.
* **The clause-dispatch floor becomes unmeasurable**, which is the only reason this axis was added (Task 1).
* This is the one axis whose `DO` body range holds **exactly one op**, which is the shape that exposes the driver's fixed per-entry cost. Fold it and the cost is invisible rather than absent.

**So a task that folds this loop has not met exit criterion 4 on it; it has removed the instrument.** If a later phase lands folding, this axis needs a fold-resistant sibling -- a body whose single clause has an effect the folder must keep -- and the ratio must be read on that instead.

**One thing that does fall out, and it is a real result rather than a caveat.** Folding a loop away is sound only when nothing observes the passes: no `trace i`, no per-clause `SIGL`, no clock read. D23's decision that the trace setting is an input to compilation is exactly the discriminator that makes it legal -- an **untraced** chunk may fold where a traced one may not. That was not the reason D23 was decided, and it is priced at Phase 9, where the spec licenses divergent trace from the first optimising pass that is not trace-neutral.

## Build identity

| | |
|---|---|
| repo commit this axis's code was measured from | `1b3efeb1affe32e30b4d21b03d95d42cce142e5e` (the harness's own `git rev-parse HEAD` at run time, quoted verbatim in the Provenance table below) |
| profile | `release` |
| `rexx-run` sha256 | `98ee5da45a2db83241489c52820e78ad3ea214061aa5ef29fc0ec0c8c0a4b935` (quoted from the Provenance table; unaffected by this task, which touches no `rexx-exec`/`rexx-core` source -- only `rexx-bench`'s two lists and a new `.rex` file) |
| `rexx-bench-suite` sha256 | `025f67242f3831e57867e801992271b812ed6edf3c0779cd3d6843b39f0281e0` (built after adding the `emptyloop` axis to `AXES`; not printed by the harness itself, recorded here because the harness binary's own code is what iterates the axis list) |

**The measured commit is the parent of this task's own commit, not this task's commit.**
The harness prints `git rev-parse HEAD` at run time, and at run time this task's changes
(`rust/bench-programs/emptyloop.rex`, the two list edits) were staged in the working tree but not
yet committed, so `1b3efeb1` -- the commit Task 0 landed -- is what it printed.
The binaries measured were built from that working tree, which is exactly what this task's own
commit (quoted in the SDD report, `.superpowers/sdd/2026-08-09-phase-4e-ir/task-1-report.md`)
contains: no source changed between the measurement and the commit.

## Oracle fingerprint

Quoted verbatim from the Provenance table the harness printed (three hashes, because the launcher
is 60 KB of `main` and the interpreter itself lives in the shared objects it loads):

* `oracle bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200,
  sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019
* `oracle lib/librexx.so.4` -- size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200,
  sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb
* `oracle lib/librexxapi.so.4` -- size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200,
  sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66

Identical to every fingerprint `perf-baseline.md` has recorded since 2026-08-08: the oracle build
has not moved under this phase.

## The measurement

Produced by `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`, run with no arguments (unpinned,
the same wrapper every committed figure in this project uses).
**Every heading from "Provenance" to "Axes this crate cannot run" below, and every table and
paragraph under them, is that program's standard-output block byte for byte.**
The method behind it -- alternating sides, 8 GiB address-space cap on both sides, the median and
sign-test interval, the fixed per-process offset -- is described once in `perf-baseline.md`'s
"Method, and the three choices that are not the obvious ones" and is not repeated here.

### Provenance

| | |
|---|---|
| measured | 2026-08-09T09:50:49+02:00 |
| repo commit | `1b3efeb1affe32e30b4d21b03d95d42cce142e5e` |
| oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
| oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
| oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=12999832 bytes, mtime=2026-08-09 09:47:56.972822878 +0200, sha256=98ee5da45a2db83241489c52820e78ad3ea214061aa5ef29fc0ec0c8c0a4b935 |
| address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
| pairs per axis | 9 sampled, 1 warm-up pair(s) discarded, oracle and this crate alternating |
| pairs for the offset line | 51 sampled, 5 warm-up |
| statistic | median; interval is the distribution-free sign-test interval for the median at a 95% target |
| working directory of every child | a fresh empty temporary directory |

### Fixed per-process offset (`startup.rex`)

**Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two numbers below are each side's own fixed cost, reported so every axis above can be read net of it -- not as a result about which interpreter starts faster.

| side | median | min | max | 95.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 7.678 ms | 5.139 ms | 9.642 ms | 7.374 - 7.861 ms | 58.6 % |
| this crate | 1.580 ms | 1.185 ms | 2.810 ms | 1.466 - 1.835 ms | 102.8 % |

Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.

### Axes

`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.

| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
|---|---:|---|---:|---:|---:|---|---:|---:|---:|
| `alloc4c` | 1000000 | oracle | 1.1438 s | 1.1361 s | 1.1518 s | 1.1388 - 1.1464 s | 1.37 % | 874289 | 880197 |
| `alloc4c` | 1000000 | this crate | 2.2839 s | 2.2495 s | 2.3005 s | 2.2539 - 2.2983 s | 2.23 % | 437857 | 438160 |
| `arith` | 500000 | oracle | 1.1616 s | 1.1491 s | 1.1717 s | 1.1553 - 1.1646 s | 1.94 % | 430435 | 433298 |
| `arith` | 500000 | this crate | 3.1044 s | 3.0843 s | 3.1099 s | 3.0877 - 3.1099 s | 0.82 % | 161064 | 161146 |
| `compound` | 5000000 | oracle | 1.1490 s | 1.1376 s | 1.1898 s | 1.1398 - 1.1621 s | 4.55 % | 4351623 | 4380896 |
| `compound` | 5000000 | this crate | 6.6810 s | 6.6505 s | 6.7150 s | 6.6638 - 6.7061 s | 0.97 % | 748387 | 748564 |
| `emptyloop` | 25000000 | oracle | 0.8955 s | 0.8797 s | 0.9558 s | 0.8839 - 0.9298 s | 8.50 % | 27916409 | 28157810 |
| `emptyloop` | 25000000 | this crate | 2.9825 s | 2.9741 s | 2.9980 s | 2.9743 - 2.9882 s | 0.80 % | 8382257 | 8386701 |
| `strings` | 3000000 | oracle | 0.8670 s | 0.8510 s | 0.8984 s | 0.8590 - 0.8767 s | 5.47 % | 3460091 | 3491003 |
| `strings` | 3000000 | this crate | 9.0791 s | 8.8988 s | 9.2070 s | 9.0068 - 9.1385 s | 3.39 % | 330429 | 330486 |
| `varlookup` | 19000000 | oracle | 1.2075 s | 1.1895 s | 1.2771 s | 1.1983 - 1.2443 s | 7.26 % | 15734413 | 15835092 |
| `varlookup` | 19000000 | this crate | 5.3131 s | 5.3019 s | 5.3294 s | 5.3047 - 5.3224 s | 0.52 % | 3576054 | 3577118 |

#### Ratios and the gate call

The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.

**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.

| axis | oracle median | this crate median | ratio | ratio interval | verdict |
|---|---:|---:|---:|---|---|
| `alloc4c` | 1.1438 s | 2.2839 s | 2.00x | 1.97x - 2.02x | SLOWER |
| `arith` | 1.1616 s | 3.1044 s | 2.67x | 2.65x - 2.69x | SLOWER |
| `compound` | 1.1490 s | 6.6810 s | 5.81x | 5.73x - 5.88x | SLOWER |
| `emptyloop` | 0.8955 s | 2.9825 s | 3.33x | 3.20x - 3.38x | SLOWER |
| `strings` | 0.8670 s | 9.0791 s | 10.47x | 10.27x - 10.64x | SLOWER |
| `varlookup` | 1.2075 s | 5.3131 s | 4.40x | 4.26x - 4.44x | SLOWER |

#### Same work on both sides

A wall time is only about the workload if the workload ran. Every sampled run on each side printed the same bytes, and the two sides printed the same bytes as each other.

| axis | stable within a side | identical across sides | stdout |
|---|---|---|---|
| `alloc4c` | yes | yes | `12888896` |
| `arith` | yes | yes | `4629643519330627.7808` |
| `compound` | yes | yes | `5000000` |
| `emptyloop` | yes | yes | `done` |
| `strings` | yes | yes | `138000000` |
| `varlookup` | yes | yes | `19000000` |

### `samples/rexxcps.rex`

The oracle's own clauses-per-second benchmark, run from the read-only C++ tree. It self-calibrates: a trial that comes in at or under a second is run again at twice the count, so the two sides do **different amounts of work** and their wall times are not directly comparable. The clauses-per-second figure each side prints is per clause and is the comparable one. Each side's `Averaged:` line is quoted so the asymmetry is visible rather than inferred.

| side | wall median | wall interval | `Averaged:` |
|---|---:|---|---|
| oracle | 1.7830 s | 1.7737 - 1.8056 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
| this crate | 4.6558 s | 4.6365 - 4.6708 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.3s)` |

| side | median cps | min | max | 96.1% interval | spread |
|---|---:|---:|---:|---|---:|
| oracle | 16995862 | 16659906 | 17221278 | 16789355 - 17186575 | 3.30 % |
| this crate | 2317948 | 2296155 | 2331047 | 2306226 - 2323553 | 1.51 % |

**Internal cps ratio: 7.33x** (oracle median over this crate's median), interval 7.23x - 7.45x.

### Axes this crate cannot run

Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.

| axis | exit status | message |
|---|---:|---|
| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |

## Why the blocked axes carry no ratio

**Written here, not emitted by the harness -- the table above is where the quoted block ends.**
`alloc`, `dispatch` and `heapshape` all fail the same way: this crate has no object model yet
(Phase 5), and each of these three programs' workload is defined in terms of a message send
(`.array~of`, a method dispatch, a forced collection reached through allocation via the object
model).
They carry no ratio for the same reason `perf-baseline.md`'s own "Axes this crate cannot run"
section gives none: there is no completed run on this side to compare against the oracle's.

## Movement against `perf-baseline.md`'s `d233d1e9` figures

Two of the five pre-existing axes moved by more than this document can wave away as ordinary
run-to-run noise, and this section names the movement rather than folding it into "differ
slightly."

| axis | `d233d1e9` ratio | this run's ratio | change | intervals overlap? |
|---|---:|---:|---:|---|
| `alloc4c` | 2.08x (2.03x-2.11x) | 2.00x (1.97x-2.02x) | -3.8% | no |
| `compound` | 6.08x (6.02x-6.14x) | 5.81x (5.73x-5.88x) | -4.4% | no |
| `arith` | 2.70x (2.68x-2.72x) | 2.67x (2.65x-2.69x) | -1.1% | yes |
| `strings` | 10.61x (10.40x-10.85x) | 10.47x (10.27x-10.64x) | -1.3% | yes |
| `varlookup` | 4.35x (4.32x-4.41x) | 4.40x (4.26x-4.44x) | +1.1% | yes |

**This document cannot distinguish two candidate explanations, and does not choose between them.**

`git log d233d1e9..1b3efeb1 -- rust/crates/rexx-exec/src rust/crates/rexx-core/src` returns exactly
one commit: `f9d9f46c`, this phase's own Task 0.
It extracted `grant_procedure_permission` and `apply_flow` out of `run_activation`'s per-clause
loop -- the hottest loop in the interpreter -- changing 183 lines of `run.rs` and adding 31 to
`roots.rs`.
Extraction is not claimed to change behaviour, and the full workspace suite agrees (1337 passed, 0
failed, before and after), but extraction can move a compiler's inlining decisions even when
behaviour is identical, and nothing in this task measured whether it did here.

**Candidate 1: Task 0's extraction moved the generated code, and `alloc4c`/`compound` are the axes
sensitive to it.**
Consistent with the two axes carrying disjoint intervals both moving in the same direction (the
ratio fell on both), and with `arith`/`strings`/`varlookup` -- which exercise less of
`run_activation`'s extracted path per iteration, or exercise it differently -- moving an order of
magnitude less.

**Candidate 2: this is ordinary cross-run noise on this machine, and `alloc4c`/`compound` are simply
the two axes it lands on hardest this time.**
`perf-baseline.md`'s own "The internal cps ratio is unstable across runs, more than the wall-clock
ratios are" section already recorded `compound` moving from 6.35x to 6.08x -- about 4% -- between
two runs with no code change between them at all, and called that "direct evidence that between-run
variance on `compound` exceeds what either run's own within-run interval reports."
A second ~4% movement on the same axis, this time coinciding with a code change, has exactly the
same magnitude as a movement that document already attributed to noise rather than to a cause.

**The consequence for later tasks.**
Task 4 predicts `emptyloop` and `varlookup`'s movement; Task 9 predicts `arith` and `compound`'s.
A task whose measured movement on `alloc4c` or `compound` comes in within a few percent of what this
section reports cannot conclude its own change caused it, because a movement of that size has now
been observed on `compound` twice, with no attributable single cause confirmed either time.
Settling which candidate above is right would need attributing Task 0's change specifically --
measuring the tree-walker's axis figures before and after `f9d9f46c` in one sitting -- which this
task does not do and was not asked to do.
Until that measurement exists, a later task claiming a movement on `alloc4c` or `compound` smaller
than about 4% should say so against this section, rather than treating this document's figures as a
clean, zero-change starting point for those two axes specifically.

## Per-iteration cost, net of the fixed per-process offset

The task brief asks this document to record, per axis, "Rust ns/clause, oracle ns/clause, the
ratio, and the number of interleaved passes."
**Stated as ns/iteration below, not ns/clause**, and the distinction is not cosmetic: the harness
computes iterations per second from each program's own `n = ...` loop bound (a fact about the
*program*), not from a per-clause count the interpreter itself reports.
`samples/rexxcps.rex` above is the one figure in this document that *is* a true per-clause count,
because that program is the oracle's own instrument and reports it directly; nothing here re-derives
a clause count for the seven axis programs, each of which executes a different, small number of
Rexx clauses per iteration (`varlookup`, for instance, is two assignment clauses per pass; `arith`
is seven -- two `NUMERIC DIGITS` clauses plus five assignments; `emptyloop` is one `NOP`, plus
whatever the interpreter's own loop-control step costs, which is not a source-level clause at all).
Manufacturing a per-clause figure by guessing that count would be a new, unverified claim, not a
restatement of what the harness measured -- so this section reports ns/iteration, computed from the
"net" throughput column above (median wall time less that side's fixed per-process offset), which is
exactly what the harness already establishes.

Nine interleaved passes per axis (`PAIRS`), one warm-up pair discarded, for every row below.

| axis | oracle ns/iteration (net) | this crate ns/iteration (net) | ratio |
|---|---:|---:|---:|
| `alloc4c` | 1136.1 | 2282.4 | 2.01x |
| `arith` | 2307.8 | 6205.6 | 2.69x |
| `compound` | 228.27 | 1335.9 | 5.85x |
| `emptyloop` | 35.51 | 119.24 | 3.36x |
| `strings` | 286.46 | 3027.0 | 10.57x |
| `varlookup` | 63.15 | 279.55 | 4.43x |

These ratios are computed from the net (offset-corrected) throughput and so differ in the second
decimal place from the "Ratios and the gate call" table above, which is computed from the raw
median wall time; both are derived from the same nine paired samples per axis and agree to within
about a percent everywhere.
Neither is "the" ratio -- Global Constraints' gate definition (point estimate against the oracle's
interval, slow side) uses the raw-median table, and it is the one to cite for a pass/fail call.

## Reproducing

```sh
cd rust
cargo build --release -p rexx-exec --bin rexx-run -p rexx-bench --bin rexx-bench-suite
./target/release/rexx-bench-suite > phase-4e-anchor-rerun.md      # a few minutes
```

`--self-check` runs a single pair per axis and labels its own output as not a baseline; useful for
exercising the harness without spending the several minutes a full run takes.

## What this document is not

It is not a re-measurement of the five pre-existing axes for `perf-baseline.md`'s own purposes --
that document's own Phase 4d-1 sections stand as they are, and this task did not edit them.
It is not an optimisation, a prediction, or an attribution of any axis's cost to any cause: Tasks 4
onward do that, against the figures recorded here.
It is not a claim that the tree-walker figures above are stable to more than a percent or two
run-to-run: `perf-baseline.md`'s own "internal cps ratio is unstable across runs" section already
established that for this same machine, and nothing in this task's single run contradicts or
extends that finding.

## Task 4a: `Do`/`Loop`, predicted versus measured

The first task in this phase to run any of a program's clauses from a compiled stream: a `DO`/`LOOP`
compiles to an op of its own and its body's clauses are stepped through the driver instead of
straight into the tree-walker's clause unit.
The construct itself is resolved by the same `Interp::run_loop` under both engines, so the two arms
agree by construction and this is a measurement about cost only.

### The prediction

Recorded before measuring: **no movement, or a regression of up to about 2%** on both `emptyloop` and
`varlookup`, IR arm against tree-walker arm.
The reasoning was a decomposition of `emptyloop`'s 120 ns/iteration measured at `777fa4a9` -- about 26
ns of repeating-loop framing, about 57 ns of controlled-loop control step, about 37 ns for the one
body clause -- none of which this promotion changes, against an oracle figure of about 35.8
ns/iteration for the whole thing.
The promotion adds an `op_of` lookup, an `ops` index and a match per body clause and removes nothing,
so the arm doing the extra work should have been the slower one.

### The measurement

Both arms are the same binary, selected through `REXX_ENGINE`, so there is no build identity to
reconcile between arms.
There is one between *builds*, and it decided the design: a first sitting read `emptyloop` at 5.6% in
the IR arm's favour, and reverting the promotion showed that most of that was the **tree-walker** arm
reading 3.05 s in one build and 2.88 s in the other -- on an arm that compiles nothing and cannot be
affected by the change. Code layout on this machine is worth about 2% on this axis.

So the estimator is taken within one binary and the reverted build is its negative control, with all
four cells interleaved in one sitting, four rounds per axis, `ulimit -v 8388608`, one fresh empty
directory:

| axis | binary | tree-walker arm | IR arm | IR vs tree-walker |
|---|---|---:|---:|---:|
| `emptyloop` (25e6) | promoted | 3.01 s | 2.96 s | **-1.7%** |
| `emptyloop` | reverted | 2.95 s | 2.955 s | -0.2% |
| `varlookup` (19e6) | promoted | 5.325 s | 5.155 s | **-3.2%** |
| `varlookup` | reverted | 5.345 s | 5.335 s | -0.2% |

### What it means, and what it does not

The prediction was **wrong in sign on both axes**, and the movement arrived by no route anyone has
named: the arm that does strictly more per-clause work is the faster one, and nothing here explains
why.
Per this phase's own rule, a movement that arrives other than by the stated hypothesis has not
confirmed the hypothesis, so this is a measured improvement without a mechanism and should not be
quoted as "promotion made loops faster" until something identifies one.

The percentages above are IR-arm-against-tree-walker-arm and are **not** comparable with the
tree-walker-vs-oracle ratios earlier in this document: this sitting ran no oracle at all.
Re-deriving 3.33x and 4.40x against these numbers would be combining two sittings, which is the
comparison this document's own method forbids.

## Task 4c: the loop header flattened -- and why the wall-clock instrument answers nothing here

BASE `de05ea58`, head `b6d54856`.
Interleaved base-against-head within one sitting, `ulimit -v 8388608`, three rounds per cell, both
arms of each binary selected through `REXX_ENGINE`.

**Predicted before measuring: no movement on either axis, on either arm** -- `|delta| < 1%`.
Everything this task moves is paid once per loop entry and neither axis enters a loop more than once.

### The instrument comes first, because it decides what the rest is worth

Two release builds of the base worktree differing **by one comment and nothing else** read, on
`emptyloop`, 2.745 s and 2.959 s on the tree-walker arm (+7.8%) and 2.907 s and 3.152 s on the IR arm
(+8.4%).

**The cause is not code layout, and getting that wrong once is why it is spelled out here.** The two
builds have a byte-identical `.text` -- same sha256, same size, same load address, differing only in
one debug-line entry -- so the executed code is the same and codegen variance cannot explain any of
it (`68f29913`). It is run-to-run variance in the measurement environment, which is the worse of the
two answers: layout could in principle be stabilised and this cannot, and **it bounds repeated runs
of one binary rather than only comparisons between two**.

So the rule this figure earns is not "cross-binary claims need a control build" but the stronger one
the spec now carries: **a wall-clock or cycle claim on this axis means nothing unless its arms ran
interleaved in one sitting**, and the ~8% is the size of what a non-interleaved comparison is reading.
`instructions:u` is unaffected -- it reads the same to within 1e-8 across runs of one binary -- and it
is what the attribution below uses.

### Instructions, which is the instrument that resolves this change

| axis | arm | base | head | delta |
|---|---|---:|---:|---:|
| `emptyloop` (25e6) | tree-walker | 38.0007e9 | 37.8507e9 | **-0.395%** |
| `emptyloop` | IR | 40.4007e9 | 40.3257e9 | **-0.186%** |
| `varlookup` (19e6) | tree-walker | 71.8207e9 | 71.6307e9 | **-0.265%** |
| `varlookup` | IR | 74.0437e9 | 73.9297e9 | **-0.154%** |

The prediction was wrong: there is a movement, it is a saving, and it is **per clause** rather than
per loop entry -- 150e6 instructions over 25e6 passes is exactly 6, and `varlookup`'s 190e6 over 38e6
body clauses is 5.

**The mechanism proposed for it was tested and refuted.** The candidate was dropping the `BodyEngine`
argument from `step`, which is on the per-clause path: a control build of `b6d54856` with that
argument put back, changing nothing else, reads 37.8507e9 and 40.3257e9 -- the head figures, to eight
significant figures, on both arms.
So the saving is somewhere else in the change and no route has been named for it.
Per this document's own rule it is recorded and not claimed.

### Cycles and wall clock, recorded and not attributed

Head is slower on both arms and both axes -- `emptyloop` +6.2% tree-walker and +1.3% IR, `varlookup`
+1.8% and +4.5%, with cycles agreeing -- **while executing fewer instructions on all four cells**.
The identical-`.text` pair above moves the same cells by 7.8% and 8.4%, which is larger than three of
those four numbers and comparable to the fourth, on binaries whose executed code cannot differ at all.
So these deltas are inside the measurement environment's own spread and are not this task's, in either
direction.

### `emptyloop` fails criterion 4, and it failed before this task

Task 4c flagged a contradiction here -- it read `emptyloop`'s IR arm 6.7% slower than its tree-walker
arm within one binary at BASE, where 4b-M's figures said parity -- and the review resolved it against
4b-M rather than against Task 4c.

**4b-M's 3.12x/3.13x are withdrawn** (`6ce592ff`). Re-measured within one binary as a paired sign
test at 4b-M's own commit `1535b030`, two independent builds read IR/TW **1.0299** and **1.0265**,
with the IR arm faster in **0 of 9** pairs both times, against the 0.9964 that was reported. At head
the same measurement reads **1.0237** wall and **1.0654** instructions; `varlookup` reads **1.0511**
and **1.0321**. So both registered axes fail criterion 4 and `emptyloop` was never passing it.

The cost is attributed, on instruction counts, per `DO`-body pass, IR arm minus tree-walker arm:

| commit | per-pass delta |
|---|---:|
| before the frame stack | 63 |
| after the frame stack (4b') | **153** |
| after `settle` was inlined | 96 |
| through Task 6 | 96 |
| after Task 4c | **99** |

**4b' owns +33 per pass, Task 4c owns +3, and Task 6 costs exactly zero on this axis.** So the
structural remedy belongs to the frame stack rather than to any later promotion, and neither the
`varlookup` residual's amortisation hypothesis nor this task's header work is where it sits.

## Task 7: `Assignment` and `Say` promoted -- and the `varlookup` discharge

BASE `caf16c90`, head `f55ea409`.
`perf stat -e instructions:u`, base against head interleaved in one sitting, `ulimit -v 8388608`, one
fresh empty directory; wall clock as criterion 4's estimator -- both arms of **one** binary through
`REXX_ENGINE`, interleaved inside each round with the within-round order alternating, rounds outermost,
9 pairs.

**Predicted before measuring: the residual does not close, |delta| under 1% on the ratio, sign not
predicted.** The reasoning was that amortisation cannot happen here: `varlookup`'s `DO` body is entered
once per pass and holds the same two clauses either way, so the per-entry cost is untouched, and
neither of its body clauses holds a literal -- `x = x + 1` is an operator and `y = x` a variable read,
so the native constant op does not run on this axis even once inside the loop.

### Instructions

| axis | arm | base | head | delta |
|---|---|---:|---:|---:|
| `emptyloop` (25e6) | tree-walker | 37.8507e9 | 37.8507e9 | **+0.000%** |
| `emptyloop` | IR | 40.3257e9 | 40.3257e9 | **+0.000%** |
| `varlookup` (19e6) | tree-walker | 71.6307e9 | 72.5807e9 | **+1.326%** |
| `varlookup` | IR | 73.9297e9 | 77.8437e9 | **+5.294%** |
| `arith` | tree-walker | 30.4006e9 | 30.4631e9 | +0.206% |
| `arith` | IR | 30.5158e9 | 30.7730e9 | +0.843% |
| `compound` | tree-walker | 77.0129e9 | 77.2634e9 | +0.325% |
| `compound` | IR | 77.6189e9 | 78.6489e9 | +1.327% |
| `strings` | tree-walker | 84.8048e9 | 85.1798e9 | +0.442% |
| `strings` | IR | 85.3657e9 | 86.9107e9 | +1.810% |
| `alloc4c` | tree-walker | 15.9093e9 | 15.9842e9 | +0.471% |
| `alloc4c` | IR | 16.0523e9 | 16.3613e9 | +1.925% |

`emptyloop` is the control and it worked: its body is a single `nop`, this task promotes nothing in it,
and both arms are identical to base at nine significant figures. **So every other row above is this
task's, and none of it is the sitting's.**

`arith`'s own base is not deterministic to better than about 0.08% -- 30.4006e9 twice and 30.4236e9
once in one session -- so its row is +0.13% to +0.21%. It is the only axis where that was seen.

### Criterion 4

| axis | tree-walker median | IR median | IR/TW wall | IR faster | IR/TW instructions |
|---|---:|---:|---:|---:|---:|
| `varlookup` | 5.2210 s | 5.4941 s | **1.0523** | 0 of 9 | **1.0725** |
| `emptyloop` | 2.9629 s | 2.9294 s | 0.9887 | 9 of 9 | 1.0654 |

**`varlookup` still fails, and the amortisation hypothesis is refuted**: 1.0321 on instructions at base
against 1.0725 at head, so the residual widened by 4 percentage points rather than closing.

**`emptyloop`'s wall clock is the environment spread, caught happening.** Two sittings of this session
read IR/TW 1.0526 and 0.9887 on binaries with *identical* instruction counts. The 0.9887 above is
recorded and claimed for nothing, in either direction.

### The remedy the number names

IR arm minus tree-walker arm, on `varlookup`:

| | per `DO`-body pass | per body clause |
|---|---:|---:|
| base | 121 | 60.5 |
| head | 277 | **138.5** |

**Task 7 adds 78 per body clause and zero per body entry**, which is the amortisation premise failing
directly -- the quantity that grew is the one paid per clause. So hoisting the op range so it travels
with `BodyEngine` addresses per-entry cost and cannot touch this, and a single-op fast path addresses a
range holding one op where `varlookup`'s promoted regions hold three and four. What is left is **the
two-level shape**: `run_clause_region` plus `run_region_ops`, two multi-argument calls and a second
op-dispatch loop per promoted clause, against the tree-walker's one `step` dispatch. It multiplies with
every further promotion.

### The tree-walker column, which is the extraction and not the promotion

Established by probe rather than argued: a build with `step`'s two arms written out inline, the shared
functions left in place for the ops, reads `varlookup` 71.6307e9 and `emptyloop` 37.8507e9 on the
tree-walker arm -- base exactly, nine significant figures, both axes -- with the IR arm unmoved. **That
probe is not the remedy**, because writing the arms out while the shared function exists is two
implementations of one instruction.

What was measured instead: `#[inline]` on the pair recovers 722e6 on `varlookup`'s IR arm, 19 per body
clause, and reads exactly zero on every tree-walker cell and on both `emptyloop` cells -- kept.
`#[inline(always)]` recovers a further 76e6 there and costs `emptyloop` 550e6 on **both** arms, a
program whose loop body enters neither function, so it perturbs `step`'s codegen rather than removing a
call -- rejected. `#[inline(always)]` on `assign_expr_target` as well reads 72.7517e9 on `varlookup`'s
tree-walker arm, worse than either -- rejected.

No mechanism below "LLVM's inlining decisions in `step` changed" has been named for the residual +0.2%
to +1.3%, so per this document's own rule it is recorded and not claimed.
