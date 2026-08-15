# Unit 0 -- what was built, what was withdrawn, and the one reading it left

**Landed as `acb363bc`.**

**The upfront noise characterisation was withdrawn mid-run and is not completed here.**
The plan records why, and the reason is a defect in the question rather than in the run: `2026-08-09-phase-4f-optimisation-loop.md`'s rewritten Unit 0, and `phase-4d-gate.md`'s second amendment, both at `ff46ee10`.
What this report covers is the harness that survives the withdrawal, the control pair the escalation rule needs, and the one measured reading the stopped run produced.

**Nothing here optimises the interpreter.**
No file under `rexx-exec` or `rexx-core` was touched, and `rust/target/release/rexx-run` is sha256 `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967` before and after -- the binary the baseline, the attribution and the gate all name.

## What was built, and why it is worth having on its own

### One child wrapper, shared

`rexx_bench::child` holds the single capped, directory-pinned way of launching either interpreter: `cd` into a fresh empty directory, `ulimit -v 8388608`, `exec` the binary, timed by `Instant` through `timing::time_once`.
`rexx-bench-suite` and `rexx-bench-band` both go through it.

**This removes a split that contaminated a figure the gate quoted.**
`phase-4d-attribution.md` records that its own prototype runs used a bash subshell with the `ulimit` builtin timed by `date +%s.%N`, while the suite used a `/bin/sh` wrapper timed by `Instant`, and that this difference is part of why its base medians land between 7.2% under and 0.2% over the committed baseline's on a byte-identical binary.
`phase-4d-gate.md` inherited that 7.2% as its band and had to disclose, in its own text, that the number was between-run **and** between-harness.
With one wrapper the between-harness half cannot recur, whatever happens to the between-run half.

**The committed baseline's harness is unchanged, and that is asserted rather than described.**
`Wrapper::default()` adds nothing between the shell and the interpreter, and `the_default_wrapper_is_the_bare_one` pins the whole argument vector -- the `-c`, the script text, the working directory, the `8388608`, the binary, the program -- to the seven strings the suite built before this module existed.
A harness change that silently repriced the baseline would be the worst available outcome here, so it is a test and not a sentence.
`the_wrapper_nests_taskset_outside_perf_outside_the_interpreter` pins the other order, because only one nesting measures the intended thing: `taskset` must set the mask that `perf` and the interpreter both inherit, and `perf` must be the interpreter's immediate parent.

### Two options on that wrapper, both off by default

* `--pin <cpulist>` confines every child to those CPUs through `taskset`.
* `--counters` counts retired cycles and instructions through `perf stat -x,`.

Both default off. `rexx-bench-suite` gained `--pin` and emits a provenance row for it **only when it is on**, so a run with no arguments still prints the block `perf-baseline.md` carries.

### `rexx-bench-band`

One invocation is one pass: a named set of axes, a warm-up pair and N sampled pairs per axis, the oracle and this crate alternating inside every pair, one tab-separated row per sampled run carrying wall time, cycles, instructions, the one-minute load average and the bytes the child printed.
`--summarise` reads accumulated rows back and reports, per configuration and per axis, the distribution over passes.

**It is kept because the escalation rule needs exactly this shape**, not because the withdrawn characterisation needed it.
The gate now spends measurement only on an axis whose ratio lands near 1.0, and "more runs, targeted, until the interval separates from 1.0" is repeated independent invocations of a cheap comparison -- which is what this program is.
Its module documentation says in as many words that it does not establish a global band and is not the loop's accept rule, and its summary line names the widest per-axis envelope as one configuration's worst axis rather than as a threshold.

The reduction is in the binary rather than in a script because a figure that decides an axis near 1.0 has to be recheckable.
`--metric wall|cycles`, `--ratio pooled|paired` and `--statistic median|min` re-reduce rows already collected, so alternative reductions can be compared on one machine state instead of on separate runs.

### The control pair

`rust/bench-control/alloc4c-101.rex` is `bench-programs/alloc4c.rex` with the loop bound raised from 1,000,000 to 1,010,000 and nothing else changed.

**The escalation rule requires it**: a tight interval near 1.0 does not distinguish a sensitive instrument from a blind one, so a pair differing by a known small amount runs alongside the axis.
`the_control_differs_from_its_axis_only_in_the_loop_bound` asserts both halves -- byte-identical outside the `n = ` line, and a bound exactly 1.01 times the axis's.
Either half can fail silently: a control that drifted anywhere else would still look like a 1% control, and a bound edited to a rounder number would change the size of the effect the instrument is asked to resolve with nothing to notice.

It is deliberately outside `bench-programs/`, because `rexx-bench-suite` asserts its axis list against that directory in both directions and a program placed there becomes a dimension of the committed baseline.
`rexx-bench-band` reaches it by path: an `--axes` entry containing a `/` is taken as a path rather than as a name.

## The one measured reading the stopped run produced

**Measured 2026-08-09, 08:10:55 to 08:32:49, tree at `77679f75` plus the harness that later landed as `acb363bc`.**
`rexx-run` sha256 `c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967`; oracle `bin/rexx` `bb5bb8cc...fa13019`, `lib/librexx.so.4` `42136c40...6d9b6fb`, `lib/librexxapi.so.4` `3536b763...690dc7d66` -- the three the gate names.
Load average 1.01 to 1.56 across every run, which on 32 cores is the benchmark itself and the agent driving it.
314 rows: `bare` 120, `pin` 103, `pinperf` 90, giving **three or four passes per arm per axis** before the run was stopped.
Every axis printed one constant value across all 314 rows and all three arms, so every timing is over the same workload.

**Three or four passes decides nothing, and the table below is published as a reading rather than as a result.**

| axis | `bare` median ratio | `pin` median ratio | shift | `bare` envelope | `pin` envelope |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | 2.0029x | 2.0018x | -0.05% | 0.63% | 0.58% |
| `arith` | 2.7072x | 2.6548x | -1.94% | 0.75% | 0.58% |
| `compound` | 5.8641x | 5.7723x | -1.57% | 0.48% | 1.20% |
| `strings` | 10.5219x | 10.0853x | -4.15% | 0.77% | 1.88% |
| `varlookup` | 4.3231x | 4.1053x | -5.04% | 1.24% | 0.48% |

Envelope is the largest deviation of a pass's ratio from the median of the passes, over the three or four passes that arm has on that axis.

**Pinning is not a neutral instrument, and that is the part of this worth carrying forward.**
It moved the central ratio on four of five axes, all in the same direction, by up to 5.04% on `varlookup` -- larger than either arm's envelope on that axis, and larger than the 4.15% on `strings`.
So `taskset` is not a control that leaves the quantity alone while tidying its variance; it changes the number being reported.
Anyone comparing a pinned measurement against an unpinned one is reading a difference that is partly the pinning.

**Whether pinning widens or narrows the spread is genuinely mixed here, and the summary "it widened" is not what the per-axis rows say.**
It narrowed `alloc4c`, `arith` and `varlookup`, and widened `compound` and `strings`.
The worst-axis figure rose from 1.24% to 1.88% through a change of which axis is worst: `strings`, which pinning widened, overtook `varlookup`, which pinning narrowed by more than half.
At three or four passes none of these movements is separated from sampling noise, and the honest summary is that pinning did not visibly reduce the spread while it did visibly move the ratio.

**Adding `perf stat` moved the ratio again, in the same direction, on all five axes**: `alloc4c` -0.33%, `arith` -1.32%, `compound` -1.57%, `strings` -1.28%, `varlookup` -0.58%, comparing `pinperf` against `pin`.
That direction is what a fixed per-process cost produces on a ratio whose two sides have different absolute durations: over these rows the oracle's medians run 0.88 s to 1.25 s and this crate's 2.31 s to 8.95 s, so a constant added to both shrinks the quotient.
Recorded as a consistent direction with a candidate mechanism, not as an established one -- nothing here measured `perf stat`'s startup cost directly.

**If core migration is not the variance source, that saves the next person an experiment**, which is the whole reason this reading is written down rather than discarded with the run.

## What is not established

* **No band, global or per axis.** Three or four passes support none, and the gate no longer asks for one.
* **Whether cycles reproduce better than wall time.** The `pinperf` arm collected cycles and the comparison was never run against enough passes to mean anything.
* **Whether a paired or minimum reduction helps.** Both are implemented and neither was evaluated.
* **The between-sitting component.**
  Every pass here is back-to-back inside twenty-two minutes, which does not satisfy the gate's own definition of independent runs -- at least one full run's duration apart, not one sitting.
  A band derived from this would have been a lower bound on the quantity it was meant to establish, which is the observation that made the cost of the withdrawn unit visible.

## The CPU governor: measured and unavailable, not considered and rejected

**There is no `cpufreq` interface on this machine to fix a governor with, because `/sys` does not exist in this environment at all.**
`ls /sys` reports no such file or directory, `/proc/mounts` carries no `sysfs` line, and consequently `/sys/devices/system/cpu` -- and every `scaling_governor` under it -- is absent.
`rust/CLAUDE.md` records `/sys` here as a tmpfs with no cgroup filesystem mounted, which is why no cgroup memory limit can be created and `ulimit -v` has to stand in for one; from this environment it is not even that.
Either way there is nothing to write a governor to.

Frequency does vary: `/proc/cpuinfo` on this AMD Ryzen AI MAX+ 395 reports per-CPU clocks spanning 1.479 to 2.972 GHz across its 32 logical CPUs on an idle machine.
So the intervention is wanted and simply cannot be applied from here.

**A later reader on a machine with sysfs should try it rather than assume it was weighed and dropped.**
Counting cycles with `perf stat` is the substitute available here, and it is a substitute rather than an equivalent: cycles are immune to frequency scaling, but a cycle ratio equals a wall-clock ratio only if both sides run at the same average frequency, and the gate is written against wall-clock ratios.

## An inconsistency left in the plan, not fixed here

`2026-08-09-phase-4f-optimisation-loop.md` still refers to "the band" in two places downstream of the section that withdrew it: the re-profile cadence ("when re-profiling stops proposing candidates above the band") and stopping rule 2 ("re-profiling yields no candidate above the band on an axis still short").
Under the rewritten Unit 0 there is no band for a candidate to be above.
Flagged rather than edited, because the plan is not this task's to amend and the substitution is not mechanical -- the accept rule's replacement is "a reproducible gain under pairing", which is a different shape of test from "above a threshold".

## Reproducing

```sh
cd rust
cargo build --offline --release -p rexx-bench --bins

# one pass: five axes, one warm-up and three sampled pairs each
./target/release/rexx-bench-band --header --pass 1 --config bare > rows.tsv

# the same, pinned to one physical core and its SMT sibling, with counters
./target/release/rexx-bench-band --pass 1 --config pinperf --pin 12,28 --counters >> rows.tsv

# an axis alongside its control pair
./target/release/rexx-bench-band --pass 1 --config control \
    --axes alloc4c,"$PWD/bench-control/alloc4c-101.rex" >> rows.tsv

# the distribution over passes, and the alternative reductions
./target/release/rexx-bench-band --summarise rows.tsv
./target/release/rexx-bench-band --summarise rows.tsv --metric cycles --ratio paired
```

One invocation is one pass, so the driver is a shell loop and the process boundary between observations is real.
Rotate the configurations within each round rather than running one arm to completion: two separate runs on this machine have invented a 5% effect that was not there.
Every child already runs in a fresh empty temporary directory, which is what keeps the scratchpad off the oracle's external-routine search path.
