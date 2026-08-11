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

### Entry 3 -- candidate 2 attempted and **accepted**: a small-integer path for `//`, `%`, `/` and `**`

**The loop's first attempt.** BASE is `d44543c6`, the state entries 1 and 2 measured.

#### The hypothesis, named before it was measured

`small_int_arith` (`eval.rs`) matched `Plus`, `Subtract` and `Multiply` and answered `None` for the other four operators. So `compound.rex`'s hot clause `k = i // tails` -- two tagged small integers under `DIGITS 9` -- converted both operands to a `Number` with a digit vector, long-divided, and allocated the result. On the IR arm it is worse than that: `Op::Arith`'s per-site hint goes sticky-general the first time a site falls through, so from iteration 2 onward that site did not even try.

The change is that one function plus a `small_int_power` helper beside it, and nothing else. **No fast path was added to the driver**, so the sharing rule is kept by construction: both engines still reach exactly one implementation, through `Interp::arith_small_int`.

#### Predicted movement, written down before the first block ran

| axis | predicted | the reasoning behind it |
|---|---|---|
| `compound` | **-35% to -52%**, central -45% | `phase-4d-attribution.md`'s P1 is this edit and measured -51.6%; entry 2's share for `Number::div` alone is 42.0% |
| `arith` | **0%**, band -2% to +2% | its `**` and `//` have non-integer operands, and its `/` sites stop trying the fast path after one iteration |
| `varlookup`, `strings`, `alloc4c`, `emptyloop`, `rexxcps` | **0%** | none of these programs contains any of the **four** operators this candidate adds, and `rexxcps`' single `%` is in its calibration section rather than its timed loop |

**One line of the candidate's own text in entry 2 is wrong and this entry corrects it rather than repeating it.** Entry 2 says `arith` is out of reach because "its divisions have non-integer operands". Two of its three divisions do have them -- `c ** 2 // 5` and the `**` inside it -- but `a = i / 3` and `c = i / 7` divide a tagged small integer by a small-integer literal, and one iteration in three of `i / 3` is *exact*, which the new `/` arm answers. What actually keeps `arith` out of reach on the measured arm is the per-site hint: `i / 3` is inexact at `i = 1`, the site is marked general, and it never tries again. The prediction was right; the reason given for it was not, and on the tree-walker arm -- where there is no hint -- it would not have held.

**Every axis predicted at 0% came in slightly positive, and that is a miss.** `compound`'s prediction held. The zeros did not: the direct interleave reads `arith` +1.52% and `varlookup` +1.77%, and the suite reads `emptyloop` +3.04% and `strings` +2.19% as well. The size is small and the mechanism turns out not to be arithmetic at all -- see the section on codegen below -- but it is written down here as a wrong prediction rather than absorbed into the result, because it is only visible as a miss by having been written first.

#### Build identity

| | |
|---|---|
| repo commit measured | `d44543c6` plus this entry's working-tree change to `eval.rs`, printed by the harness in all four blocks |
| BASE `rexx-run` | size=13819064, sha256 `f166bb30215747c9753379d2009011f8956c6695bd5efce130711a1999c98255` -- **the same binary entries 1 and 2 measured**, rebuilt from a clean `git status` and reproducing their fingerprint byte for byte |
| HEAD `rexx-run` | size=13816800, sha256 `08c55ab1d3a475838fa6311b4eb33a9e33504c75e0fde23881487abbda7c3450` |
| `rexx-bench-suite` | sha256 `4949a32114e8bf728598a05575ff02043920a75ef68fb8b89f1972edccc0c694` -- **entry 1's harness binary exactly**, and one binary for all four blocks |
| oracle | the same three objects entry 1 fingerprints, unchanged |

An earlier sitting was **started and discarded** before any of the numbers below were taken. Two doc comments in `eval.rs` were corrected while its first block was running; a doc comment shifts the debug line table and so changes the binary, and the earlier HEAD build's sha256 (`db2c35b2...`) is not this one's. Measuring a binary that is not the committed source is the defect that would have been, so the sitting was stopped, the source finished, and everything below re-taken.

#### The accept measurement: the two binaries alternating in one loop

This is the accept rule's literal shape -- both binaries inside one loop, on one machine state -- and it is **not** `rexx-bench-suite`, so it is reported as its own instrument rather than as a suite figure. Wall clock, `ulimit -v 8388608` on both arms, `REXX_ENGINE=ir`, one fresh empty working directory, and the base/head order **rotated every round** so a drift across the sitting cannot become a binary difference. 2026-08-11 14:26:27 to 14:29:24 +02:00, seven rounds per axis, load average 6.12 falling to 2.09.

`varlookup` is here as a control: it contains none of the four operators, so a change on it is not the mechanism.

| axis | base median | head median | head / base | rounds with that sign |
|---|---:|---:|---:|---:|
| `compound` | 6.7235 s | 3.3663 s | **-49.93%** | **7 of 7** |
| `arith` | 3.0583 s | 3.1047 s | **+1.52%** | 7 of 7 |
| `varlookup` | 4.4589 s | 4.5376 s | **+1.77%** | 6 of 7 |

Per-round, `compound` reads -50.7%, -49.1%, -49.2%, -50.0%, -50.8%, -50.4%, -49.6%. There is no round in which it is not about half.

#### The same comparison in the configuration fixed at the top of this file

Four blocks of `rexx-bench-suite`, one contiguous sitting 2026-08-11 14:30:11 to 14:57:19 +02:00, **ABBA over the two binaries** -- base, head, head, base -- each block itself an oracle/this-crate interleave over 9 sampled pairs. Every block exited 0, none printed a "not a baseline" section, and **every axis in every block printed the same bytes on both sides and under both binaries**, `compound` included at `5000000`. One-minute load average at the four block boundaries: 1.24, 1.80, 2.49, 3.66, ending at 4.29.

This crate's median over the oracle's, recomputed from the two medians the harness prints:

| axis | b1 base | b2 head | b3 head | b4 base | base arm | head arm | **change** | this sitting's own base/head block gaps |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `alloc4c` | 1.9665 | 1.9725 | 1.9669 | 1.9800 | 1.9733 | 1.9697 | **-0.18%** | 0.69% / 0.28% |
| `arith` | 2.6363 | 2.6783 | 2.6900 | 2.5870 | 2.6117 | 2.6842 | **+2.78%** | 1.89% / 0.44% |
| `compound` | 5.8022 | 2.9027 | 2.9644 | 5.8956 | 5.8489 | 2.9336 | **-49.84%** | 1.60% / 2.10% |
| `emptyloop` | 3.1954 | 3.2891 | 3.2417 | 3.1427 | 3.1690 | 3.2654 | **+3.04%** | 1.66% / 1.45% |
| `strings` | 10.4237 | 10.5706 | 10.6244 | 10.3174 | 10.3705 | 10.5975 | **+2.19%** | 1.02% / 0.51% |
| `varlookup` | 3.6985 | 3.7515 | 3.7042 | 3.7015 | 3.7000 | 3.7279 | **+0.75%** | 0.08% / 1.27% |
| `rexxcps` (cps) | 7.2079 | 6.9590 | 6.6522 | 6.2200 | 6.7137 | 6.8056 | not usable | 14.72% / 4.51% |

**The last column is what the rest of the table has to be read against, and only `compound` clears it.** Every other axis moved by about the gap this sitting's own two same-binary blocks show, so the suite alone cannot call any of them. `compound` moved by thirty times its own gap.

**`rexxcps` produced no usable figure this sitting and is recorded as such rather than as a small movement.** The oracle's own median clauses-per-second fell monotonically across the four blocks -- 16368501, 15561756, 14936464, 14003945, a 14.5% decline -- so the ratio's denominator was drifting under it. This crate's own cps, which that drift does not touch, is 2270910/2251441 on base against 2236218/2245342 on head: -0.90%, itself inside the base arm's own 0.86% block gap.

**The machine was not as quiet as entry 1's.** In the ten minutes before the sitting, work outside this sandbox took the host to 96.5%, 96.9% and 96.5% busy in three samples, with load average climbing 1.29 -> 33.15 while `ps` inside the sandbox showed nothing running at all. The sitting was held until six consecutive ten-second samples read 85% idle or better, and started then. Load climbed again during it, which is what the `rexxcps` denominator is showing. The ABBA order removes a linear drift and removes nothing else.

#### What moved, against entry 1

Entry 1 puts `compound` at **5.8982x** on the IR arm. This sitting's base arm reads 5.8489x for the byte-identical binary in a different sitting -- 0.84% apart, well inside the roughly 4% between-run movement entry 1 quotes for that axis from `perf-baseline.md` with no code change at all. **Head is 2.9336x.**

Entry 2's ceiling construction for this candidate was 42.0% of the axis, implying 5.8982x -> 3.42x. The measured landing is **2.93x**, past that ceiling, for the reason the candidate's own text predicted: the share is the division alone, and the change also removes the operand conversion, the result allocation and the root pushes around it.

#### The hypothesis is confirmed by its own route, and the mechanism was checked directly

`perf stat -e instructions:u` on the IR arm, a different configuration from the sitting and quoted only as a mechanism check:

* **The fast path is genuinely entered.** A 200,000-iteration `k = i // 500` loop executes 2,041,601,642 instructions on base and 775,224,862 on head, printing the identical answer.
* **And one differential really was vacuous, which is why that check exists.** Operands taken from `word()` are heap strings and never carry the tag, so the 41,508-case grid enters the fast path nowhere: base 1,967,696,843 instructions against head 1,968,468,181, **+0.04%**, which is the codegen cost the untouched axes show and not a path change. The grid whose operands are loop control variables runs 235,254,533 against 196,003,199, **-16.7%**. Both grids are kept -- the first is a control on the general path, the second is the witness -- but only the second could have failed.
* **`compound` itself:** 75,348,780,466 instructions on base against 41,425,715,830 on head, -45.0%; cycles -49.5%, against the -49.9% wall.

#### The cost this candidate has on axes the four new operators never reach, which the predictions missed

`arith` and `varlookup` are each about 1.5% slower at head, and **neither reaches any of the four new arms on the measured engine** -- `arith` for the hint reason set out above, and `varlookup` because the only arithmetic it contains is `x = x + 1`, an operator that was already on the fast path and whose arm did not change. So this is not the mechanism doing extra work, and it was checked rather than assumed:

Medians of three runs per side, with each side's own full range beside them, because a single-run difference of a tenth of a per cent would be worth nothing:

| axis | base instructions | head instructions | change | base range | head range |
|---|---:|---:|---:|---:|---:|
| `arith` | 30,016,589,535 | 30,047,589,441 | **+0.10%** | 3.7e-4 | 1.7e-8 |
| `varlookup` | 66,671,661,164 | 67,070,660,474 | **+0.60%** | 1.4e-8 | 2.2e-8 |

`varlookup` executes 399 million more instructions for work that did not change, reproducible to eight figures. **The cause is codegen: `small_int_arith` grew, and its caller is inlined into `run_ops`, which is the one function every axis runs.** Cycles move further than instructions on both -- `arith` +1.18%, `varlookup` +1.92% -- so there is an IPC component on top of the instruction count.

**This is a real cost channel for every future candidate that touches the arithmetic arm**, and it is worth more than this entry's own arithmetic: a change that adds nothing to a hot path can still charge that path half a per cent by displacing it, and only an instruction count separates that from measurement noise.

#### Correctness, which outranks the number

**No divergence, on either engine.** Every probe below ran from a fresh empty directory under the wrapper `rust/CLAUDE.md` fixes, with stdout, stderr and exit status read separately.

* **41,508 cases** over `/`, `%`, `//` and `**`, operands from `0` to the tag's own limits including both signs, twelve `DIGITS` settings from 1 to 25, errors trapped and reported as their error number rather than aborting: **byte-identical to the oracle** on base and on head, and on `ir` and `tree-walker` alike. Base matching too is what makes it a control rather than a result.
* **15,891 cases** with the operands arriving as **loop control variables**, which is how a program gets a tagged small integer: byte-identical to the oracle on both engines. This is the grid that actually enters the fast path -- 235M instructions on base against 196M on head -- and the first grid, which did not, is why that check exists.
* The committed regression witness is `the_small_int_fast_path_answers_what_the_general_path_answers`, extended from three operators to seven: 25 operands squared by nine precisions by both `FORM`s, every fast answer compared against `rexx-num`'s own, with a **per-operator floor** so that `+` reaching the path thousands of times cannot satisfy the count on `/`'s behalf. Its `expect` on the general path's result is an assertion in its own right: a fast path that answered where the interpreter raises 42.3 or 26 fails on that line.
* Four new witnesses beside it, each pairing a refusal with its adjacent success: `/` declining `1 / 3` and taking `6 / 2`; a zero divisor declining on all three division operators; the sign rule (`-7 % 3` is `-2`, `-7 // 3` is `-1`, `7 // -3` is `1`, measured on the interpreter); and `**` declining a negative exponent and `2 ** 30` under `DIGITS 9` while taking it under `DIGITS 10`.
* **`**`'s guard rests on a read of the oracle's own algorithm, not on an inference.** `NumberString::power` reduces the exponent bitwise and works at `DIGITS` plus the exponent's digit count plus one, so every intermediate is the base raised to a prefix of the exponent and no wider than the result; a result needing no rounding is therefore reached without any. It also calls `prepareOperatorNumber` with `NOROUND`, which is why the shared operand guard is stricter than the interpreter for that one operator -- a decline costs speed and never an answer.

Gates: `cargo test --workspace` **1432 passed, 0 failed** in dev and **1432 passed, 0 failed** in release. BASE's count was **re-counted rather than inherited from the brief**: 1428 run, which is 1432 less the four witnesses added here. It was counted in a `git worktree` at `d44543c6`, where 27 of those 1428 *fail* -- every one of them because the tests that read the C++ `ootest/` tree resolve it as `crates/rexx-exec/../../../ootest`, which exists only beside the real checkout. That is a property of the worktree and not of BASE, and it is written down so the next person to re-count this way does not read it as a regression. `cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean from a `cargo clean`ed target directory with `Checking rexx-exec` confirmed in the log. The corpus runner and the dual-engine sweep are inside that count and reported 10 and 9 tests passed in the release run, read as counts rather than as exit statuses.

#### Disposition: **accepted**

The candidate's own falsification condition was a paired run moving `compound` by less than about 30%, or any differential divergence. It moved 49.9% with the sign holding in every one of seven direct alternations and in both suite alternations, and no differential diverged. **The hypothesis is confirmed by the route it named**, which the instruction count is the evidence for.

Accepted **with the cost recorded**: about +1.5% on `arith` and `varlookup`, +2% to +3% on `emptyloop` and `strings` in the suite arm figures, from codegen rather than from work. Against `compound` falling from 5.90x to 2.93x that is a trade worth making once. It is not obviously worth making six times, and the next candidate to touch this code should read its own instruction count on an axis it does not mean to move.

Commit: `3cd2e80d13532a45b406da3fb390b55ea4a93c9a`, read back from `git log` after committing.

#### What this entry cannot say

* **It is one sitting on one Linux host**, unpinned, on a machine whose governor cannot be fixed and whose host load is not under this session's control and was demonstrably not zero.
* **The two instruments disagree on the small numbers and neither settles them.** The direct interleave puts `arith` at +1.52% and `varlookup` at +1.77%; the suite puts them at +2.78% and +0.75%. Only the instruction counts are reproducible enough to assert a direction, and they measure work rather than time.
* **It re-profiles nothing.** `compound`'s shares are now all wrong -- the denominator moved by half -- and entry 2's queue entries for that axis are stale, exactly as the plan says they would be. The next candidate on `compound` needs a fresh profile, not entry 2's table.
* **It says nothing about `%` and `**` in a real program.** `compound.rex` uses `//` alone. The other three operators are correct by the grids above and unmeasured for speed, because no axis in this suite uses them.

### Entry 4 -- candidate 1, lever (a) attempted and **accepted**: a builtin's counted answer as the tag, not as a heap string

**BASE is `18a8e0c8`**, the state entry 3 left. **Lever (b) -- reclamation, a trigger policy, `phase-4d-retention.md`'s -16% on `strings` -- was not touched and remains queued with that figure intact.** The accept rule judges one change against one preceding binary, and an attempt moving both levers would produce an accepted change whose mechanism nobody could name.

#### The re-profile that chose the target, because entry 2's shares for `compound` are stale

Entry 3 halved `compound` and entry 2's queue counted the allocations it removed, so the axis was re-profiled at head before anything was picked. Same instrument as entry 2: `samply` 0.13.1 `--save-only`, 1 kHz, `REXX_ENGINE=ir` set in the script, fresh empty directory, `ulimit -v 8388608`, analysed through `pollard` with `expand_inlines`. `strings` 9324 ms, 9390 samples, `unsymbolicated_pct` 0.021%; `compound` 3446 ms against entry 2's 6765, which is entry 3's halving showing up in the profiler.

`strings`' allocator shares reproduce entry 2 -- glibc family 36.1% self, the arena side (`alloc_with_uncollected` + `ptr::write::<Slot>`) 8.0% -- but the **callers** are not what the queue assumed. The largest single malloc stacks on that axis are `Number::add_signed`, `truncated_to` and `aligned_to`, and `Interp::arith_general`'s whole subtree is **16.4%** of the axis. That is `total = total + length(joined)` running decimal addition on two values that are both small integers.

**It runs there because `LENGTH` hands its answer back as a heap string.** `builtin::string::length` was `interp.text(bytes.to_string().as_bytes())`, and `Interp::arith_small_int` answers only when **both** operands decode to `SmallInt`. One untagged operand costs the whole clause: `Number::from_i64` for the tagged side, a parse for the untagged one, digit vectors through the add, and a render back. Ten builtins had that shape -- `LENGTH`, `POS`, `LASTPOS`, `COMPARE`, `COUNTSTR`, `VERIFY` in `string.rs` and `WORDS`, `WORDINDEX`, `WORDLENGTH`, `WORDPOS` in `word.rs`.

#### The hypothesis, named before it was measured

`Interp::counted(value: usize)` returns `ObjRef::small_int(value)` and the ten sites call it. **This is not a new kind of value**: `Interp::literal` already inlines a source literal whose bytes are the canonical rendering of a small integer, so a `SmallInt` reaches every consumer in the crate today, and what changes is which of D15's two existing representations a counted answer starts in.

**There is no `DIGITS` admissibility test, and that is deliberate rather than omitted.** `Interp::number` needs one because a `Number` has already been rounded to the precision that produced it. A count has not been rounded by anything, and the interpreter renders it in full whatever `DIGITS` is in force -- measured on the oracle, `numeric digits 3 ; say length(copies('a',1234))` is `1234`, and only `length(...) + 0` is `1.23E+3`. The second half is `small_int_arith`'s existing `within_digits` guard declining an operand too wide for the precision, which is the same rule reached by the same code.

#### Predicted movement, written down before the first round ran

| axis | predicted | the reasoning behind it |
|---|---|---|
| `strings` | **-12% to -25%**, central -18% | `arith_general` is 16.4% of the axis and the whole of it is one clause falling off the small-integer path for want of a tag; plus `LENGTH`'s and `POS`'s own render, copy and slot |
| peak RSS, `strings` | **-25% to -45%** | five heap objects per iteration become three |
| `rexxcps` | -2% to -8% | calls `LENGTH`, `WORD` and `SUBSTR`, but diffusely |
| `arith`, `compound`, `varlookup`, `alloc4c`, `emptyloop` | **0%**, band -1% to +1% | none of these programs calls a builtin whose result this changes |

#### Build identity

| | |
|---|---|
| BASE `rexx-run` | size=13816800, sha256 `08c55ab1d3a475838fa6311b4eb33a9e33504c75e0fde23881487abbda7c3450` -- **entry 3's HEAD binary reproduced byte for byte** from a clean `git status` at `18a8e0c8`, after `cargo clean -p rexx-exec -p rexx-bench --release` |
| HEAD `rexx-run` | size=13756944, sha256 `0daf0817724aae9b08812096d19cae0c28f3ee0ed100f37029b053f4c5f380f2` |
| oracle | the same three objects entry 1 fingerprints, unchanged |

**The measured HEAD binary is the committed source's release build, and that was checked rather than assumed** -- entry 3 discarded a sitting over exactly this. The witness and the type alias added after the sitting are `#[cfg(test)]`, so rebuilding the release binary from the committed tree reproduces `0daf0817` byte for byte; it was rebuilt twice more, once after a full `cargo clean`, and hashed each time.

#### The accept measurement: the two binaries alternating in one loop

The accept rule's literal shape, and **not** `rexx-bench-suite`, so it is reported as its own instrument. Wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, one fresh empty working directory, the base/head order **rotated every round**. 2026-08-11 15:22:59 to 15:29:47 +02:00, seven rounds per axis, load average 1.51 to 1.93 across the sitting, every run exiting 0. Every axis but `rexxcps` printed a byte-identical stdout under both binaries in all seven rounds, hashed per run.

| axis | base median | head median | head / base | rounds with that sign |
|---|---:|---:|---:|---:|
| `strings` | 9.2585 s | 6.9634 s | **-24.79%** | **7 of 7** |
| `alloc4c` | 2.3520 s | 1.6523 s | **-29.75%** | **7 of 7** |
| `varlookup` | 5.0142 s | 4.5163 s | -9.93% | 7 of 7 |
| `compound` | 3.5019 s | 3.2700 s | -6.62% | 7 of 7 |
| `emptyloop` | 2.9716 s | 2.8985 s | -2.46% | 7 of 7 |
| `arith` | 3.1270 s | 3.1216 s | -0.17% | 5 of 7 |

`rexxcps` is not read as wall time, for entry 1's reason. Both binaries self-calibrated to the identical `100 x 100`, so their own clauses-per-second figures are comparable: five rounds, order rotated, base median **2,270,198** against head **2,322,521** -- **+2.30%**, head ahead in all five.

#### Two of the seven predictions were wrong, and only one of them was avoidable

**`alloc4c` at -29.75% against a predicted 0%.** `alloc4c.rex`'s hot clause is `total = total + 3 + length(s)`. It calls `LENGTH`. The prediction said "none of these programs calls a builtin whose result this changes" without reading the program, and the program is forty lines of commentary above six lines of code. That is the whole error; nothing subtle went wrong.

**`varlookup` -9.93%, `compound` -6.62% and `emptyloop` -2.46%, against a predicted 0% -- and these are *not* this candidate's mechanism.** Those three programs call no builtin at all. `perf stat -e instructions:u,cycles:u`, three runs per side, medians:

| axis | instructions base | instructions head | change | cycles change | wall change |
|---|---:|---:|---:|---:|---:|
| `strings` | 86,239,817,791 | 70,096,926,555 | **-18.72%** | -24.36% | -24.79% |
| `alloc4c` | 16,008,876,899 | 12,017,611,459 | **-24.93%** | -28.70% | -29.75% |
| `varlookup` | 67,034,532,212 | 67,001,311,086 | **-0.05%** | **-10.07%** | -9.93% |
| `compound` | 41,381,778,014 | 41,368,280,804 | **-0.03%** | **-7.29%** | -6.62% |
| `emptyloop` | 41,353,097,384 | 41,337,639,544 | **-0.04%** | **-2.04%** | -2.46% |
| `arith` | 30,053,874,969 | 30,052,572,617 | -0.00% | -0.18% | -0.17% |

**Three axes execute the same instruction stream and retire it in 2% to 10% fewer cycles.** The binary shrank by 59,856 bytes when ten call sites stopped rendering integers to text, and this is what that bought: a code-layout effect, entry 3's displacement channel running the other way and an order of magnitude larger than the +0.60% entry 3 measured on `varlookup`. **It is a windfall this candidate did not aim at and cannot claim**, and the next change that moves the binary's size can take it back.

**`arith` is what makes that a finding rather than a suspicion about the sitting.** It calls no builtin either, and it shows neither an instruction change nor a cycle change -- so the effect is axis-specific, not a sitting-wide bias favouring whichever binary ran second.

#### Peak resident set, which is the candidate's own falsification check

`/usr/bin/time -f %M`, same wrapper, one run per side.

| axis | base | head | change |
|---|---:|---:|---:|
| `strings` | 3,728,016 KB | 2,511,244 KB | **-32.64%** |
| `rexxcps` | 1,798,660 KB | 1,741,192 KB | -3.20% |
| `varlookup` | 149,404 KB | 149,260 KB | -0.10% |
| `compound` | 40,332 KB | 40,836 KB | +1.25% |
| `arith` | 439,260 KB | 440,676 KB | +0.32% |

The candidate's stated falsification was a paired run moving `strings` under about 10%, **or peak RSS not falling when it lands** -- which would have said the allocations counted were not the ones removed. It moved 24.79% and the resident set fell by a third, so neither fired.

#### Correctness, which outranks the number

**No divergence, on either engine.** Every probe ran from a fresh empty directory under the wrapper `rust/CLAUDE.md` fixes, stdout, stderr and exit status read separately.

* **6,720 cases** -- the ten builtins over sixteen call shapes, ten subjects (empty, `0`, `00`, a 1234-byte string, an embedded-blanks string), six needles and seven `DIGITS` settings from 1 to 20 -- each answer then put through twenty uses: bare `say`, all six arithmetic operators, `=`, `==`, `<<`, `DATATYPE`, a compound tail key, `FORMAT`, `ABS`/`SIGN`/`TRUNC`, `MAX`/`MIN`, `RIGHT`/`LEFT`, concatenation, `PARSE VALUE` with a column pattern, and `WORD`, with syntax trapped and reported as its own `rc`. 126,084 output lines, **byte-identical to the oracle on stdout and stderr** on base and on head, and on `ir` and `tree-walker` alike. Base matching too is what makes it a control rather than a result.
* **The grid can fail, and that was checked rather than argued.** `COUNTSTR` mutated to answer one fewer diverges from the oracle on 1,031 lines. The mutation was reverted from a copy and the release binary rehashed to `0daf0817`.
* **What the grid cannot do is see the representation**, and that is the point of the change: `Body::Text{b"46"}` and `SmallInt(46)` are observationally identical, so no differential can tell them apart and none of these 6,720 cases would notice all ten call sites being put back. The representation is pinned by an assertion instead.
* A `TRACE I` probe reproduces the oracle's own `>F> LENGTH => "5"` and the `>O> "+" => "6"` after it, byte for byte, before and after.

The committed witness is `a_counted_answer_is_tagged_rather_than_a_heap_string`, which goes through `builtin::dispatch` -- so one enumeration reaches `word.rs`'s four names as well as `string.rs`'s six -- and asserts the **handle** decodes to the expected `SmallInt`, not only the bytes. Its adjacent success is `SUBSTR('012345',1,3)`, which must stay a heap string: `012` is bytes that are the value, with no integer behind them, and that is what pins the rule to *counted* answers rather than to "anything that looks like a number".

**It adds coverage, which is a separate claim from being able to fail and was measured separately.** `WORDS` put back to `interp.text(...)` turns the witness red; the entire workspace suite run *without* the witness (`--skip`, `--no-fail-fast`) stays green under that same mutation. `word.rs` was restored from a copy and `sha256sum -c`'d, never `git checkout --`.

Gates: `cargo test --workspace` **1433 passed, 0 failed, 4 ignored** in dev and **1433 passed, 0 failed, 4 ignored** in release, run counts read rather than exit status alone. Inside the release run, the corpus runner reported 10 passed and the dual-engine sweep 9 passed. **BASE was re-counted rather than inherited**: `cargo test --workspace -- --list` in a `git worktree` at `18a8e0c8` enumerates 1436, of which 4 are `#[ignore]`, so BASE runs **1432** -- agreeing with entry 3, and reached by listing rather than running, which is what avoids the 27 `ootest/`-path failures entry 3 records for that worktree. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean from a full `cargo clean`, with `Checking rexx-exec` confirmed in the log -- and it was not clean the first time: the witness's case table tripped `clippy::type_complexity`, which is why the run from a cold target directory is the one that counts.

#### Disposition: **accepted**

The hypothesis is confirmed by the route it named, and the instruction count is the evidence: the two axes that call `LENGTH` retire 18.7% and 24.9% less work, the resident set on `strings` falls by a third, and no differential diverged.

Accepted **with the windfall disclaimed rather than banked**: of the seven axes moved, two moved because this candidate removed work and three moved because the binary got smaller. `varlookup`, `compound` and `emptyloop` should be read as `18a8e0c8` + this build's layout, not as a result about them.

Commit: `d9bf260ec7873e2b2f0b30624eb4b9d7efb3df5c`, read back from `git log` after committing.

#### What was ruled out, and why, since the queue asked for one representation

* **An inline digit buffer for `Number` -- the `smallvec` dead end the plan and this file both cite and neither can produce.** Not attempted, and this entry licenses nothing about it in either direction; the citation is still a claim with no measurement behind it. The reason it was not chosen: at head the `Number` allocations on `strings` are reached *because* a counted answer arrives untagged and forces the general decimal path. Removing the reason removes them wholesale -- 16.1 billion instructions -- where making each one cheaper leaves the path, the parse and the render in place. It is also a rewrite of `digits: Vec<u8>` across five files of `rexx-num`, against one new constructor and ten call sites here.
* **Inlining a short string into `Body::Text`.** `Body` is 80 bytes, dominated by the cold `Stem` variant, so an inline buffer would cost nothing in width -- but the builtins **build** their results in a `Vec` (`builtin::buffer`, `Vec::new()` and extend) and hand it over by move, so the malloc has already happened before `Body` sees it. Inlining would mean rewriting each builtin's internals, not changing one representation.
* **`required_string`'s `[u8]::to_vec`, 15.5% of `strings`** -- six copies of an argument per iteration, taken so the borrow of `interp` can end. Real, and larger than several queued candidates, but it is a borrow-structure change and not a representation, so it belongs to a candidate of its own rather than to this lever.

#### What this entry cannot say

* **It is one sitting on one Linux host**, unpinned, on a machine whose governor cannot be fixed.
* **It did not run `rexx-bench-suite`, so it carries no oracle ratios.** Every figure above is against the immediately preceding binary, which is what the accept rule asks for and all it asks for. Where `strings` and `alloc4c` now sit against the oracle is unmeasured, and entry 2's warning stands either way: `alloc4c`'s oracle ratio has an inflated denominator and no candidate should be aimed at it.
* **The layout windfall is reproducible for this pair of binaries and nothing more.** Three perf reps and seven wall rounds agree on it; whether it survives the next build is not something this entry measured, and the honest expectation is that it does not.
* **It changed ten call sites and no others.** `state.rs` and `datetime.rs` render an integer answer to text the same way at fourteen further sites (`DIGITS`, `FUZZ`, `QUEUED`, `LINES`, `DATE('B')`, `TIME('T')` among them). None is reached by any axis in this suite, so none was changed and none was measured.
* **`%` and `**` on a counted operand are correct by the grid above and unmeasured for speed**, because no axis in this suite puts one there.

### Entry 5 -- candidate 3 attempted and **accepted**: every simple-variable write and the loop control bound to an integer slot

**BASE is `217006fe`**, the state entry 4 left. Its `rexx-run` is entry 4's HEAD binary reproduced byte for byte, which is what says the two intervening commits are the record and nothing else.

#### The re-profile that chose the sites, because entry 2's shares for `varlookup` are two accepted changes old

Entry 4 explicitly disclaims `varlookup`'s movement in its own sitting as a code-layout windfall it cannot claim, so this axis's shares at head were unknown and were measured before anything was picked. Same instrument as entries 2 and 4: `samply` 0.13.1 `--save-only`, 1 kHz, `REXX_ENGINE=ir` set in the script, fresh empty directory, `ulimit -v 8388608`, analysed through `pollard` with `expand_inlines`. 4471 ms, 4468 samples, `unsymbolicated_pct` 0.0023%; the run's duration sits inside the head wall times below.

A share is a subtree total unless it says self.

| block at head | share | samples |
|---|---:|---:|
| `Interp::assign_expr_target` -- **site (i), the store side** | 26.9% | 1203 |
| ... of which `Interp::slot_of`, the name-keyed `HashMap` | 20.0% | 894 |
| ... of which `malloc`, which is the `Vec<u8>` copy of the name | 2.9% | 130 |
| `bind_control` -> `slot_of` -- **site (ii), the control variable's write** | 7.6% | 338 |
| `loop_advance` -> `read_at`, the id-keyed map -- **site (ii), its re-read** | 5.9% | 262 |
| addressable total | 40.4% | 1803 |

**All three of entry 2's sites are still there, and one of them turns out not to be a site of its own.** Entry 2 lists "(iii) the id-keyed map on reads the compiler could not resolve, 6.3% of the axis" separately from the loop control variable. At head every `read_at` sample that is not under `Op::Load`'s own `read_symbol` (2.8%, and that one already carries a compile-time slot) is the control variable's re-test read. So (iii) and (ii)'s read half are one block, 5.9%, and this entry takes **(i) and (ii)**; there is no third thing left to take on this axis. That is a finding about the queue rather than a shortfall.

#### The hypothesis, named before it was measured

`Op::Store { index, src }` carried no slot, so `assign_expr_target`'s `ExprKind::Variable` arm called `code.symbols.name(*id).as_bytes().to_vec()` and looked *that* up in `Plan::names`. The `Vec` existed only so `trace_assignment` could borrow it. `bind_control` did the same lookup once per loop pass for a variable whose slot is fixed when the loop is entered, and `loop_advance`'s re-test read went through the id-keyed map beside it.

The change is that `Op::Store` carries a `PlanSlot` -- `ReadSlot` renamed, because the resolution is the same one for a read and a write -- and that `LoopState::Controlled` and `LoopState::OverOnce` carry the control variable's slot, taken once at loop entry by `control_slot`.

**The sharing rule is kept by construction, and that is the whole of the design.** The slot is an **argument** to the one `Interp::assign_expr_target`, which both engines still enter: `Op::Store`'s arm in `drive.rs` passes what its op carries, the tree-walker's `Assignment` arm passes `None`, `PARSE` passes `None`, and `bind_control`'s stem and compound arms pass `None`. **No store path was added to the driver.** Only the `Variable` arm reads the argument, because it is the only one that writes a slot by name at all -- a stem write is `stem_assign` under a name and a compound write resolves a tail key at the write site.

**`control_slot` reads the plan's map and never `Interp::slot_of`**, which is what makes it free of side effects: `slot_of` *grows* the frame for a name nobody has bound, and doing that at loop entry rather than at the first write would bind a name earlier than the interpreter does. A control it cannot answer for leaves both the write and the re-read resolving their own slot exactly as before.

#### Predicted movement, written down before the first round ran

| axis | predicted | the reasoning behind it |
|---|---|---|
| `varlookup` | **-22% to -34%**, central -28% | 40.4% addressable above, less `set_slot`, the frame read and the write itself, which all stay |
| `compound` | -8% to -18%, central -12% | one store plus a controlled loop per pass; entry 2 gave this candidate 10.3% there |
| `emptyloop` | -8% to -20%, central -12% | loop control only, no store; `loop_advance` was 53.9% of that axis in entry 2 |
| `alloc4c` | -5% to -15%, central -9% | one store plus a controlled loop |
| `arith` | -4% to -12%, central -7% | entry 2 gave 2.3%, taken before the loop-control half was part of the candidate |
| `strings` | -3% to -10%, central -6% | entry 2 gave 4.6% |
| `rexxcps` | 0% to -8% | assignments and a loop, diffusely |

**No axis was predicted at 0%, so this candidate has no untouched-axis control of its own**, and the displacement channel entry 3 discovered cannot be read here the way entry 3 read it. What replaces it is an instruction count on **every** axis, beside cycles and beside wall: entry 4's layout windfall was three axes retiring the *same* instruction stream faster, and a change that moves instructions everywhere is separated from that by the instruction column rather than by an untouched axis.

**Two predictions were beaten and none was missed.** `varlookup` came in at -41.2% against a -34% band edge and `emptyloop` at -31.4% against -20%. Both are the same error: the prediction counted the profiled blocks and not the `Vec<u8>` allocation and free on the traced-name copy, which `emptyloop` pays once per pass with no assignment in its body at all.

#### Build identity

| | |
|---|---|
| BASE `rexx-run` | size=13756944, sha256 `0daf0817724aae9b08812096d19cae0c28f3ee0ed100f37029b053f4c5f380f2` -- **entry 4's HEAD binary reproduced byte for byte**, from a clean `git status` at `217006fe` |
| HEAD `rexx-run` | size=13762264, sha256 `ea3b80c84e11c5a85ba8b8efede9872f0226b57e8f69d479957202d49bfc3922`; the binary **grew** by 5,320 bytes |
| oracle | the same three objects entry 1 fingerprints, re-hashed here and unchanged: `bb5bb8cc...`, `42136c40...`, `3536b763...` |

**The measured HEAD binary is the committed source's release build, checked rather than assumed**, and the check was needed. A first sitting was taken, and then doc comments were corrected, a `debug_assert!` was added to `bind_control` and `cargo fmt` reflowed two hunks -- none of which changes what the release build executes, all of which move its sha256. **Everything below is re-taken on `ea3b80c8`**, and the source was reproduced back to it by rebuilding and comparing the hash rather than by reading the diff.

#### The accept measurement: the two binaries alternating in one loop

The accept rule's literal shape, and **not** `rexx-bench-suite`, so it is reported as its own instrument. Wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, one fresh empty working directory, the base/head order **rotated every round**. 2026-08-11 16:37:14 to 16:41:58 +02:00, seven rounds per axis, load average 1.89 to 2.38 across the sitting, every run exiting 0. All 42 rounds printed a byte-identical stdout under both binaries, hashed per run.

| axis | base median | head median | head / base | rounds with that sign |
|---|---:|---:|---:|---:|
| `varlookup` | 4.4587 s | 2.6200 s | **-41.24%** | **7 of 7** |
| `emptyloop` | 2.9010 s | 1.9895 s | **-31.42%** | **7 of 7** |
| `compound` | 3.2301 s | 2.7969 s | **-13.41%** | **7 of 7** |
| `strings` | 6.9198 s | 6.2433 s | -9.78% | **7 of 7** |
| `alloc4c` | 1.6344 s | 1.5291 s | -6.44% | **7 of 7** |
| `arith` | 3.1570 s | 2.9624 s | -6.16% | **7 of 7** |

Per-round on `varlookup`: -41.5%, -41.1%, -41.5%, -41.4%, -40.9%, -41.2%, -41.2%. There is no round in which it is not about four tenths.

`rexxcps` is not read as wall time, for entry 1's reason. Both binaries self-calibrated to the identical `100 x 100`, so their clauses-per-second figures are comparable: five rounds, order rotated, base median **2,307,138** against head **2,372,902** -- **+2.85%**, head ahead in all five.

**A second sitting, 20 minutes earlier, on a binary differing from `ea3b80c8` in doc comments alone, read -41.41%, -31.12%, -13.27%, -8.93%, -5.85% and -5.53% on the same six axes.** It is not the accept measurement -- it timed a binary that is not the committed source -- but two sittings agreeing within 0.7 of a point on every axis is what this machine's reproducibility looks like at this size, and that is worth more here than a discarded sitting usually is.

#### Instructions beside cycles beside wall, which is what says this is work rather than layout

`perf stat -e instructions:u,cycles:u`, three runs per side per axis, arms alternating, medians. A different configuration from the sitting above and read for work rather than for time.

| axis | instructions base | instructions head | change | cycles change | wall change |
|---|---:|---:|---:|---:|---:|
| `varlookup` | 66,999,520,289 | 43,092,993,940 | **-35.68%** | -42.05% | -41.24% |
| `emptyloop` | 41,351,965,597 | 28,127,668,706 | **-31.98%** | -31.15% | -31.42% |
| `compound` | 41,380,973,834 | 37,005,953,886 | **-10.57%** | -18.64% | -13.41% |
| `strings` | 70,141,816,452 | 62,900,476,008 | **-10.32%** | -8.95% | -9.78% |
| `alloc4c` | 11,926,274,026 | 10,697,093,981 | **-10.31%** | -12.26% | -6.44% |
| `arith` | 30,020,655,807 | 28,865,243,876 | **-3.85%** | -4.06% | -6.16% |

**Every axis executes measurably fewer instructions, so none of this is entry 4's windfall running again**, and the binary grew rather than shrank. What the columns do *not* settle is the remainder: on four of the six the cycle move is larger than the instruction move, by 6.4 points on `varlookup` and 8.1 on `compound`. Removing a hash of a byte string and the dependent load behind it is exactly the shape that raises instructions per cycle, but **nothing here measures IPC and this entry does not claim it** -- `alloc4c` and `strings` move the other way, and no mechanism offered covers both directions.

#### Which of the two sites did the work, measured rather than apportioned

A third binary was built from the committed source with `control_slot` answering `None` for everything, so site (ii)'s two resolutions are off and site (i) plus the per-pass name copy remain. Instructions, three runs per side, medians.

| axis | base | site (i) only | both sites | site (i) alone | both |
|---|---:|---:|---:|---:|---:|
| `varlookup` | 67,018,860,368 | 51,214,434,083 | 43,092,993,940 | **-23.58%** | -35.70% |
| `compound` | 41,421,553,228 | 39,201,392,608 | 37,005,953,886 | **-5.36%** | -10.66% |
| `emptyloop` | 41,271,104,002 | 38,872,954,683 | 28,127,668,706 | **-5.81%** | -31.85% |

`varlookup` is about two thirds the store side, `compound` about half each, and `emptyloop` is site (ii) almost entirely -- its loop body is a `NOP` and it contains no assignment at all, so its -5.81% under the site (i) build is the `Vec<u8>` name copy in `loop_advance`'s re-test and nothing else. **That is the piece the prediction missed**, and it is why two axes came in past their bands.

#### Peak resident set, which moves nowhere and is recorded for that

`/usr/bin/time -f %M`, same wrapper, one run per side: `strings` 2,511,212 KB against 2,511,112, `varlookup` 149,124 against 149,428, `alloc4c` 392,564 against 391,156, `arith` 438,420 against 438,176, `emptyloop` 194,692 against 196,980, `compound` 39,620 against 40,048. Within 1.2% on every axis and in both directions.

**This candidate removes lookups and one small per-clause allocation, not retained storage**, so a resident set that had fallen would have wanted explaining. Entry 4's falsification -- "peak RSS not falling when it lands" -- is that candidate's and not this one's.

#### Correctness, which outranks the number

**No divergence, on either engine.** Every probe ran from a fresh empty directory under the wrapper `rust/CLAUDE.md` fixes, stdout, stderr and exit status read as three separate descriptors.

* **The three shapes the candidate's own text names were probed first**, because a cached index that survives a frame growing is a wrong-answer defect and outranks any number here. `INTERPRET` binding new names inside a loop whose control slot was taken before it; `PROCEDURE EXPOSE` in a callee that then grows its own frame; a trap handler that assigns and grows the frame with the loop continuing after it; `DROP (v)` binding a name at run time; a loop **inside** an `INTERPRET` fragment, whose control slot comes from the fragment's own translated map; a `TRACE I` run over a store, a control write and a compound target; and stem, compound and `PARSE` targets together. Byte-identical to the oracle on `ir` and `tree-walker` alike.
* **A 600-program grid**, crossing eight loop-control shapes by five write targets by five frame-growth events by three trace settings, each program its own process so a raising case aborts only itself. Run against BASE and against HEAD, on both engines: **all 2400 descriptors byte-identical between the two binaries**, and the divergence-from-oracle set identical line for line, 154 of 1200 program-and-engine comparisons on both. Those 154 are pre-existing gaps -- `CONDITION('D')`, a missing `>V>` on `DROP (v)`'s indirect value, a trace difference in a `SIGNAL ON SYNTAX` handler -- and the remaining **1046 match the oracle on BASE and still match on HEAD**.
* **The grid can fail, and that was measured rather than argued.** `write_slot` answering one past the plan's slot takes the divergence count from 154 to 649; `control_slot` answering one past takes it to 800. Both mutations were reverted by `cp` from a copy and the release binary rehashed to `ea3b80c8`.

The committed witnesses are `a_resolved_slot_still_names_its_variable_after_the_frame_grows`, which runs the three growth shapes in one program on **both** engines against bytes measured from the oracle, and `a_compound_control_resolves_its_tail_on_every_pass` beside it as the adjacent case -- a control whose slot must *not* be resolved early, where the body moves which tail it is and the oracle answers `8` where a resolved-once implementation answers `4`. Two compile-time witnesses in `golden_tests.rs` pin the emitted form: that a simple target carries the plan's slot while a stem and a compound target carry none, and that the slot a write carries comes from the plan rather than from the write's position in the stream.

**Two coverage measurements, and one of them is negative.** A `control_slot` off by one reddens the growth witness *and* dozens of existing loop tests, so that witness is not the catcher for a wrong slot -- what is unique to it is the growth. And **widening `control_slot` to answer for a compound control leaves the entire workspace suite green**, including the witness written as its pair: `at` is read in `bind_control`'s `Simple` arm alone, selected by the same `shape_of` predicate, so the value is computed and discarded. That filter is unobservable, no test can pin it, and both the function and the test now say so rather than implying otherwise.

Gates: `cargo test --workspace` **1437 passed, 0 failed, 4 ignored** in dev and **1437 passed, 0 failed, 4 ignored** in release, run counts read rather than exit status alone. Inside the release run the corpus runner reported 10 passed and the dual-engine sweep 9 passed. **BASE was re-counted rather than inherited, and the brief's figure was one behind**: `cargo test --workspace -- --list` in a `git worktree` at `217006fe` enumerates 1437, of which 4 are `#[ignore]`, so BASE runs **1433** -- entry 4's own figure for `d9bf260e`, which `217006fe` sits on top of as a record-only commit. The brief said 1432, which was BASE for entry 4 rather than for this one. HEAD lists 1441, exactly four more, which are the four witnesses above. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean from a full `cargo clean`, with `Checking rexx-exec` confirmed in the log.

#### Disposition: **accepted**

The candidate's own falsification was a paired run moving `varlookup` by less than about 15%, **or a frame whose slots grow under a cached index**. It moved 41.24% with the sign holding in all seven alternations, no probe of the three named growth shapes diverged, and the 600-program grid is byte-identical to BASE across every descriptor. **The hypothesis is confirmed by the route it named**, and the instruction count is the evidence: 23.9 billion fewer instructions on `varlookup`, split two-to-one between the two sites by a third build.

Entry 2's ceiling construction for this candidate was 41.7% of the axis. The measured wall landing is 41.24%, which is *at* that ceiling while the instruction count moved 35.68% -- so unlike entry 3, this one did not overshoot its share, and the gap between the two columns is the part this entry cannot attribute.

Commit: `58dd6a24bc98d14e3f1473d90a4230203dc797da`, read back from `git log` after committing.

#### What this entry cannot say

* **It is one sitting on one Linux host**, unpinned, on a machine whose governor cannot be fixed.
* **It did not run `rexx-bench-suite`, so it carries no oracle ratios.** Every figure above is against the immediately preceding binary, which is what the accept rule asks for and all it asks for. Where `varlookup` now sits against the oracle is unmeasured, and entry 4's disclaimer means it was already unknown going in.
* **The cycle move exceeds the instruction move on four of six axes and this entry does not explain it.** IPC was not measured. The direction is what removing memory-dependent work does, and two axes go the other way.
* **The tree-walker gets nothing from this and was not measured for it.** It passes `None` at every site, so its store still resolves its own slot; the only thing it gains is the `Vec<u8>` name copy removed from `assign_expr_target` and from `loop_advance`, which is shared. Every figure above is the IR arm.
* **`Op` did not grow and the question of whether it should is untouched.** `Store { index: u32, at: PlanSlot, src: u16 }` is 10 bytes plus a discriminant and the `size_of::<Op>() == 12` assertion still holds unchanged.
* **The `DO OVER` control's slot is carried and never exercised by a benchmark.** `LoopState::OverOnce` binds once, so the resolution saves one lookup per loop rather than one per pass; it is correct by the grid and worth nothing measurable.

### Entry 6 -- candidate 1, lever (b) attempted and **accepted**: reclamation, and a loop's per-pass roots released per pass

**BASE is `bebf38ee`**, the state entry 5 left. Its `rexx-run` reproduces entry 5's HEAD binary byte for byte from a clean `git status`, which is what says the intervening commit is the record and nothing else.

**This is a defect fix that is also the queue's largest remaining candidate**, and the two framings do not agree about how to read the result. `phase-4-exclusions.txt` carried both causes under KNOWN GAPS; that row is now marked closed in place, with the re-measured figures, per that file's own CLOSED DEFECTS convention.

#### The framing, corrected before anything was written

**Nothing triggered a collection at all.** `Heap::collect` had two production callers: `Interp::alloc_with` gated on `stress_collect`, which only `run_program_collect_every_alloc` sets, and the `GC('Force')` builtin. So this was not a "collect more often" change.

**And a trigger alone would not have fixed the growth.** `Interp::loop_advance` pushed one root per pass for a counted loop's control variable and `Interp::eval_condition` one per `WHILE`/`UNTIL` test, both into the single `step_in_temps_frame` belonging to the whole `DO` instruction. Those roots are live by definition, so a collector cannot reclaim through them.

**Both were still exactly as `phase-4d-retention.md` measured them, re-checked at BASE** before anything was touched, 8,000,000 iterations, `/usr/bin/time -v` under `ulimit -v 8388608` from a fresh empty directory, every run exiting 0 and printing `ok`: `do n; nop; end` flat at 2,484 KB, `do i = 1 to n; nop; end` 64,708 KB, `do n while zz; nop; end` 64,680 KB, `do n until zz; nop; end` 63,688 KB, `do n; yy = 'abc'; end` 1,000,080 KB.

#### The change

**Cause A -- a trigger policy, and it is fixed rather than searched.** `Interp::alloc_with` collects when `Heap::live_count()` reaches a watermark, and sets the next watermark to twice the survivors, floor `COLLECT_FLOOR` = 65,536 objects. Two properties are what it was chosen for, and both are arguments rather than measurements: the peak is bounded by a multiple of the *live set* instead of by the program's total allocation, and the collector's total work is bounded by a constant times the program's total allocation, because between two collections the program must allocate at least as many objects as the first left alive. It is the shape `phase-4d-retention.md`'s prototype used, kept so this entry's landing can be read against that document's prediction. **It was measured once and not tuned.** Unit 0's rule is the reason: a threshold adjusted until a benchmark number looked right would not survive a different workload, and the record would stop being evidence.

**Cause B -- both sites, not one.** `loop_advance`'s `Controlled` arm opens a `RootSet` frame before its per-pass pushes and pops it once `bind_control` has written the new control value into the variable's own storage; `eval_condition` opens one around its own push and pops it before answering. **The `eval_condition` site was taken as well**, and the brief left that optional. It is two lines, it is the identical shape, and `phase-4-exclusions.txt`'s own text says a fix to `loop_advance` alone "turns every axis green and leaves `DO WHILE` and `DO UNTIL` growing without bound". It also **cannot confound the numbers below**: no benchmark axis contains a `WHILE` or an `UNTIL`, so that half of the change is invisible to every wall-clock figure in this entry and is measured by the probes instead.

**One new root, which is not bookkeeping.** `loop_advance` did not root the value it binds. Between `Interp::number` creating it and `bind_control` writing it, a trace render allocates and a compound control resolves a tail key -- so the frame alone would have been a use-after-free rather than a fix. That push is load-bearing and the mutation section below is the evidence.

#### The prediction, and what it was

The brief supplied the prototype's figures as a prior to check rather than a result to reproduce: peak RSS falling **42x on `arith`, 60x on `varlookup`, 298x on `strings`**, and `strings` **16% faster**. **The prior held on memory and did not on time**, and it is worth saying exactly how: the RSS multiples came in at 41.1x, 59.4x and 220.1x -- the third lower because entry 4 had already taken `strings`' base peak from 3.7 GB to 2.5 GB -- while `strings`' wall win is -5.3% rather than -16%, for the same reason.

**The prior contained no prediction at all for `arith`'s or `alloc4c`'s wall time, and that is exactly where the surprise landed.** `phase-4d-retention.md` reported both as inside its own noise at n=3. They are not: `arith` is +6.6% and `alloc4c` +2.7%, both reproducible. Nothing was predicted, so nothing was missed -- and a candidate whose prior is silent on two of six axes is a candidate whose prior was thin, which is the thing to carry forward rather than the number.

#### Build identity

| | |
|---|---|
| BASE `rexx-run` | size=13762264, sha256 `ea3b80c84e11c5a85ba8b8efede9872f0226b57e8f69d479957202d49bfc3922` -- **entry 5's HEAD binary reproduced byte for byte** at `bebf38ee` |
| HEAD `rexx-run` | size=13925200, sha256 `ae24900aa72300de1b1d3cf1a274a5c74aed741a58b7708e449d177c9723018d`; the binary **grew** by 162,936 bytes |
| the third binary | `COLLECT_FLOOR` set to `usize::MAX` and nothing else changed, so cause B is in and cause A is out: sha256 `9afe461af4369a5a206c2caac61e01fa02eabac5efda4fab3d59c6cdff2fc8da` |
| oracle | the same three objects entry 1 fingerprints, re-hashed and unchanged: `bb5bb8cc...`, `42136c40...`, `3536b763...` |

**The measured HEAD binary is the committed source's release build**, rebuilt from a `cargo clean`ed tree after the clippy run and hashed again to `ae24900a`.

#### The accept measurement: the two binaries alternating in one loop

The accept rule's literal shape, and **not** `rexx-bench-suite`. Wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, one fresh empty working directory, base/head order **rotated every round**. 2026-08-11 17:48:42 to 17:52:56 +02:00, seven rounds per axis, load average 2.56 rising to 3.58, all 84 runs exiting 0, and each axis's stdout hashed per run and identical across all fourteen.

| axis | base median | head median | head / base | rounds with that sign |
|---|---:|---:|---:|---:|
| `strings` | 6.0931 s | 5.7692 s | **-5.32%** | **7 of 7** |
| `emptyloop` | 2.0033 s | 1.9414 s | **-3.09%** | **7 of 7** |
| `varlookup` | 2.6902 s | 2.6295 s | **-2.25%** | **7 of 7** |
| `compound` | 2.8218 s | 2.7849 s | -1.31% | 6 of 7 |
| `alloc4c` | 1.5849 s | 1.6276 s | **+2.69%** | 5 of 7 |
| `arith` | 3.0023 s | 3.2015 s | **+6.63%** | 7 of 7 |

Per-round on `arith`: +5.6%, +5.3%, +6.7%, +7.3%, +6.8%, +5.7%, +8.4%. There is no round in which it is not slower, and it is the largest single move on this table.

`rexxcps` is not read as wall time, for entry 1's reason. Both binaries self-calibrated to the identical `100 x 100`: five rounds, order rotated, base median **2,393,798** clauses per second against head **2,526,039** -- **+5.52%**, head ahead in all five.

#### Peak resident set, which is what this candidate is actually for

`/usr/bin/time -f %M`, same wrapper, one run per side.

| axis | base | head | fall |
|---|---:|---:|---:|
| `strings` | 2,511,564 KB | 11,408 KB | **220.1x** |
| `rexxcps` | 1,744,052 KB | 11,488 KB | **151.8x** |
| `emptyloop` | 197,496 KB | 2,776 KB | **71.1x** |
| `varlookup` | 149,648 KB | 2,516 KB | **59.4x** |
| `arith` | 440,184 KB | 10,704 KB | **41.1x** |
| `compound` | 41,072 KB | 2,824 KB | **14.5x** |
| `alloc4c` | 392,972 KB | 180,396 KB | 2.1x |
| `startup` | 2,788 KB | 3,008 KB | 0.9x |

`alloc4c` is the one axis whose live set is genuinely large, so 2.1x is what a collector can reach there and not a shortfall. `startup` goes the other way by 220 KB, which is the binary's own growth.

**The user-visible symptom is gone**, and that is the defect rather than the number. Under the project's standard `ulimit -v 1048576`, at BASE `arith.rex` and `strings.rex` both die at rc 134 with an empty stdout and `memory allocation of 402653184 bytes failed` on stderr; at HEAD both exit 0 with `4629643519330627.7808` and `138000000`.

#### Where the time went, which the wall clock alone gets backwards

`perf stat -e instructions:u,cycles:u`, three runs per side per axis, arms alternating, medians. A different configuration from the sitting above.

| axis | instructions base | instructions head | change | cycles change | wall change |
|---|---:|---:|---:|---:|---:|
| `varlookup` | 43,103,417,693 | 43,523,113,088 | +0.97% | -0.59% | -2.25% |
| `emptyloop` | 28,127,015,102 | 28,635,681,040 | +1.81% | +1.24% | -3.09% |
| `compound` | 37,026,601,429 | 37,169,537,787 | +0.39% | -0.92% | -1.31% |
| `strings` | 62,941,012,332 | 63,197,495,286 | +0.41% | **+6.54%** | **-5.32%** |
| `alloc4c` | 10,718,049,587 | 11,258,970,022 | +5.05% | +6.96% | +2.69% |
| `arith` | 28,881,521,649 | 29,956,994,783 | +3.72% | +11.68% | +6.63% |

**`strings` retires 6.5% more user cycles and finishes 5.3% sooner, and both are true.** `cycles:u` does not count the kernel. Minor page faults and the two time components, one run per side: `strings` 627,872 faults and 0.84 s system at base against **2,427 faults and 0.00 s** at head, with user time going the other way, 5.26 s to 5.43 s. `varlookup` 37,261 -> 136 faults, `arith` 109,760 -> 2,208, `alloc4c` 114,356 -> 55,802. **The win is kernel time on a heap that never stopped growing; the cost is user time.** Entry 2 was reading the same thing from the other side when it found `mprotect`, `munmap` and `sysmalloc` in `strings`' own profile.

#### Which cause did what, measured rather than apportioned

The third binary above has cause B in and cause A out. Instructions, three runs per side, medians.

| axis | base | cause B only | both | cause B alone | cause A adds |
|---|---:|---:|---:|---:|---:|
| `arith` | 28,881,090,013 | 28,894,985,743 | 29,903,747,781 | +0.05% | **+3.49%** |
| `alloc4c` | 10,712,944,869 | 10,528,680,671 | 11,247,204,773 | -1.72% | **+6.82%** |
| `emptyloop` | 28,125,666,059 | 28,525,608,865 | 28,625,608,689 | **+1.42%** | +0.35% |
| `varlookup` | 43,092,659,884 | 43,396,613,244 | 43,548,613,556 | +0.71% | +0.35% |
| `compound` | 37,019,025,702 | 37,099,899,846 | 37,189,905,316 | +0.22% | +0.24% |
| `strings` | 62,924,005,067 | 63,031,788,325 | 63,138,170,564 | +0.17% | +0.17% |

**The whole of the instruction cost on `arith` and `alloc4c` is the trigger, and the frames are free everywhere except `emptyloop`**, where two `Vec` operations per pass on a loop that does nothing else read +1.42%. `alloc4c`'s -1.72% under cause B alone is not a mechanism -- nothing in that change can remove work from an axis with no per-pass root to release -- and it is left unexplained rather than claimed.

#### Why reclamation costs what it costs, profiled rather than reasoned

`samply` 0.13.1 `--save-only`, 1 kHz, `REXX_ENGINE=ir`, fresh empty directory, analysed through `pollard` with `expand_inlines`; `unsymbolicated_pct` 0.0% at head and 0.012% at base. **The two profiles were taken separately and are not interleaved, so only the shares are read and the durations are not.**

On `arith` at head, `Interp::alloc_with`'s whole subtree is **5.5%** of the run. `Heap::collect` is **4.8% total and 0.3% self**, and `core::ptr::drop_glue::<rexx_core::heap::Slot>` under it is **4.5%**. The glibc free family is 15.3% self at head against 9.0% at base.

**So the cost is not marking and it is not sweeping: it is the `free()` of each reclaimed object's payload, which a leak never pays.** A heap that never collects never calls `free` on a heap object at all. `arith` is the axis with the highest allocation-to-work ratio and a tiny live set, so it pays the whole of that and gets only the page-fault saving back; `alloc4c` pays it on top of marking a live set that is genuinely large, which is what entry 2 predicted when it said a collector "will move it the wrong way".

#### Correctness, which outranks the number by more than usual here

**This is a garbage collector, so the instrument matters more than the result.** `run_program_collect_every_alloc` sets `stress_collect`, which collects on *every* allocation -- strictly more often than any watermark -- so a program that survives it survives the trigger. A swept slot's generation is incremented, so a stale handle **misses** rather than aliasing, and the failure is a loud `a live value` or a wrong answer rather than a silent read of another object.

* **The whole workspace, with collect-on-every-allocation forced on for every run.** Every harness that runs a Rexx program is green: the corpus differential against the oracle, the dual-engine sweep, the trace oracle, the `ootest` assertion, `bif` and `keyword` harnesses, `collect_stress` itself. **79 tests fail, and all 79 are `rexx-exec`'s own lib unit tests** -- `let true_value = interp.text(b"1"); let false_value = interp.text(b"0");` holds the first handle in a Rust local across the second allocation. That is test code, and no unit test reaches 65,536 live objects, so the production trigger cannot fire there.
* **The dual-engine populations under the stress mode on both arms**, by patching `ir_dual.rs`'s own comparison to run each case a third and fourth time: corpus 121 cases, `bif` 5,185, `expressions` 4,259, `keyword` 896 -- **10,461 cases, 20,922 stress runs, 182,106 collections**, every one byte-identical to its own plain run on stdout, stderr and exit status. 1,954 of those runs collected nothing, which is the documented shape of a program whose every value is an inlined small integer. The file was restored from a copy afterwards and `sha256sum -c`'d, never `git checkout --`.
* **The differential against the oracle is unchanged**: the corpus runner reports 10 passed and the dual-engine sweep 9 in the release run, read as counts.

**One real unrooted window was found, and closing it is part of this change.** `Heap::collect`'s own doc named it and left it to whoever wired in a collector: `EXIT`'s result is rooted by a one-clause temp that `leave_stepped_clause` pops before `Flow::Exit` reaches `run_activation`, and nothing names it from there to `exit_code_for`. A `Heap::collect` forced immediately before `exit_code_for` **panicked on `a live value` in four harnesses**. `Interp::apply_flow` now takes a root that outlives the temps stack (`Interp::root_exit_value`, `add_global`) at the one place an activation's value stops being a clause's temporary -- both the `Flow::Exit` and the `Flow::Return` arm, so `EXIT`, a top-level `RETURN`, a `RAISE` with an `EXIT` tail and a handler's own exit are all covered without enumerating them. With it, the same forced-collect probe leaves the whole workspace green. **Both arms are needed, measured rather than assumed**: rooting `Flow::Exit` alone leaves three harnesses panicking under that probe, because a top-level `RETURN` reaches `exit_code_for` too. An earlier attempt that rooted at the *producers* instead -- the `EXIT` instruction and the `RAISE` tail -- left them red as well, and that is what moved the root to the one consumer.

**The trigger itself never fires inside that window**, which is why the stress mode had never found it: nothing on the path from the pop to `exit_code_for` allocates. That is a property of today's code rather than an invariant, which is the reason it was closed rather than written down as benign.

**The committed witness is `a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`** (`collect_stress.rs`), five loop shapes on both engines under collect-on-every-allocation, every expected string measured on the oracle. Its fifth row -- a compound control stepped by `BY 0.5`, so the bound value is a heap object and the write resolves a tail key -- is the one that fails; the first four are the adjacent successes that pin the failure to that shape rather than to loops. A fourth column says per row whether the row allocates at all, in both directions, because `do i = 1 to 3` allocates nothing and asserting that it collects would assert something false.

**It adds coverage, which is a separate claim from being able to fail, and both mutations were measured.** Moving the `pop_frame` to before `bind_control`, and separately deleting the `push_temp` of the bound value, each redden this witness -- and **the entire workspace suite without it stays green under both**, the 10,461-case sweep included with the stress mode on. `run.rs` was restored from a copy and `sha256sum -c`'d after each.

**And one of the three sites cannot be pinned by any test, which is stated rather than implied.** Popping `eval_condition`'s frame *before* its `to_text` leaves the entire workspace green, including the new witness. Nothing allocates between the push and the last use of the value, so the root there is unobservable -- the same shape `collect_stress.rs` already records for `eval_arithmetic`'s `right_value`. The frame around it is a memory-lifetime device, and its correctness rests on the value never being handed back rather than on a measurement.

Gates: `cargo test --workspace` **1438 passed, 0 failed, 4 ignored** in dev and in release, run counts read rather than exit status alone. **BASE was re-counted rather than inherited, and the brief's figure was five behind**: `cargo test --workspace -- --list` in a `git worktree` at `bebf38ee` enumerates 1441, of which 4 are `#[ignore]`, so BASE runs **1437** -- entry 5's own HEAD figure. The brief said 1432, which was BASE for entry 4 and is the same stale figure entry 5's brief carried. HEAD runs exactly one more, the witness above. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean from a full `cargo clean`, with `Checking rexx-core` and `Checking rexx-exec` confirmed in the log.

#### Disposition: **accepted**

The candidate's stated falsification was peak RSS not falling by at least an order of magnitude on `strings`, or any differential divergence, or the stress harness finding an unrooted value. `strings` fell **220x**; nothing diverged; the harness did find one unrooted value, which was the `EXIT` window the tree already had on record, and it is closed in this change rather than carried.

**Accepted with the cost recorded and not netted out.** Four axes get faster and two get slower, and the two that get slower are bar-bound: `arith` +6.6% and `alloc4c` +2.7%. The mechanism is named and measured -- reclamation pays a `free()` per object -- so this is a known trade rather than an unexplained regression. It is taken because the alternative is a program whose live set is two integers aborting at rc 134 with no Rexx condition, no traceback and its output lost, which is a defect and not a performance property.

**The hypothesis is confirmed by the route it named** on memory, where the instruction and page-fault columns both agree. It is **not** confirmed on `strings`' wall time: the prototype's -16% came from an axis peaking at 3.7 GB, entry 4 already took that to 2.5 GB, and what is left is -5.3%.

Commit: `967adba3e2dc638502c13ab2742122569a02fc47`, read back from `git log` after committing.

#### What this entry cannot say

* **It is one sitting on one Linux host**, unpinned, on a machine whose governor cannot be fixed.
* **It did not run `rexx-bench-suite`, so it carries no oracle ratios.** Every figure is against the immediately preceding binary. Where `arith` and `alloc4c` now sit against the oracle is unmeasured, and both moved the wrong way.
* **The trigger policy was measured once, at one setting, and nothing here says it is the right one.** `COLLECT_FLOOR` at 65,536 and a doubling watermark are an argument about amortisation, not a measured optimum, and the entry deliberately does not contain a sweep over thresholds.
* **The `eval_condition` half of cause B is measured by probes and by no benchmark.** No axis in this suite is a `WHILE`, an `UNTIL` or a `FOREVER`, which is the blind spot `phase-4-exclusions.txt` warned about; what says that half works is the 8,000,000-iteration probe going from 64,680 KB to 2,768 KB, not any axis above.
* **Nothing here measures footprint in a test.** The witness added checks rooting, not memory, so a reintroduced leak that kept its roots correct would pass every gate in the workspace. That half of the coverage gap is still open and the exclusions row says so.
* **`alloc4c`'s -1.72% under cause B alone is unexplained** and no mechanism offered covers it.
* **The tree-walker was measured only through the stress populations.** Every wall, instruction and RSS figure above is the IR arm.

### Entry 7 -- the trigger's **signal** changed, on the oracle's shape: the arena about to grow, not the live count

**BASE is `967adba3`**, entry 6's own commit. This entry changes one condition and nothing else, and it is recorded separately because entry 6 is not wrong -- it measured what it measured -- and the record is appended to rather than rewritten.

**It does not pass the accept rule and is not claimed to.** No bar-bound axis moves in work terms. It lands as a policy correction with a measurement behind it on a shape no axis has, and the section below says exactly what that measurement is and what it is not.

#### Where the hypothesis came from, and the half of it that does not port

Moritz read the oracle's own trigger. `NormalSegmentSet::handleAllocationFailure` (`interpreter/memory/MemorySegment.cpp:1135`) does three things in order: `collect()`, then `adjustMemorySize()` -- "now that we have good GC data, decide if we need to adjust the heap size" -- then retry the allocation. So the reference implementation allocates until it cannot, collects, and **sizes the heap from what the collection just learned**.

**The first half is not available here and the correction came before anything was built.** The oracle can trigger on failure because it owns its segments. This crate takes each object's **payload** from `malloc`, and `malloc` does not fail -- it grows the process until the OOM killer arrives. There is no failure event to hang a trigger on.

**The analogue is a branch that already existed.** `Heap::alloc_with_uncollected` either reuses a slot from `free_head` or pushes onto `slots`. The push arm is the moment the arena would grow, which is what allocation failure means to the oracle, and it costs nothing to detect. `Heap::will_grow` exposes it. Entry 6's doubling watermark is already `adjustMemorySize`'s half, so only the signal changed:

```
before   live_count()  >= collect_at
after    will_grow()  &&  slot_capacity() >= collect_at
```

**Two facts about the heap this entry has to be accurate about, because the policy's limits follow from them.** Slots are *not* per-object `malloc`: `slots` is one growing `Vec<Slot>` with a free list, arena-allocated and reused after a collection. What goes through `malloc` individually is each object's payload inside `Body`. And **the trigger counts slots, not bytes** -- a few very large strings are few slots and a great deal of memory, and neither half of this policy reacts to them. That is written into `Interp::collect_at`'s own doc as a named limitation rather than left to be discovered; closing it needs payload sizes `Body` does not carry.

#### Predicted movement, written down before the first measurement

| where | predicted | the reasoning behind it |
|---|---|---|
| every benchmark axis | **no change at all**, identical collection counts | once the free list is exhausted, capacity and live count are the same number, so the two conditions fire at the same allocation |
| a large live set that dies, then churn | **fewer collections, same peak RSS** | the free list holds the dead transient's slots; the live count has fallen back to the floor, so the old condition fires while thousands of swept slots sit unused |
| peak resident set anywhere | **no change** | the policy decides *when* to reclaim, not *how much* |

**All three held.** This is the first entry in this record whose predictions were not beaten or missed, and the reason is that it predicted no movement -- which is a weaker thing to get right.

#### The measurement

**On every axis the two conditions fire at the same allocations.** Collection counts, read from `Outcome::collections` through a pair of builds differing from the two below only by an `eprintln` behind an environment variable:

| axis | live-count trigger | growth trigger | peak RSS, KB |
|---|---:|---:|---|
| `strings` | 274 | 274 | 11,480 / 11,444 |
| `arith` | 52 | 52 | 10,732 / 10,596 |
| `alloc4c` | 30 | 30 | 180,528 / 180,108 |
| `compound` | **0** | **0** | 2,752 / 2,404 |
| `varlookup` | **0** | **0** | 2,496 / 2,756 |
| `emptyloop` | **0** | **0** | 2,496 / 2,440 |

**On the shape the two differ on, the difference is a factor of two to three.** A program building a 200,000-tail stem, dropping it, then making a long tail of short-lived values -- every run printing the same two lines:

| program | live-count trigger | growth trigger | peak RSS |
|---|---:|---:|---|
| 200,000 tails, dropped, then 1,000,000 churn | 19 collections | **8** | -- |
| 200,000 tails, dropped, then 2,000,000 churn | 35 collections | **11** | 60,860 KB / 60,656 KB |

Same footprint, same output, a third of the collector invocations.

#### This entry corrects entry 6's attribution, which did not separate the two causes on memory

**Three of the six axes collect zero times**, so `compound`'s 14.5x, `varlookup`'s 59.4x and `emptyloop`'s 71.1x falls in peak resident set are **entirely cause B** -- the loop's per-pass frames -- and owe nothing to the trigger. Entry 6 reported those multiples for the change as a whole and did not say which cause produced them. `strings`, `arith` and `alloc4c` are the axes cause A moves, and entry 6's split of the *instruction* cost already said so from the other side: the trigger's instruction cost is +3.49% and +6.82% on `arith` and `alloc4c` and about +0.2% on the other four.

#### The wall clock moves and it is layout, disclaimed rather than banked

Paired interleaved, base/head rotated every round, seven rounds per axis, 2026-08-11 18:27:04 to 18:31:04 +02:00, load average 0.31 rising to 0.98 -- the quietest sitting in this record -- all 84 runs exiting 0 with a stdout hash identical across all fourteen runs of each axis.

| axis | e6 median | e7 median | change | rounds with that sign | instructions | cycles |
|---|---:|---:|---:|---:|---:|---:|
| `arith` | 3.0427 s | 2.9812 s | -2.02% | 7 of 7 | **+0.14%** | -1.81% |
| `alloc4c` | 1.4435 s | 1.4726 s | +2.02% | 6 of 7 | **+0.15%** | +1.71% |
| `emptyloop` | 1.9268 s | 1.9464 s | +1.02% | 6 of 7 | +0.06% | +1.35% |
| `varlookup` | 2.5677 s | 2.5491 s | -0.72% | 7 of 7 | +0.03% | -1.21% |
| `strings` | 5.4137 s | 5.4406 s | +0.50% | 4 of 7 | -0.02% | -3.20% |
| `compound` | 2.7482 s | 2.7439 s | -0.15% | 6 of 7 | -0.07% | -0.25% |

**No axis executes measurably different work**: every instruction column is inside 0.15%, which is what has to be true when the collection counts are identical. The wall and cycle columns move by up to 2% in both directions, and that is the layout channel entry 4 named -- a cycle move with no instruction move behind it. **`arith` at -2.02% in seven of seven rounds is not this candidate giving back a fifth of entry 6's regression**, and nothing here should be read that way.

Peak resident set is unchanged on every axis: `arith` 10,120 / 10,720 KB, `compound` 2,424 / 2,784, `strings` 11,444 / 11,160, `varlookup` 2,540 / 2,472, `alloc4c` 180,448 / 180,284, `emptyloop` 2,456 / 2,436.

#### Correctness

**The whole workspace with collect-on-every-allocation forced on for every run**, re-run on this source because the trigger changed: every harness that runs a Rexx program is green, and the same 79 `rexx-exec` lib unit tests fail for the same reason entry 6 records -- test code holding an `ObjRef` in a Rust local across an allocation, which no watermark can reach.

**The committed witness is `tests/collect_policy.rs`**, and it is the first thing in this tree that reads `Outcome::collections` on an *ordinary* run. `collect_stress.rs` cannot: that mode overrides the trigger, and every corpus program is far below the growth allowance, so under `run_program` they all collect zero times and pass whatever the policy is. Two tests, and the second is the adjacent case: with the `DROP` removed the live set stays live, there are no free slots to wait for, and the count must **not** fall -- which is what stops the first test being satisfied by a collector that simply fires less.

**Both were mutation-checked, in both directions.** Reverting the condition to the live count reddens the first test and **the entire workspace without that file stays green**. Setting the growth allowance to `usize::MAX`, so nothing collects at all, reddens both.

Gates: `cargo test --workspace` **1440 passed, 0 failed, 4 ignored** in dev and release -- BASE's 1438 plus the two above. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean from a full `cargo clean`, with `Checking rexx-core` and `Checking rexx-exec` confirmed in the log. `rexx-run` sha256 `8ef5c32e99503841b11aece93eb32200127366cf8b0742c12bc467212e1fe82d`, size 13832312, reproduced from the committed source after that clean.

#### Whether the IR should emit a GC op: **no**, and the reason is not "not yet"

Raised alongside the trigger question, and it is a design question rather than a measurement, so it is answered here rather than tried.

**What a VM emits is not "collect now" but a safepoint poll, and polls exist to stop *other threads*.** HotSpot puts them at loop back-edges and returns so that a thread requesting a collection can bring the others to a known state. This crate is single-threaded until Phase 6: there is nothing to stop, so a poll buys nothing and costs an op in `run_ops`' own loop -- the one function every axis runs, and the one entry 3 measured a 399-million-instruction displacement in.

**The question an op could usefully answer is a different one -- where is the root set precise -- and the answer is that the precise point already exists and is not an op.** The clause boundary truncates the temps stack to a known watermark, once per clause, through an enter/leave pair that is already there. **This entry's own numbers are what say that matters**: three of six axes reclaim their whole footprint from cause B, which is precision and not frequency, and they collect zero times.

**It is still not worth making the trigger.** Collecting only at clause boundaries would leave a single clause that allocates heavily -- a long concatenation chain, an `INTERPRET` fragment -- unable to collect at all while it ran, and the allocation site is where the pressure is actually known. Precision and trigger are different axes and this crate now has one of each, at no cost in instructions.

#### Disposition: **accepted, and explicitly not as a performance candidate**

The accept rule asks for a paired interleaved comparison showing a reproducible gain. **This shows none**: no axis moves in work terms, and the wall movements are layout in both directions. Judged as an optimisation it would be discarded.

It lands because the question it answers is not "is this faster" but "what should the collector trigger on", and there the evidence is one-sided: the two signals are indistinguishable on everything this suite measures, and on the one shape that separates them the growth signal does a third of the work for the same footprint. **It is also the shape the reference implementation uses**, minus the half that does not port, which is a better warrant for a policy than a constant chosen here.

Commit: `3d14fdf46fe25e8e93392490a23594b7298996f5`, read back from `git log` after committing.

#### What this entry cannot say

* **The collection counts were read from instrumented builds**, not from the two binaries the wall figures come from. They differ from those only by an `eprintln` behind an environment variable and by `Heap::will_grow` being `pub`, but they are not the same binaries and the counts are not a property of the ones that were timed.
* **The transient shape is one program, not a family.** How the two signals compare across live-set sizes, churn lengths and object widths is unmeasured; three points were taken and all three favour the growth signal.
* **Nothing here reacts to payload bytes**, and the limitation is named in the code rather than closed. A program holding a few very large strings collects on neither policy.
* **It re-measures no oracle ratio**, like every entry since entry 2.
* **The `-2.02%` on `arith` and `+2.02%` on `alloc4c` are not results.** Their instruction columns are flat and this entry claims neither.

---

### Entry 8 -- a candidate added to the queue, not attempted: replace the global allocator

**Added 2026-08-11 by Moritz.** This is a queue entry rather than an attempt; nothing was built or measured for it.

**Why it belongs on the queue now rather than at entry 2.** Entry 2 ranked candidate 1 as "stop calling the allocator once per value", and its two levers both reduce the *number* of allocations. This is the third lever nobody listed: make each one cheaper. It moved from uninteresting to interesting because of what entry 6 landed -- **a heap that never collected never called `free` on a heap object at all**, and reclamation now pays a `free()` per reclaimed payload, which entry 6 measured as the whole of its cost on `arith` (+6.6%) and part of it on `alloc4c` (+2.7%).

**The ceiling.** Entry 2 measured the glibc allocator family at **36.1% self on `strings`**, and entry 4 reproduced the shape after removing a third of that axis's allocations. Every axis except `emptyloop` shows the family in its top functions. **A replacement allocator does not remove an allocation; it changes what each one costs**, so the ceiling is a fraction of that 36.1% rather than the whole of it, and no honest number is available before it is built.

**What is already available.** No `#[global_allocator]` is set anywhere in the workspace, so every allocation goes to glibc. `mimalloc` 0.1.52 and `tikv-jemallocator` 0.6.1 and 0.7.0 are in the offline cargo cache, so this needs no network.

**The decision this candidate carries, and it is not a measurement.** `libmimalloc-sys` and `tikv-jemalloc-sys` both **compile and link a C library**. This project is a clean-room *Rust* reimplementation whose oracle is a C++ interpreter, and vendoring a C allocator to beat that interpreter is a choice about what the project is, not just about what is fast. It is also a new build dependency on five platforms, where `:35` requires every phase gate to run. **Whoever attempts this must have that decision made rather than assume it**, and a Rust-native allocator or an arena for payloads is the alternative that avoids the question entirely.

**Why it is not ranked above candidates 4 to 7.** Those have measured ceilings on named mechanisms. This one's ceiling is a fraction of a share, its risk is a C dependency, and its win is orthogonal to every other candidate -- it will still be there after them, and it will be easier to judge once the allocation *count* work is done.

**What would falsify it.** A paired run moving `strings` and `arith` by less than about 5%, which would say the allocator was never the cost that mattered at this size.

**Its relationship to the payload-bytes blind spot.** Entry 7 recorded that neither trigger policy reacts to payload bytes, so a program holding a few very large strings collects on neither. A replacement allocator does not close that either; the two are independent and both remain open.

### Entry 8 -- entry 6's wall figures re-taken on a quiet host, and two of them were wrong

**No code change. Nothing here is a candidate.** This entry re-measures entry 6 and corrects it, which is what this record's own rule asks for: an entry that turned out wrong is corrected by a later entry saying so, not by editing the one that was wrong.

**Why it was re-taken.** Moritz reported that another agent on this host had started using a great deal of CPU. Entry 6's sitting ran at one-minute load average 2.56 rising to 3.58; entry 7's ran at 0.31 to 0.98. Neither number is alarming on 32 CPUs, and load average is exactly the instrument entry 3 already recorded as inadequate here -- it saw the host at 96.5% busy while `ps` inside this sandbox showed nothing running at all.

#### What was done differently

**Host idle read directly from `/proc/stat` rather than inferred from load average**, and a sitting started only after **six consecutive five-second samples at or above 90% idle**. That gate is entry 3's discipline made mechanical.

**It was needed, and it caught the interferer live.** The first attempt at the fourth sitting below started at **0.1% idle** -- the host went from 99.1% to 0.1% across two consecutive samples -- and its `varlookup` arm contains a single round reading **+39.0%** against a median of -3.8%. That sitting is kept in the table as the instrument's own worst case and is not read for anything else.

#### Five sittings, same two binaries per column, seven rounds per axis, order rotated every round

Sittings 1 to 3 are BASE `ea3b80c8` against entry 6's head `ae24900a`. Sittings 4 and 5 are BASE against entry 7's head `8ef5c32e`, which entry 7 established executes the same work to within 0.15% on every axis, so the two columns are comparable and the split is only which head was to hand.

| axis | s1 busy | **s2 quiet** | **s3 quiet** | s4 *0.1% idle* | **s5 quiet** | quiet median | quiet range |
|---|---:|---:|---:|---:|---:|---:|---:|
| `strings` | -5.32% | **-10.83%** | **-10.61%** | -13.13% | **-11.34%** | **-10.83%** | 0.73pt |
| `emptyloop` | -3.09% | -3.71% | -3.35% | -1.68% | -2.01% | **-3.35%** | 1.70pt |
| `varlookup` | -2.25% | -2.56% | -2.85% | -3.83% | -3.24% | **-2.85%** | 0.69pt |
| `compound` | -1.31% | -0.64% | -0.59% | -1.07% | -0.71% | **-0.64%** | 0.13pt |
| `alloc4c` | **+2.69%** | -3.81% | -1.12% | -2.68% | -1.56% | **-1.56%** | 2.69pt |
| `arith` | +6.63% | +3.39% | +5.47% | +0.98% | +2.06% | **+3.39%** | 3.41pt |

Every run of all five sittings exited 0, and each axis's stdout hash is identical across all seventy runs of it.

#### The three corrections to entry 6

**1. `strings` is -10.8%, not -5.3%. Entry 6 understated its own headline by half.** The three quiet sittings read -10.83, -10.61 and -11.34, a range of 0.73 of a point, against the busy sitting's -5.32. That also moves it much closer to `phase-4d-retention.md`'s prototype prediction of -16%, and entry 6's explanation for the shortfall -- that entry 4 had already taken `strings`' base peak from 3.7 GB to 2.5 GB -- now accounts for the whole of the remaining gap rather than half of it.

**2. `alloc4c` is not a regression, and it is not a result either.** Entry 6 reported +2.69% and read it as the collector costing most where the live set is genuinely large, which is what entry 2 predicted. Across five sittings it reads +2.69, -3.81, -1.12, -2.68 and -1.56, and the sign is unstable **within** sittings too -- 5 of 7 rounds in the two that are closest to zero. **The honest verdict is that this axis produced no usable wall figure**, and entry 6's sentence about it should be read as withdrawn rather than adjusted. Entry 2's prediction is neither confirmed nor refuted here.

**3. `arith` is +3.4%, not +6.6%, and it is still the one real regression.** Its sign held in every one of the 35 rounds across all five sittings. The magnitude is the part that moves.

#### Why those two axes are the unstable ones, from an instrument load cannot touch

Entry 6's time-component measurement -- `/usr/bin/time`'s `%S`, `%U` and minor-fault count -- says where the win and the cost live, and both quantities are load-independent in a way a wall percentage is not. The win is kernel time saved on a heap that stopped growing; the cost is user time spent collecting and freeing. **Predict the net as (user added - system saved) over the base wall, and it lands on the quiet medians:**

| axis | system saved | user added | predicted | quiet median |
|---|---:|---:|---:|---:|
| `strings` | 0.84 s | 0.17 s | **-10.90%** | -10.83% |
| `arith` | 0.12 s | 0.26 s | **+4.75%** | +3.39% |
| `alloc4c` | 0.13 s | 0.10 s | **-2.00%** | -1.56% |
| `varlookup` | 0.02 s | -0.04 s | **-2.27%** | -2.85% |

**`strings` is predicted to within 0.07 of a point.** `alloc4c` is the difference of two nearly equal quantities, 0.13 s against 0.10 s, which is exactly the shape whose sign a busy host can flip -- and did. `arith` is cost-dominated by two to one, which is why its sign never moved. **So this is not a choice between two sittings on the ground that one is quieter; the load-independent instrument independently predicts the quiet one.**

#### What does not change

* **Every peak resident set figure.** Resident set does not depend on CPU contention: `strings` 220x, `rexxcps` 152x, `emptyloop` 71x, `varlookup` 59x, `arith` 41x, `compound` 14.5x, `alloc4c` 2.1x all stand, as does `arith` and `strings` going from rc 134 to rc 0 under the standard address-space cap.
* **Every instruction count**, in entry 6, in entry 7 and in the cause A/cause B split. Entry 5 measured this instrument reproducing to eight figures.
* **Every correctness result.** No differential, no stress run and no gate is a timing measurement.
* **Entry 7 entirely.** Its sitting was already on a quiet host (0.31 to 0.98), its conclusion was that the wall movements were layout, and its instruction columns -- flat to within 0.15% -- are what carried that. Nothing here disturbs it. Its `arith` -2.02% and `alloc4c` +2.02% remain disclaimed, and the sittings above are consistent with that: BASE to entry 7's head reads +2.06% on `arith` where BASE to entry 6's head reads +3.39%, a difference inside the spread of either.
* **The disposition.** Entry 6 was accepted as a defect fix with its cost recorded. The cost is smaller than it recorded -- one axis at +3.4% rather than two axes at +6.6% and +2.7% -- so the trade is better, not worse.

#### What this entry cannot say

* **It does not re-run `rexxcps`**, whose clauses-per-second figure entry 6 took in the busy sitting. +5.52% at 5 of 5 stands unretested, and it is a throughput figure on a self-calibrating program rather than a wall ratio, so the mechanism above does not obviously apply to it.
* **Three quiet sittings is not a distribution.** The quiet ranges above -- 0.13 to 3.41 points -- are a spread over three points, and the two widest are the two axes whose net is a small difference of larger numbers.
* **It cannot say what the interferer was.** `ps` inside this sandbox showed nothing but this session's own agent at any point, exactly as entry 3 recorded, so the host's idle percentage is the only evidence of it.
* **The quiet-run-up gate is not in any committed harness.** It was a shell loop in the scratchpad, and `rexx-bench-suite` has no such check. A later sitting that skips it can reproduce sitting 1 without noticing.
