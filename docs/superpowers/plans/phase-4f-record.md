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

**Every derived column in this entry is computed at full precision from the harness's printed medians and rounded once, at display.** Rounding a block's ratio to four places first and deriving from that moves a spread by a hundredth of a point and a distance below by up to five units, which is not a difference that means anything but is a difference a recomputation would report as a discrepancy.

| axis | **IR / oracle** | spread | **tree-walker / oracle** | spread | IR against tree-walker | in the bar |
|---|---:|---:|---:|---:|---:|---|
| `alloc4c` | **1.9793x** | 1.09% | **1.9325x** | 0.60% | +2.42% | yes |
| `arith` | **2.6658x** | 0.17% | **2.6677x** | 0.77% | -0.07% | yes |
| `compound` | **5.8982x** | 0.26% | **5.7189x** | 0.11% | +3.14% | yes |
| `strings` | **10.5770x** | 0.18% | **10.5058x** | 0.65% | +0.68% | yes |
| `varlookup` | **3.7409x** | 0.42% | **4.1870x** | 0.58% | **-10.65%** | yes |
| `rexxcps` | **7.3466x** | 0.31% | **7.1162x** | 0.21% | +3.24% | yes |
| `emptyloop` | **3.1816x** | 1.27% | **3.1163x** | 0.54% | +2.09% | no |

**The last column is a ratio of ratios taken from different blocks, and it is the weakest figure in this entry.** The two arms never ran inside one block, so it is not the within-build paired comparison `rexx-arms` produces and it must not be quoted as one; the ABBA order removes a linear drift from it and nothing removes the rest. Read against the spread columns beside it, `varlookup`, `compound` and `rexxcps` separate clearly, `alloc4c` and `emptyloop` separate weakly, and `strings` and `arith` do not separate at all.

#### How far the bar-bound axes are from parity

The bar is **within noise of the oracle, or better, on every classic-Rexx axis**. On the IR arm, the arm this crate ships:

| axis | IR / oracle | distance from 1.0, in units of the gap between this axis's own two blocks | of this crate's wall time, the fraction that has to go |
|---|---:|---:|---:|
| `alloc4c` | 1.9793x | 45 | 49.5% |
| `arith` | 2.6658x | 359 | 62.5% |
| `compound` | 5.8982x | 319 | 83.0% |
| `varlookup` | 3.7409x | 173 | 73.3% |
| `rexxcps` | 7.3466x | 280 | 86.4% |
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

### Entry 2 -- the first profiling pass, and the candidate queue it produced, at `610cb4ef`

**No change, no disposition.** This entry optimises nothing and attempts no candidate; it profiles the state entry 1 measured and produces the queue the loop consumes. Working notes, with every table this entry quotes from and the scripts that made them, are in `.superpowers/sdd/2026-08-09-phase-4e-ir/f4-profile-report.md` -- outside the repository, like the rest of the SDD ledger.

#### What was profiled, and how

`samply` 0.13.1, `--save-only`, one profile per axis per side, thirteen in all. 1 kHz on this crate; **4 kHz on the oracle**, because the oracle's runs are about a second and 1 kHz gives it barely a thousand samples. Analysed through `pollard` with `expand_inlines`, because almost every interesting callee here is inlined and the enclosing function is not the one paying. `unsymbolicated_pct` read 0.0012% to 0.020% across the thirteen.

Each run from a fresh empty directory, `/dev/null` on stdin, `ulimit -v 8388608`, stdout and stderr as separate files. **The arm was named on every run** -- `REXX_ENGINE=ir`, set in the script rather than inherited, for the reason entry 1 sets out at length.

Build identity is entry 1's: repo `610cb4efe9ae9378a63560eb584c982bab64dee1`, working tree clean before and after, `rexx-run` sha256 `f166bb30215747c9753379d2009011f8956c6695bd5efce130711a1999c98255`, the same three oracle objects.

**The profiler did not distort the workload.** Profile durations against entry 1's two IR blocks: `varlookup` 4557 ms against 4552.2 / 4551.6, `arith` 3136 against 3086.4 / 3085.9, `compound` 6765 against 6741.6 / 6743.5, `strings` 9183 against 9141.4 / 9149.3, `alloc4c` 2350 against 2278.9 / 2279.3, `emptyloop` 2867 against 2838.0 / 2825.5. The largest deviation is `alloc4c` at +3.1% and every other axis is inside 1.6%. On the oracle side, `varlookup` 1183 ms against 1214.3 / 1219.3, `arith` 1174 against 1158.8 / 1156.6, `compound` 1181 against 1141.5 / 1144.8, `strings` 851 against 863.5 / 865.8, `alloc4c` 1162 against 1145.1 / 1157.9. Every axis printed on both sides the bytes entry 1 records.

**A share below is a subtree total unless it says self**, and shares from different rows must not be added -- allocator self time sits *inside* every other row's subtree, exactly as `phase-4d-attribution.md`'s C6 says.

#### The answer to "is the gap diffuse or concentrated"

**Concentrated -- but on a different cause per axis, and no single candidate closes any axis.**

Each bar-bound axis has one block worth 29% to 48% of its own time, and on four of the six that block is a different one. The implication for the loop is that a candidate's ceiling is per axis and mostly does not transfer, so the queue is a set of local levers rather than one lever with six readings.

| axis | the largest single block | its share | what remains at that block's ceiling |
|---|---|---:|---:|
| `varlookup` | variable binding resolved by hashing | 41.7% | 3.7409x -> 2.18x |
| `compound` | `Number::div` -- `//` with no small-integer path | 42.0% | 5.8982x -> 3.42x |
| `arith` | the general allocator | 39.2% | 2.6658x -> 1.62x |
| `strings` | the general allocator plus this crate's arena | 45.7% | 10.5770x -> 5.74x |
| `alloc4c` | the general allocator plus this crate's arena | 42.5% | 1.9793x -> 1.14x |
| `rexxcps` | the general allocator plus this crate's arena | 48.4% | 7.3466x -> 3.79x |

**Every "what remains" figure in this entry is a construction, not a measurement**: entry 1's ratio multiplied by one minus a share taken in a different sitting, on the assumption the cost goes to zero and nothing else grows. It is the loosest possible ceiling and it is what the brief asks for. `rexxcps`' row multiplies a wall-time share into a clauses-per-second ratio, which holds only because cps is inversely proportional to this crate's per-clause wall time.

#### The oracle side, which is the thing a one-sided profile cannot tell you

Profiled on the same programs, same wrapper. Three of the six candidate blocks turn out to be costs the oracle does **not** pay, and two turn out to be costs it pays as heavily or more.

* **Variable binding.** `RexxActivation::getLocalVariable` is **1.3% of the oracle's `varlookup`**. This crate spends **41.7%**. The oracle's locals are integer slots assigned at parse time; this crate assigns them at plan-build time and then throws the index away at the assignment site.
* **The `//` on `compound`.** The oracle uses `RexxInteger::remainder`, **7.7% total** of a 1181 ms run -- 0.09 s. This crate takes `Number::div`, **42.0%** of a 6765 ms run -- 2.84 s. **A factor of 31 on one operator.** No `NumberString` path appears on the oracle for this axis at all.
* **Decimal arithmetic on `arith` is the opposite case, and it inverts what `arith` means.** Grouping every frame whose function is a `NumberString` or `Numerics::` member, the oracle spends **74.0% self** and 91.9% total -- 0.869 s self out of a 1174 ms run, with only 0.21 s in anything those functions call. Grouping every `rexx_num::` frame, this crate spends **18.3% self** and 74.9% total -- 0.574 s self out of a 3136 ms run, with **1.78 s in what they call**. So this crate's decimal kernels are not slower than the oracle's; measured in absolute self time they are about a third **faster**, and the whole of `arith`'s 2.666x sits in the allocation and copying around each `Number` rather than in the arithmetic. A candidate that makes long division faster has almost no ceiling; a candidate that stops `Number` allocating has all of it.
* **`memcmp` on `compound` is shared and is not the differentiator.** This crate 13.9% self, the oracle **19.7% self** -- the oracle's own `CompoundVariableTail::compare` walking its tail BST. In absolute terms 0.94 s against 0.23 s, so it is 4x and real, but it is the same mechanism on both sides and it is not the shape of a 5.9x.
* **`alloc4c`'s denominator is confirmed inflated, on the IR arm this time.** The oracle spends **74.5% of that axis under `MemoryObject::newObject` and 60.5% under `collect()`**, with `CompoundTableElement::live` 20.9% self, re-marking a growing table this crate never marks. This reproduces `phase-4d-attribution.md`'s C6 finding at head. **No candidate should be queued against `alloc4c`'s ratio**, and a collector will move it the wrong way.

#### The other half of the allocation story, measured outside the profiler

Peak resident set, `/usr/bin/time -f %M`, same wrapper, one run each: `strings` **3,728,048 KB against the oracle's 20,728 KB**, `arith` 440,604 against 20,472, `varlookup` 150,264 against 20,484, `compound` 41,464 against 20,504, `alloc4c` 579,432 against 536,808. `rexxcps` peaks at 1,798,548 KB on this crate.

That is what "a never-swept arena" costs in a number that is not a share, and it is why `mprotect` (3.5%), `munmap` (2.4%) and `sysmalloc` (2.0%) appear in `strings`' own profile: glibc is growing and trimming a heap that never shrinks by use. `alloc4c` is the one axis where the two sides are close, because there the oracle's table is genuinely live too.

#### The queue, ranked by ceiling

Ranked by the largest ceiling on any bar-bound axis, as the loop's brief requires -- **not** by confidence and not by tidiness. Candidate 2 is the only one with a measured prototype and it is one edit, so it is the cheapest first attempt; that is a remark about cost, not a re-ranking.

**1. Stop calling the allocator once per value.** Ceiling **48.4% `rexxcps`, 45.7% `strings`, 42.5% `alloc4c`, 39.2% `arith`, 29.4% `compound`, 5.3% `varlookup`** (glibc allocator family self time, plus `Heap::alloc_with_uncollected` and the 96-byte `Slot` write). Implied: `arith` -> 1.62x, `rexxcps` -> 3.79x, `strings` -> 5.74x.
*Mechanism, and it is two independent levers rather than one change.* **(a) Fewer allocations.** The split is measurable and differs by axis: on `compound` this crate's arena side is **0.0%** and libc is 29.4%, so every allocation there is `Number`'s digit `Vec<u8>`; on `strings` and `rexxcps` the arena side is 8.9% and 9.7%, so a large part is the heap object itself. So the lever is an inline representation for a small `Number` and for a short string, not a general one. **(b) Reclamation.** `phase-4d-retention.md` measured a trigger policy at -16% on `strings` and nothing distinguishable on `arith`, `compound` or `varlookup`, and never measured `alloc4c` -- which the oracle profile above says is where a collector costs most and reclaims least.
*What would falsify it.* A paired run of either lever moving `strings` less than about 10%, or peak RSS not falling when it lands -- which would say the allocations counted were not the ones removed.
*Risk to the sharing rule: none.* `Heap`, `Slot` and `Number` sit below both engines and neither has an engine-specific path.
*One warning.* Lever (a) for `Number` is adjacent to this record's own headline dead end. The `smallvec` result the plan and this file both cite is **not in this repository**: case-insensitive searches of the working tree, of `git log -S` over all refs, and of `.superpowers/` find the string only in those two sentences. Find the original figures or re-measure and record them; do not treat the citation as the result.

**2. A small-integer path for `//`, `%`, `/` and `**`.** Ceiling **42.0% on `compound`**; implied 5.8982x -> 3.42x, and the prototype measured more than its share.
*Mechanism.* `small_int_arith` (`eval.rs`) matches `Plus`, `Subtract` and `Multiply` and answers `None` for everything else, so `k = i // 500` on two tagged small integers converts both to a `Number` with a digit vector, long-divides, and allocates the result. Add `IntDiv` and `Remainder` through `checked_div`/`checked_rem`; `/` must decline when the division is not exact, and `**` needs its own overflow-checked path. `phase-4d-attribution.md`'s P1 is exactly this edit and measured **-51.6% on `compound`** in one interleaved run -- more than the 42.0% share, because the share is the division alone while the change also removes the operand conversion, the result allocation and the root pushes around it. **The overlap with candidate 1 is real and is why -51.6% exceeds 42.0%**; the two shares must not be added.
*`arith` gets nothing from this and that is the prediction, not a miss.* P1 moved `arith` -0.7%: its divisions have non-integer operands, so a small-integer path cannot reach them.
*What would falsify it.* A paired run moving `compound` by less than about 30%; or any differential divergence -- the sign rule for a negative dividend, and the result's exponent and rendering under a `NUMERIC DIGITS` the small-integer path bypasses, are where this breaks if it breaks.
*Risk to the sharing rule: none.* `small_int_arith` is one function in `eval.rs` reached from both engines through `arith_small_int`.

**3. Bind every simple-variable read and write to an integer slot, not just `Op::Load`.** Ceiling **41.7% on `varlookup`**, 10.3% on `compound`, 4.6% on `strings`, 2.3% on `arith`; implied `varlookup` 3.7409x -> 2.18x.
*Mechanism, at three sites that share one resolver.* `Op::Load` already carries a compile-time `ReadSlot`, and that is what bought `varlookup` its -13.5%. Nothing else does. **(i) The store side.** `Op::Store { index, src }` carries no slot, so `assign_expr_target`'s `ExprKind::Variable` arm calls `code.symbols.name(*id).as_bytes().to_vec()` -- a fresh `Vec<u8>`, 6.3% of the axis -- and looks *that* up in `Plan::names`, a `HashMap<Box<[u8]>, usize>`, 29.1% of the axis. The `Vec` exists only so `trace_assignment` can borrow it, and on an untraced run it is pure waste. **(ii) The loop control variable.** `loop_advance` is 25.1% of `varlookup` and 53.9% of `emptyloop`; inside it `bind_control` -> `slot_of` is 33.9% and `Interp::read` -> the id-keyed `HashMap<SymbolId, usize>` is 25.4%, both once per iteration for a variable whose slot is fixed when the loop is entered. **(iii) The id-keyed map** that remains on reads the compiler could not resolve, 6.3% of the axis.
*Op width is not in the way.* `size_of::<Op>()` is pinned at 12 by an assertion in `ir/mod.rs`; `Store { index: u32, at: u32, src: u16 }` is 10 bytes plus a discriminant, so it fits beside the existing widest variant.
*What would falsify it.* A paired run moving `varlookup` by less than about 15%; or a frame whose slots grow under a cached index -- `PROCEDURE EXPOSE`, `INTERPRET` and a trap handler are the three shapes to check, and `RootSet`'s aliases are already chased at bind time.
*Risk to the sharing rule: real, and it is named in the code.* `compile.rs` says the store target is deliberately left unresolved so that "a stem, a compound tail and the `>=>` line stay one implementation, and resolving the name here would be the second one". **The rule is kept by making the slot an argument to the one `assign_expr_target`, never a second store path in `drive.rs`** -- which is what P2 did, at -10.7% on `varlookup` using the id-keyed map alone. A candidate that adds an `Op::StoreSlot` executed by its own code in the driver is out.

**4. Decide "is this a tagged small integer" without rendering the number to text and parsing it back.** Ceiling **14.8% on `arith`**; implied 2.6658x -> 2.27x.
*Mechanism.* `small_int_for` (`value.rs`) answers cheaply when `Number::plain_integer` accepts and otherwise calls `format_with` -- a full decimal render into a fresh `String`, 13.8% of the axis -- scans it for `.` and `E`, parses it back to an `i64`, and discards the string. Every arithmetic result on `arith` reaches that line. The fix is an exact predicate over the `Number`'s own exponent and digit vector that accepts the same set.
*The obvious shortcut is not the fix, and this is measured.* P3 simply returned `None` when `plain_integer` declined: **-16.3% on `arith`**, but `compound` +1.6% and `strings` +2.7% *slower*, both outside that run's noise. The probe genuinely accepts values `plain_integer` declines, and removing it turns those into heap objects.
*What would falsify it.* The replacement accepting a different set -- a differential divergence, or `compound` and `strings` getting slower again.
*Risk to the sharing rule: none.* One function in `value.rs`, below both engines.

**5. The clause boundary returns a 104-byte value.** Ceiling **11.4% on `varlookup`**, 6.5% `emptyloop`, 3.7% `alloc4c`, 1.6% `compound`, 1.4% `strings`, 1.0% `arith`; implied `varlookup` 3.7409x -> 3.31x, `alloc4c` 1.9793x -> 1.91x.
*Mechanism, and two of its three facts are read from the binary rather than from a profile.* `size_of::<Failure>()` is **104** and `size_of::<Raised>()` is **104**, read with `gdb -batch -ex 'print sizeof(rexx_exec::error::Failure)'` against the profiled binary -- so `Failure`'s width is `Raised`'s. The compiled clause epilogue in `run_ops::<false>` copies **0x68 = 104 bytes stack-to-stack, twice**, behind two discriminant tests, at `+3605`..`+3696` and `+3728`..`+3752`. That is where the profile's third fact lands: one instruction of that sequence, `movaps xmm1, xmmword [rsp + 0xd0]`, carries **509 of `varlookup`'s 4575 samples**. Boxing the large arm -- `Failure::Raised(Box<Raised>)` -- makes `Failure` pointer-sized and `Result<T, Failure>` fit in registers; `Raised` is built on the error path only, so the box costs nothing any benchmark reaches. The oracle's own per-clause loop, `RexxActivation::run`, is 1.8% self on `varlookup`, 2.1% on `compound` and 5.7% on `rexxcps`, and below the reported cut on the other three.
*What would falsify it, and it needs saying because this is the one candidate a profile proposes and cannot confirm.* Sampling attributes time to a stalling load; it does not prove the copy is what stalls. A run of aligned 16-byte loads reading back what 8-byte stores just wrote is the shape that blocks store-to-load forwarding, and the counter that would settle it, `ld_blocks.store_forward`, is not exposed by `perf` on this machine. **The confirming instrument is the change itself**: re-read the asm to check the copies are gone, then take the paired run. If the copies vanish and `varlookup` moves less than about 4%, the attribution was wrong and the entry says so.
*Risk to the sharing rule: none.* One type in `error.rs`; both engines return it.

**6. Resolve a builtin call at compile time instead of validating UTF-8, testing a hash set and scanning a table.** Ceiling **9.1% on `strings`** (`is_builtin` 5.4%, the linear `Iter<Builtin>::find` 3.7%); implied 10.5770x -> 9.62x.
*Mechanism.* `builtin::dispatch` runs `std::str::from_utf8(name).is_ok_and(|name| in_scope().contains(name))` and then `IMPLEMENTED.iter().find(...)`, on every call, answering a question the parser had the information to answer. For scale, `dispatch`'s whole subtree is 30.7% of `strings`, so about three tenths of the time in builtins is spent deciding which builtin to run.
*What would falsify it.* A paired run moving `strings` under about 4%.
*Risk to the sharing rule: real if the resolved index is put in `Op` only.* The resolution belongs where both engines read it -- the plan or the instruction -- not in the op stream.

**7. Stop binary-searching the line table once per stepped clause.** Ceiling **6.2% on `alloc4c`**, 4.5% `rexxcps`, 2.8% `varlookup`, about 1.5% on `compound` and `arith`; implied `alloc4c` 1.9793x -> 1.86x, `rexxcps` 7.3466x -> 7.02x.
*Mechanism.* `enter_stepped_clause` calls `clause_line` -> `ProgramSource::line_of`, a binary search over the line-offset table, unconditionally, because `SIGL` must stay correct whether or not `TRACE` is on. The line is a static property of the instruction and the oracle's instruction objects carry it.
*Its loudest axis is its least trustworthy one.* `emptyloop` reads 12.0% and `emptyloop` is the axis carrying the recorded between-build codegen sensitivity of about 15 instructions per pass, so the figures that should decide this are `alloc4c`'s and `rexxcps`'s.
*Risk to the sharing rule: real.* Putting the line on the instruction keeps one implementation; putting it in `Op` alone leaves the tree-walker computing it a second way.

#### What the profile ruled out, and one question it answers that was already open

* **`arith`'s decimal arithmetic is not the gap.** The two implementations are within a few per cent of each other in absolute time on that axis. Anything proposing to speed up long division is not a candidate at this distance.
* **`memcmp` on `compound` is not a differentiator.** The oracle spends a larger share of its own run there than this crate does.
* **`alloc4c`'s 2.0x remains a debt.** 74.5% of the oracle's time on that axis is under `newObject` and 60.5% under `collect()`.
* **Widening `Op` from 12 bytes to 16 is not worth queueing**, and this is the plan's own recorded open question answered with evidence rather than left for a build. `run_ops::<false>`'s **self** time -- the whole op-decode loop, the only thing a stride change can reach -- is **4.9% on `varlookup`, 1.9% on `compound`, 1.7% on `strings`, 1.4% on `rexxcps`, 1.1% on `arith`** and below the top-20 cut on `alloc4c`. A cache-line argument can move a fraction of that. At 2.0x to 10.6x from the bar, Unit 0's own rule -- "a candidate whose ceiling is 2% is not worth queueing at this distance" -- disposes of it. It is cheap to try and it is not on this queue; if someone tries it anyway, `varlookup` is the axis with anything to see and `emptyloop` is the axis least able to prove it.
* **The entry state's 3% IR-against-tree-walker wall-clock regression is not a memory-footprint effect.** Peak resident set, both arms, same wrapper: `compound` 40,640 KB against 41,688, `strings` 3,727,388 against 3,728,840, `varlookup` 149,728 against 150,184, `rexxcps` 1,798,548 against 1,799,196 -- within 0.3% on all four, with the IR arm the lower of the two every time. Whatever the IPC story is, it is not working-set size. Still not queued.

#### What this entry cannot say

* **It is one profiled run per axis per side.** A share here is a single sample of a distribution nobody characterised, and a re-profile will read a different small number. Every ceiling above is therefore a candidate's *hypothesis*, and the accept rule is still a paired interleaved wall-clock comparison against the immediately preceding binary.
* **Sampling attributes time; it does not attribute instructions.** Candidate 5 rests hardest on that distinction and says so in its own text.
* **The shares are not additive and no combination is predicted.** Allocator self time is inside every other subtree, and `phase-4d-attribution.md` measured P1, P2 and P3 both singly and together without being able to establish that no interaction exists.
* **Nothing here is a correctness statement.** Both sides printed identical bytes on every axis, which guards against profiling a run that did not do the work. No differential ran.
* **It changed no code.** No candidate was attempted, no prototype was built, and the binary profiled is byte-identical to entry 1's.
