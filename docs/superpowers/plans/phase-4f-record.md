# Phase 4f record -- the optimisation loop's ledger

The one committed, appended-to record that `docs/superpowers/plans/2026-08-09-phase-4f-optimisation-loop.md` calls for.
**Appended to, never rewritten.** An entry that turned out wrong is corrected by a later entry saying so, not by editing the one that was wrong -- the record's job is to stop the next person retrying a dead end, and a record that quietly loses its dead ends cannot do it.

**Failures carry the same weight as successes.** `smallvec` for `Number::digits` was measured, rejected, and written down once already; that result is worth more than most accepted changes and it survived only because someone wrote it down.

## What every entry carries

| field | what it means |
|---|---|
| cause or hypothesis | what the change is supposed to be exploiting, named before measuring |
| predicted movement | which axes, and by how much, stated before the measurement |
| measured movement | what the paired interleaved comparison actually read |
| disposition | accepted or discarded, and **a change that reached its axis's target by a route other than its stated hypothesis has not confirmed the hypothesis** and says so |
| commit | read back from `git log` after committing, never written from memory |

## The configuration, fixed once, inherited by every entry below

**Two configuration choices are not free, and this section is where they are pinned.**
Unit 0 measured both: pinning with `taskset` moved four of five axes in one direction by up to 5.04% on `varlookup`, and turning `perf stat` on moved the ratio again on all five.
So a figure taken under a different wrapper is not comparable with the ones here, and an entry that changes the wrapper says so in its own text rather than letting the difference pass as a result.

| | |
|---|---|
| instrument | wall clock |
| harness | `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`, built release |
| **not** the harness | `rexx-arms` compares two arms of **one build** and does not reach the oracle -- `arms.rs`'s own scope note says it deliberately does not, though `child::Side` carries the oracle. It is Phase 4e's instrument, not this phase's oracle instrument. |
| wrapper | `Wrapper::default()`: **unpinned, no `perf stat`** -- no `--pin`, `Counted::Nothing` |
| address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
| statistic | median of 9 sampled pairs, 1 warm-up pair discarded; interval is the distribution-free sign-test interval for the median at a 95% target |
| working directory | a fresh empty temporary directory, created per run by the harness |
| arm | named on every child through `REXX_ENGINE` and printed in the provenance block; `--engine ir` or `--engine tree-walker` |
| oracle | the fingerprinted build under `/home/moritz/dev/repos/ooRexx/build`, `LD_LIBRARY_PATH` set by the harness |

**Why the bare wrapper and not the pinned one.**
The two oracle ratios this phase starts from -- `phase-4e-anchor.md`'s table and the stopped run's unpinned medians recorded in Unit 0 -- were both taken unpinned and uncounted.
Pinning is not obviously worse, and Unit 0's own reading is that it did not visibly reduce the spread while it did visibly move the ratio; the reason to stay unpinned is comparability with the only oracle figures on record, not a claim that unpinned is the better instrument.

**The CPU governor cannot be fixed on this machine and that is a property of the machine.**
`/sys` does not exist -- `ls /sys` reports no such file and `/proc/mounts` carries no `sysfs` line -- so no `scaling_governor` is readable and none can be written.
Measured at rest immediately before entry 1's sitting: 1480.469 to 2965.942 MHz across the 32 logical CPUs.
It is handled, not solved: the oracle and this crate **alternate inside every block**, so a frequency excursion lands on both sides of the ratio in the same order, and the arm order across a sitting is balanced (see entry 1) so a drift across the sitting does not become an arm difference.

## Entries

### Entry 1 -- the entry state: both arms against the oracle, at `131f1b40`

**No change, no hypothesis, no disposition.** This entry optimises nothing; it is the measurement every later entry is read against, and it is the first oracle comparison this project has taken with `Engine::Ir` in existence.

**Why it did not exist already.** `phase-4e-anchor.md` and `perf-baseline.md` are **tree-walker against the oracle**, taken before `Engine::Ir` existed. `phase-4e-gate.md` is **IR against the tree-walker**, on instructions and cycles, within one binary. Neither is IR against the oracle, and the bar is stated against the oracle.

#### Build identity

| | |
|---|---|
| repo commit | `131f1b4061545fcd180346c1a470a1fd9da8bcc4`, printed by the harness in all four blocks |
| working tree | clean at that commit; `git status --porcelain` empty before the build |
| profile | `release` |
| `rexx-run` | size=13819064 bytes, sha256 `f166bb30215747c9753379d2009011f8956c6695bd5efce130711a1999c98255` |
| `rexx-bench-suite` | sha256 `4949a32114e8bf728598a05575ff02043920a75ef68fb8b89f1972edccc0c694` |

Both binaries were rebuilt after `cargo clean -p rexx-exec -p rexx-bench --release`, because a clean `git status` says nothing about what is in `target/`.

#### Oracle identity

Three objects, because the launcher is 60 KB of `main` and the interpreter is in the shared objects it loads.

| object | size | sha256 |
|---|---:|---|
| `build/bin/rexx` | 62600 | `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019` |
| `build/lib/librexx.so.4` | 17853856 | `42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb` |
| `build/lib/librexxapi.so.4` | 667792 | `3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66` |

**Identical to every fingerprint on the record since 2026-08-08**, including `phase-4e-anchor.md`'s and `phase-4d-gate.md`'s three. The oracle has not moved, so the movements below are this crate's.

#### The harness measures the arm it names, checked rather than assumed

`Side::rust` used to hand its child an **empty** environment, so `REXX_ENGINE` came from whatever launched the harness -- which means `rexx-bench-suite`, `rexx-bench-band` and `rexx-time` silently changed which engine they measured the day `rexx-run`'s unset default moved to `Engine::Ir`. `7f81c7c9` fixed it. Four checks, because a figure attributed to the wrong arm is the one error nothing downstream could notice:

* **The type.** `Side::rust(binary, arm)` takes the arm as a parameter and there is no constructor without one; its `env` is `[("REXX_ENGINE", arm.engine())]`. `child.rs`'s `time_once` overlays that on the inherited environment, so an explicit value wins over an inherited one.
* **The guard test.** `cargo test --release -p rexx-bench a_rust_side_names_its_arm` -- `1 passed; 0 failed; 29 filtered out`, run count read rather than the exit status alone.
* **A hostile inherited value is overridden.** `REXX_ENGINE=bogus ./target/release/rexx-bench-suite --self-check --engine ir` completes every axis and exits 0, with `| this crate's engine | REXX_ENGINE=ir |` in its provenance. Handed to `rexx-run` directly, the same value is fatal at rc 2:

  ```
  rexx-run: REXX_ENGINE is `bogus`: expected `ir` or `tree-walker`
  ```

  So under the old constructor that run would have failed outright rather than measured anything.
* **The two spellings really select different engines.** `perf stat -e instructions:u` on one `varlookup` at a reduced bound, twice each: tree-walker 763411912 and 763411673, IR 699219924 and 699220368 -- a ratio of 0.9159, against the 0.91619 `phase-4e-gate.md` records for that axis on that instrument. The variable changes the executed instruction stream, not only a label.

#### The sitting

One contiguous sitting, 2026-08-11 12:40:40 to 13:08:27 +02:00, four blocks of `rexx-bench-suite` under the configuration fixed at the top of this file. Every block exited 0, none printed a "This run is not a baseline" section, and every axis in every block reported stdout stable within each side and identical across the two sides.

**The arm order is ABBA -- `ir`, `tree-walker`, `tree-walker`, `ir`.** Each block is itself an oracle/this-crate interleave over 9 sampled pairs, so within a block the ratio is drift-cancelling in the way this project's method requires. The two arms cannot occupy one block -- the suite measures one arm per invocation -- so the balanced order is what keeps a drift **across** the sitting out of the arm difference: a linear trend hits the IR arm's two blocks symmetrically about the tree-walker's two.

One-minute load average read 1.19, 1.18, 1.08, 1.05 and 1.15 at the five block boundaries, on 32 logical CPUs with nothing else running but this session's own agent process. Frequency measured at rest before the sitting ranged 1480.469 to 2965.942 MHz across the 32 CPUs, and could not be fixed for the reason given at the top of this file.

| block | started | arm | rc |
|---|---|---|---:|
| 1 | 12:40:40 | `ir` | 0 |
| 2 | 12:47:38 | `tree-walker` | 0 |
| 3 | 12:54:34 | `tree-walker` | 0 |
| 4 | 13:01:30 | `ir` | 0 |

#### The measurement, per block

This crate's median wall time over the oracle's, **recomputed from the two medians the harness prints** rather than copied from its own ratio column, which is printed to two decimal places. The medians are printed to four, so the quantisation here is under 0.02% on every axis.

| axis | b1 `ir` | b2 `tw` | b3 `tw` | b4 `ir` |
|---|---:|---:|---:|---:|
| `alloc4c` | 1.9901 | 1.9383 | 1.9266 | 1.9685 |
| `arith` | 2.6634 | 2.6780 | 2.6574 | 2.6681 |
| `compound` | 5.9059 | 5.7158 | 5.7220 | 5.8905 |
| `emptyloop` | 3.1614 | 3.1248 | 3.1078 | 3.2017 |
| `strings` | 10.5865 | 10.5400 | 10.4717 | 10.5675 |
| `varlookup` | 3.7488 | 4.1992 | 4.1748 | 3.7330 |
| `rexxcps` (cps) | 7.3353 | 7.1088 | 7.1237 | 7.3579 |

The medians those ratios are computed from, in seconds, oracle then this crate. They are quoted here rather than left in the harness output because the harness output is not committed: the four blocks, the control's rows and the driver log are in `.superpowers/sdd/2026-08-09-phase-4e-ir/f4-entry-evidence/`, which `.gitignore` excludes along with the rest of the SDD ledger.

| axis | b1 `ir` | b2 `tw` | b3 `tw` | b4 `ir` |
|---|---|---|---|---|
| `alloc4c` | 1.1451 / 2.2789 | 1.1508 / 2.2306 | 1.1601 / 2.2351 | 1.1579 / 2.2793 |
| `arith` | 1.1588 / 3.0864 | 1.1566 / 3.0974 | 1.1644 / 3.0943 | 1.1566 / 3.0859 |
| `compound` | 1.1415 / 6.7416 | 1.1435 / 6.5360 | 1.1456 / 6.5551 | 1.1448 / 6.7435 |
| `emptyloop` | 0.8977 / 2.8380 | 0.8935 / 2.7920 | 0.8939 / 2.7781 | 0.8825 / 2.8255 |
| `strings` | 0.8635 / 9.1414 | 0.8571 / 9.0338 | 0.8579 / 8.9837 | 0.8658 / 9.1493 |
| `varlookup` | 1.2143 / 4.5522 | 1.2008 / 5.0424 | 1.2060 / 5.0348 | 1.2193 / 4.5516 |

`rexxcps` is not a wall-clock row and its ratio above is not a wall-clock ratio. The program self-calibrates -- the oracle ran `200 x 100 iterations of 1000 clauses` and this crate `100 x 100` in every block -- so the two sides did **different amounts of work** and only the clauses-per-second figure each prints is comparable. Median cps, oracle then this crate: 16682668 / 2274309, 16685605 / 2347179, 16757296 / 2352335, 16693627 / 2268800.

#### The measurement, per arm

Each arm's figure is the mean of its own two blocks; the spread column is the gap between those two blocks as a fraction of that mean, which is this sitting's answer to "how reproducible is an oracle ratio at head" and is the number a later escalation decision reads.

| axis | **IR / oracle** | spread | **tree-walker / oracle** | spread | IR against tree-walker | in the bar |
|---|---:|---:|---:|---:|---:|---|
| `alloc4c` | **1.9793x** | 1.09% | **1.9325x** | 0.61% | +2.42% | yes |
| `arith` | **2.6658x** | 0.18% | **2.6677x** | 0.77% | -0.07% | yes |
| `compound` | **5.8982x** | 0.26% | **5.7189x** | 0.11% | +3.14% | yes |
| `strings` | **10.5770x** | 0.18% | **10.5058x** | 0.65% | +0.68% | yes |
| `varlookup` | **3.7409x** | 0.42% | **4.1870x** | 0.58% | **-10.65%** | yes |
| `rexxcps` | **7.3466x** | 0.31% | **7.1162x** | 0.21% | +3.24% | yes |
| `emptyloop` | **3.1816x** | 1.27% | **3.1163x** | 0.55% | +2.09% | no |

**The last column is a ratio of ratios taken from different blocks, and it is the weakest figure in this entry.** The two arms never ran inside one block, so it is not the within-build paired comparison `rexx-arms` produces and it must not be quoted as one; the ABBA order removes a linear drift from it and nothing removes the rest. Read against the spread columns beside it, `varlookup`, `compound` and `rexxcps` separate clearly, `alloc4c` and `emptyloop` separate weakly, and `strings` and `arith` do not separate at all.

#### How far the bar-bound axes are from parity

The bar is **within noise of the oracle, or better, on every classic-Rexx axis**. On the IR arm, the arm this crate ships:

| axis | IR / oracle | distance from 1.0, in units of the gap between this axis's own two blocks | of this crate's wall time, the fraction that has to go |
|---|---:|---:|---:|
| `alloc4c` | 1.9793x | 45 | 49.5% |
| `arith` | 2.6658x | 354 | 62.5% |
| `compound` | 5.8982x | 318 | 83.0% |
| `varlookup` | 3.7409x | 174 | 73.3% |
| `rexxcps` | 7.3466x | 281 | 86.4% |
| `strings` | 10.5770x | 504 | 90.5% |

**All six are NOT MET, and none of them escalates.** Unit 0's rule is that an axis far from 1.0 relative to its own spread is decided and earns no further measurement; the nearest of these sits 45 block-gaps away. `emptyloop`, which the bar does not bind, is 3.1816x at 54 block-gaps and is equally decided.

#### The sensitivity control, because the arm column above claims a per-cent resolution

The arm differences in that column are between 0.07% and 3.24%, so calling any of them real is a claim that the instrument resolves about a per cent, and Unit 0 requires that claim to be demonstrated rather than asserted.

`rust/bench-control/alloc4c-101.rex` is `alloc4c.rex` with the loop bound raised by exactly 1% and nothing else, with a test asserting both halves. Run through `rexx-bench-band` -- the tool `bench-control/README.md` names for this, on the same wrapper, unpinned and uncounted -- on the IR arm, axis order ABBA, 5 sampled pairs per block and 10 samples per axis per side, immediately after the main sitting:

| side | `alloc4c` median | `alloc4c-101` median | measured difference |
|---|---:|---:|---:|
| oracle | 1.1584 s | 1.1693 s | **+0.940%** |
| this crate, IR | 2.2948 s | 2.3184 s | **+1.028%** |

A little under and a little over 1%, which is what the control's own README predicts: 1% more iterations is slightly less than 1% more wall time because each side pays a fixed per-process offset that does not scale with the loop. **The second half of the control also behaves**: the *ratio* barely moves, 1.9810x against 1.9827x (+0.09%), which is what has to happen when both sides are given the same 1% more work.

**It is a separate invocation taken after the sitting, not alongside it**, because no axis escalated and there was nothing to run it beside. What it licenses is the statement that this harness on this machine separates a per-cent difference between two medians of one side; it is not evidence about any particular row above.

One thing falls out of it for free: that invocation's own `alloc4c` IR ratio is **1.9810x**, against the main sitting's 1.9793x -- 0.09% apart, from a different harness binary in a different invocation.

#### What moved against the state Phase 4f starts from

The prior state is the stopped run recorded in Unit 0, at `77679f75`, tree-walker, unpinned medians.

| axis | prior (`77679f75`, tw) | tree-walker here | IR here | prior -> tw | prior -> IR |
|---|---:|---:|---:|---:|---:|
| `alloc4c` | 2.0029x | 1.9325x | 1.9793x | -3.5% | -1.2% |
| `arith` | 2.7072x | 2.6677x | 2.6658x | -1.5% | -1.5% |
| `compound` | 5.8641x | 5.7189x | 5.8982x | -2.5% | +0.6% |
| `strings` | 10.5219x | 10.5058x | 10.5770x | -0.2% | +0.5% |
| `varlookup` | 4.3231x | 4.1870x | 3.7409x | -3.1% | **-13.5%** |

**Every column here crosses both a commit boundary and a sitting boundary**, so a movement of a few per cent is not attributable to anything: `phase-4e-anchor.md`, quoting `perf-baseline.md`, records `compound` moving about 4% between two runs with no code change at all, and treats that as the size of this machine's between-run variance on that axis.

**Exactly one movement clears that, and it is `varlookup` at -13.5% on the IR arm.** Everything else on this table is inside the noise the project has already measured.

#### The scoped-out axes, measured and labelled

* **`startup`** -- reported by the harness as the fixed per-process offset and explicitly **not comparable**: this crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing work the oracle does. Oracle medians 5.867, 6.175, 5.741, 6.175 ms across the four blocks; this crate 1.546, 1.624, 1.508, 1.555 ms. No ratio is taken and none should be.
* **`dispatch`**, **`alloc`** and **`heapshape`** -- **no ratio exists, on either arm.** All three exit 120 with `rexx-exec: a message send is not implemented (Phase 5)`, identically in all four blocks, so there is no completed run on this side to compare against the oracle's. `heapshape` is named here for completeness; the plan gives it no bar because it has no attribution.

#### Four things worth carrying forward, and the first is the surprise

**1. Against the oracle, on wall clock, the IR arm is behind the tree-walker on more axes than it is ahead.** It is ahead on `varlookup` alone (-10.65%), behind on `compound` (+3.14%), `rexxcps` (+3.24%), `alloc4c` (+2.42%) and `emptyloop` (+2.09%), and neither on `arith` (-0.07%) or `strings` (+0.68%), whose differences do not separate from the spread. Phase 4e's head table has the IR arm at or below 1.0 on **four** axes on `instructions:u`; the wall clock does not reproduce that ordering. Nothing here says Phase 4e was wrong -- it says the instrument it closed on and the instrument the bar is stated on disagree about which arm is faster.

**2. The wall clock tracks Phase 4e's cycle column, not its instruction column.** Cross-sitting, so this is a direction and not an arithmetic claim. `phase-4e-gate.md`'s IR-over-tree-walker figures put `alloc4c` at 0.99682 instructions and 1.01562 cycles, `compound` at 0.97214 and 1.01856, `varlookup` at 0.91619 and 0.88777; this entry's wall-clock differences are +2.42%, +3.14% and -10.65%. On `alloc4c` and `compound` the instruction column has the wrong sign and the cycle column has the right one; on `varlookup` both columns have the right sign and the cycle column is the closer of the two. `emptyloop` is predicted by neither -- 1.09223 instructions, 1.00294 cycles, +2.09% wall. **The candidate this points at is instructions-per-cycle rather than instruction count**, and nothing here measures it; the way to settle it is a `perf stat` sitting on both arms at head, which is a different configuration from this one and says so.

**3. The IR bought one axis against the oracle and no others.** `varlookup` -13.5% against the prior state is the whole of the measurable return so far, and it is a real one. The other four axes with a prior figure moved by less than this machine's between-run variance.

**4. The between-block reproducibility of an oracle ratio at head is 0.11% to 1.27%** across all fourteen arm-and-axis cells: widest on `emptyloop` (1.27%, IR) and `alloc4c` (1.09%, IR), narrowest on `compound` (0.11%, tree-walker). That is the first such figure for the IR arm, and it is what a future escalation on an axis near 1.0 has to be read against. It is a gap between two blocks, not a distribution over many.

#### What this entry cannot say

* **Its arm comparison is not paired.** Both arms are the same binary, selected through `REXX_ENGINE`, but each ran in blocks of its own rather than alternating inside one, so the arm column is a ratio of ratios and not a paired reading. `phase-4e-gate.md`'s `rexx-arms` tables remain the sharper instrument for that question, on their own instruments.
* **It measures wall clock only.** No `perf stat` ran in the sitting, deliberately: turning it on moves the ratio on every axis and would have made these figures incomparable with the two oracle baselines on the record.
* **It is one Linux host**, unpinned, on a machine whose governor cannot be fixed. Global Constraints names five platforms and none of them runs this.
* **It says nothing about correctness.** No differential ran in this sitting; what it checked is that both sides printed identical bytes on every axis, which is a guard against timing a run that did not do the work, not a correctness result.
* **`rexxcps`' figure is a clauses-per-second ratio and not a wall-clock one**, because the program calibrates itself to a different iteration count on each side.

#### What re-running the older documents' commands measures today

`phase-4e-anchor.md` and `perf-baseline.md` both hold **tree-walker** figures, and both already carry a note saying so. Their reproduction command is `./target/release/rexx-bench-suite` with no arguments, and **with no arguments that command now measures the IR arm** -- the harness's unset `--engine` default is `Arm::Ir`, matching the engine `rexx-run` ships. `--engine tree-walker` is what reproduces the arm those documents were taken on. Neither document has been re-measured, and this entry does not re-measure them: its tree-walker column is a fresh measurement at `131f1b40`, not a reproduction of theirs.
