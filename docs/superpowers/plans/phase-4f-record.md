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

### Entry 9 -- candidate 4 attempted and **accepted**: the tag decision read off the number's shape, not off a rendering of it

**BASE is `c761c5f4`**, whose `rexx-run` reproduces entry 7's HEAD binary byte for byte from a clean `git status` -- which is what says the two intervening commits are the record and nothing else. The record-only commit `db0d2942`, carrying the re-measurement below, landed on the branch while this candidate was being built; it changes nothing under `rust/`, so the binary it names is the one measured here.

**Two entries carry the number 8** -- the allocator queue addition and entry 6's re-measurement -- and this is the ninth by position. Where this entry cites one of them it says which.

#### The hypothesis, named before it was measured

`small_int_for` (`value.rs`) decides whether an arithmetic result becomes an inline `SmallInt` or a heap `Body::Num`. It answered cheaply when `Number::plain_integer` accepted, and otherwise called `format_form` -- a full decimal render into a fresh `String` -- scanned the result for a `.` and an `E`, parsed it back to an `i64`, and threw the string away. **Every arithmetic result on `arith` reached that line**, because `plain_integer` answers only for a value *already* written as an integer and `arith`'s results are decimals.

The change is `Number::rendered_integer`, an exact predicate over the number's own exponent and digit vector, and the one call site that now goes through it. The rendering asks three questions -- what rounding to `DIGITS` does to the value's shape, whether the exponential trigger fires, and whether a decimal point is written -- and every one of them is a property of that shape rather than of the string. So the predicate is `round_to`'s branch decision, then `format_with`'s trigger, then the sign of the exponent, and it allocates nothing: the rounded copy and the rendered `String` both go.

**The obvious shortcut is not this change, and the record already measured why.** Prototype P3 returned `None` whenever `plain_integer` declined: -16.3% on `arith`, but `compound` +1.6% and `strings` +2.7% *slower*, because the probe genuinely accepts values `plain_integer` declines -- every result whose *rounding* is the integer, `12.4` at `DIGITS 2` being the shape -- and dropping them turns those into heap objects. **Accepting the same set is the whole of the work**, and what holds it is an exhaustive equivalence check rather than an argument.

#### Predicted movement, written down before the first timed run

| axis | predicted | the reasoning behind it |
|---|---|---|
| `arith` | **-6% to -14%**, central -10% | entry 2's ceiling is 14.8% of the axis; the replacement is not free, since it does the same rounding and trigger arithmetic the render's first two steps do, so under the ceiling |
| `compound`, `strings`, `varlookup`, `emptyloop`, `alloc4c`, `rexxcps` | **0%** in work terms, band -1% to +1% on instructions | since entries 3 and 4 their hot clauses stay on `arith_small_int`'s tagged path, which answers through `exact_small_int` and never builds a `Number`, so this predicate is not reached |
| peak resident set | **no change anywhere** | the change alters neither which values are heap objects nor their size |

**Two of the three were wrong, and the second is the avoidable one.**

**`arith` came in at -16.9% against a -14% band edge.** The ceiling counted the render; the change also removes the `str::parse` after it and `round_to`'s clone of the digit vector -- and since entry 6 a clone is a `free()` as well as a `malloc`, which is the cost that entry naming itself.

**`rexxcps` moved -2.76% in instructions against a predicted 0%, and the prediction was made without reading the program.** `samples/rexxcps.rex` contains `do j=1.1 to 2.2 by 1.1`, a **decimal** loop control: every pass adds `1.1` to a decimal and hands the result to this predicate. It also builds `acompound.key1.loop` from a `substr` result, an untagged operand that sends its `+ 1` down the general path. That is entry 4's error repeated -- "none of these programs calls a builtin whose result this changes", said of a program nobody had opened -- and it is written down as a miss rather than absorbed, because the axis it lands on is the one this candidate would otherwise have claimed nothing about.

#### Build identity

| | |
|---|---|
| BASE `rexx-run` | size=13832312, sha256 `8ef5c32e99503841b11aece93eb32200127366cf8b0742c12bc467212e1fe82d` -- **entry 7's HEAD binary reproduced byte for byte** at `c761c5f4`, after `cargo clean -p rexx-exec -p rexx-core -p rexx-num -p rexx-bench --release` |
| HEAD `rexx-run` | size=13849736, sha256 `35999021f97f24bb81e8c7a65083340084a8e1107087e11caf34dc8bc7f82141`; the binary **grew** by 17,424 bytes |
| oracle | the same three objects entry 1 fingerprints, re-hashed here and unchanged: `bb5bb8cc...`, `42136c40...`, `3536b763...` |

**The measured HEAD binary is the committed source's release build**, rebuilt after a full `cargo clean` at the end of the gates and hashed again to `35999021`, byte-compared against the binary the sittings ran.

#### The accept measurement: the two binaries alternating in one loop, twice

The accept rule's literal shape, and **not** `rexx-bench-suite`. Wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, one fresh empty working directory, base/head order **rotated every round**, seven rounds per axis. All 168 runs exited 0 and each axis's stdout hash is identical across all fourteen runs of it in both sittings.

**Sitting 2 is gated the way entry 8's re-measurement asks for**: six consecutive five-second samples of host idle read from `/proc/stat`, all at or above 90%, before the first run. It read 99.4, 99.0, 99.2, 99.1, 99.3 and 99.3. Sitting 1 ran before that entry landed, with `vmstat` showing 98% to 99% idle immediately before it and one-minute load average 0.65 rising to 1.10 across it.

| axis | s1 base | s1 head | **s1** | s2 base | s2 head | **s2 (gated)** | rounds with that sign |
|---|---:|---:|---:|---:|---:|---:|---:|
| `arith` | 2.9608 s | 2.4704 s | **-16.56%** | 2.9582 s | 2.4575 s | **-16.93%** | **7 of 7 in both** |
| `varlookup` | 2.5541 | 2.5763 | +0.87% | 2.5443 | 2.5730 | +1.13% | 7 of 7 in both |
| `emptyloop` | 1.9505 | 1.9653 | +0.76% | 1.9487 | 1.9604 | +0.60% | 6 of 7 in both |
| `compound` | 2.7723 | 2.7811 | +0.32% | 2.7815 | 2.7729 | -0.31% | 5 of 7 in both |
| `strings` | 5.4177 | 5.4534 | +0.66% | 5.4989 | 5.4178 | -1.47% | 4 of 7 in both |
| `alloc4c` | 1.4591 | 1.4819 | +1.56% | 1.4513 | 1.4534 | +0.14% | 5 of 7, then 4 of 7 |

Per-round on `arith` in the gated sitting: -16.7%, -16.8%, -17.6%, -17.2%, -16.1%, -16.9%, -16.9%. There is no round in either sitting in which it is not about a sixth.

**Three axes produced no usable wall figure and are recorded as such rather than as small movements.** `compound`, `strings` and `alloc4c` change sign between the two sittings and their sign does not hold inside either. That is the shape entry 8's re-measurement found on `alloc4c` and it is what the instruction column below is for.

`rexxcps` is not read as wall time, for entry 1's reason. Both binaries self-calibrated to the identical `100 x 100`, so their clauses-per-second figures are comparable: five rounds, order rotated, base median **2,517,320** against head **2,589,453** -- **+2.86%**, head ahead in all five.

#### Instructions beside cycles beside wall, which is what separates the work from the layout

`perf stat -e instructions:u,cycles:u`, three runs per side per axis, arms alternating, medians. A different configuration from the sittings above and read for work rather than for time.

| axis | instructions base | instructions head | change | cycles change | wall (gated) |
|---|---:|---:|---:|---:|---:|
| `arith` | 29,928,696,959 | 23,911,740,306 | **-20.10%** | -17.09% | -16.93% |
| `rexxcps` | 37,961,632,660 | 36,915,319,357 | **-2.76%** | -3.56% | +2.86% cps |
| `emptyloop` | 28,599,608,641 | 28,724,522,037 | +0.44% | +0.50% | +0.60% |
| `varlookup` | 43,548,512,766 | 43,684,529,060 | **+0.31%** | +0.63% | +1.13% |
| `alloc4c` | 11,212,656,827 | 11,242,704,364 | +0.27% | -1.82% | +0.14% |
| `compound` | 37,228,690,278 | 37,246,304,463 | +0.05% | -0.09% | -0.31% |
| `strings` | 63,152,777,126 | 63,180,769,526 | +0.04% | -0.79% | -1.47% |

**Two axes execute measurably less work and four execute measurably more.** `arith` and `rexxcps` are the mechanism. The other four are entry 3's displacement channel: `varlookup` retires **136 million extra instructions for work that did not change**, on the same shape entry 3 measured at 399 million -- a function on the arithmetic path changed size and its caller is inlined into `run_ops`, which every axis runs. It is a quarter of what entry 3 charged and it is the price of touching this code.

**`arith`'s instruction move is larger than its cycle move**, which is the opposite of entries 5 and 6, so what was removed retired at a *higher* instructions-per-cycle than what remains. Nothing here measures IPC and this entry does not explain it; rendering a decimal into a `String` is branch-light, arithmetic-heavy work, which is the shape that would do it.

**`alloc4c`'s -1.82% in cycles against +0.27% in instructions and +0.14% in wall is not claimed.** `cycles:u` does not count the kernel, and entry 6 measured that axis as the one where kernel time still moves.

#### The mechanism, measured directly rather than inferred from the axis

Two 200,000-iteration probes, `perf stat -e instructions:u`, twice each, from a fresh empty directory, both printing the identical answer on both binaries.

* **The clause that reaches the predicate**, `a = i / 3`: 1,254,429,248 instructions on base against **1,067,343,257** on head, **-14.9%**, answer `66666.6667` on both.
* **The control, which does not**, `a = i + 1`: 349,414,903 against 350,614,491, **+0.34%**, answer `200001` on both. Two tagged operands stay on `arith_small_int` and never construct a `Number`, so the only thing this pair can show is the displacement -- and that is what it shows.

That pair is what says the win is this predicate and not something else that moved with it.

#### What P3's control says today, which is less than it did

The candidate's own falsification named `compound` and `strings` getting slower the way P3's did. **They did not**: +0.32% and +0.66% in one sitting, -0.31% and -1.47% in the other, with the sign holding in neither and instruction counts of +0.05% and +0.04%. P3 moved them +1.6% and +2.7%.

**That control is weaker now than when it was written, and this entry says so rather than banking it.** P3 was measured before entries 3 and 4. `compound`'s `//` and `strings`' `LENGTH` both built `Number`s then and both stay tagged now, so those two axes may no longer be sensitive to this predicate's width at all. Their not moving is consistent with the set being unchanged and is not evidence for it. **The evidence for the set is the equivalence check below.**

#### Peak resident set, which moves nowhere and is recorded for that

`/usr/bin/time -f %M`, same wrapper, one run per side: `arith` 9,876 KB against 10,372, `strings` 11,680 against 11,420, `varlookup` 2,476 against 2,544, `compound` 2,436 against 2,576, `alloc4c` 179,552 against 180,400, `emptyloop` 2,476 against 2,500. Every axis is within a few hundred KB and in both directions, which is what has to happen when the same values are heap objects as before.

#### Correctness, which outranks the number, and here it *is* the work

**The predicate's contract is an equivalence, so the test is that equivalence over a generated population rather than a list of interesting values.** `rexx-num`'s `the_shape_predicate_answers_what_the_rendering_says` runs the old implementation -- render, refuse a `.` or an `E`, parse back -- as the oracle for the new one: **101,528 cases**, 37 mantissas chosen for the shapes the two can disagree about crossed with exponents `-24..=24`, both signs, and precisions 0 to 25 plus 1000 and `u64::MAX`. **22,511 are accepted, and 777 of those are outside `plain_integer`** -- the set P3 dropped, and the floor asserting it is what stops the test passing for a predicate that is `plain_integer` in disguise. Every accepted case is asserted to render identically under `FORM ENGINEERING`, which is what lets the caller decide a representation without knowing the form in force.

`the_tag_decision_is_the_rendering_read_back` (`value.rs`) does the same at the call site, where the tag's own 61-bit range narrows the answer further, over a grid crossing `i64::MAX`, `SMALL_INT_MAX` and one either side of each with twelve precisions.

**The differential, on both engines, at BASE and at HEAD.** 26 generated programs -- one per `DIGITS` in `{1,2,3,4,5,6,9,10,15,18,19,20,25}` by both `FORM`s -- of 2,965 arithmetic cases each, **77,090 cases**, every case put through five uses that would show a representation change: the rendering itself, its length, `DATATYPE(v,'W')`, a re-render through `+ 0`, and a concatenation. Errors are trapped per case and reported as their number. Run on the oracle and on both binaries on both engines:

* **All 156 descriptors are byte-identical between BASE and HEAD** -- 26 programs by two engines by stdout, stderr and exit status -- and the divergence-from-oracle set is identical line for line, 874 lines on every arm.
* **Those 874 are one pre-existing gap and it is not this candidate's**: `DATATYPE(v,'W')` answers `0` where the oracle answers `1` for a whole number of 19 digits or more, because this crate asks `Number::whole_value(ARGUMENT_DIGITS)` and that constant is 18. It is identical at BASE, it is unrelated to representation, and it is recorded here because the grid surfaced it and nothing else in the record names it.
* **The remaining 76,216 lines match the oracle on both binaries and both engines**, every rendering and every condition number among them.

**Under the collector's stress mode**, because a representation change alters which values are heap objects: the same 26 programs through `run_program_collect_every_alloc`, which collects on *every* allocation -- **2,125,348 collections**, stdout, stderr and exit status identical to the plain run of each. The committed corpus stress harness is green too, and its `NO_ALLOCATION_PROGRAMS` set is the pin that matters here: it is committed data naming exactly which corpus programs allocate nothing, in both directions, so a predicate that moved one program's values between the tag and the heap reddens it.

**Two mutations, and the pair is the finding.**

* **Narrowing the predicate to `plain_integer`'s set -- which is P3 -- reddens all three new tests, and the entire workspace without them stays green: 1439 passed, 0 failed**, corpus differential and dual-engine sweep included. So nothing that existed before this entry could have caught it. That is the same fact entry 4 recorded from the other side -- `Body::Text{b"46"}` and `SmallInt(46)` are observationally identical -- and it is why the equivalence check is the instrument and the differential is not.
* **Widening the exponential trigger by one** -- `adjusted > trigger` for `>=` -- reddens **8 existing tests**, the dual-engine sweep and both exempt-set harnesses among them. So the existing suite does catch a *wrong answer*; what it cannot catch is a *narrower set*, and those are different failures.

`crates/rexx-num/src/lib.rs` was restored from a `cp` copy after each and `sha256sum -c`'d, never `git checkout --`, and the release binary was rebuilt and re-hashed to `35999021`.

Gates: `cargo test --workspace` **1442 passed, 0 failed, 4 ignored** in dev and in release, run counts read rather than exit status alone. Inside the release run the corpus runner reported 10 passed and the dual-engine sweep 9. **BASE was re-counted rather than inherited, by running the suite at BASE before anything was edited**: **1440 passed, 0 failed, 4 ignored** in both profiles, which is entry 7's own figure; HEAD runs exactly two more, the two in `tests/rendered.rs` -- the third new test replaced one whose subject this change deletes. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean from a full `cargo clean`, with `Checking rexx-num`, `Checking rexx-core` and `Checking rexx-exec` confirmed in the log.

#### One false comment was removed rather than carried

`value.rs` carried a fifty-line doc comment describing `small_int_for` as deciding **by rendering** -- "that constraint turns out to force the right design", "guarantees a `SmallInt`'s rendering and a `Body::Num`'s rendering can never drift apart, which two independent implementations could not promise" -- and it was attached to `canonical_small_int`, two functions away, so it described neither the code above it nor the code below it. This change makes the first half false as well as misplaced. It is moved onto `small_int_for` and rewritten: the no-drift guarantee is now held by an asserted equivalence rather than by construction, which is a weaker warrant and says so.

#### Disposition: **accepted**

The candidate's stated falsification was the replacement accepting a different set, shown by a differential divergence or by `compound` and `strings` getting slower as P3's did. Neither fired: 156 descriptors are byte-identical between the two binaries across 77,090 cases on both engines, 101,528 generated cases agree with the rendering they replace, and the two axes P3 moved did not move in either direction with a sign that held.

**The hypothesis is confirmed by the route it named**, and the instruction count is the evidence: 6.0 billion fewer instructions on `arith`, with a probe pair isolating the clause that reaches the predicate (-14.9%) from one that does not (+0.34%).

Entry 2's ceiling construction for this candidate was 14.8% of the axis, implying 2.6658x -> 2.27x. The measured wall landing is **-16.9%**, past that ceiling, and the entry names the two costs the share did not count rather than claiming the share was wrong: the `str::parse` after the render, and `round_to`'s clone of the digit vector, whose `free()` a heap that never collected never used to pay.

Accepted **with the displacement recorded and not netted out**: four axes retire 0.04% to 0.44% more instructions for work that did not change, `varlookup` most at 136 million. Against `arith` losing a sixth of its wall time that is a trade worth making, and it is the second time this record has paid it.

**One thing this candidate does not do, and it was the brief's own hypothesis.** Entry 6's collector made `arith` slower, and the reasoning offered was that every value kept as a tagged immediate is an object never allocated, never marked and never freed -- so this would attack the regression's own mechanism. **Peak resident set says it did not**: 9,876 KB against 10,372, unchanged, because the set of values that are heap objects is exactly what this change was required not to move. What it removed is the *transient* rendering, not a retained object. The regression entry 8's re-measurement puts at +3.4% is more than covered by this candidate, and by a different mechanism than the one predicted.

Commit: `e1f080fbb2969fed83691f2f7d7f676ba2ab0ac6`, read back from `git log` after committing.

#### What this entry cannot say

* **It is two sittings on one Linux host**, unpinned, on a machine whose governor cannot be fixed. The second is gated on host idle; the first is not, and they agree on `arith` to 0.4 of a point.
* **It did not run `rexx-bench-suite`, so it carries no oracle ratios.** Every figure is against the immediately preceding binary, which is what the accept rule asks for. Where `arith` now sits against the oracle is unmeasured; entry 1 had it at 2.6658x and three accepted changes have moved it since.
* **`compound`, `strings` and `alloc4c` have no wall figure here at all.** Their signs flip between the two sittings and hold in neither, and only their instruction counts -- flat to within 0.05% to 0.27% -- say anything.
* **The equivalence is asserted over a generated population, not proved.** 101,528 cases with the boundaries in them is not every `Number`, and the population is this entry's own choice; a shape nobody thought of is a shape the floors cannot miss.
* **`rexxcps`' +2.86% is a clauses-per-second figure**, not a wall ratio, and its -2.76% instruction count is what it rests on.
* **The tree-walker was measured only for correctness.** Every wall, instruction and RSS figure is the IR arm; the predicate sits below both engines, so the tree-walker gets the same work removed and no benchmark here says by how much.

### Entry 10 -- entry 9's displacement figures withdrawn: the instruction instrument does not resolve half a per cent here

**No code change, no candidate.** This entry corrects entry 9, which is this record's own rule -- an entry that turned out wrong is corrected by a later entry saying so, not by editing the one that was wrong. Entry 9's disposition, its correctness results and its wall figures are untouched.

**What prompted it.** The instruction column is what entry 9 used to separate work from layout on five axes it did not mean to move, and it read those counts at **three runs per side** on the strength of entry 5's finding that this instrument reproduces to eight figures (a full range of 1.4e-8 on `varlookup`). That is not what it does at head. Re-measured at **six runs per side**, on a host gated the way entry 8's re-measurement asks -- six consecutive five-second `/proc/stat` samples at or above 90% idle, reading 99.2% at the sixth -- each side's own full range is:

| axis | entry 9's change | this run's change | base range | head range | resolved? |
|---|---:|---:|---:|---:|---|
| `arith` | -20.10% | **-20.08%** | 0.280% | 0.201% | **yes**, by two orders of magnitude |
| `rexxcps` | -2.76% | **-2.64%** | 0.027% | 0.011% | **yes** |
| `emptyloop` | +0.44% | +0.47% | 0.331% | 0.386% | marginal, 1.2x its own range |
| `alloc4c` | +0.27% | +0.73% | 0.904% | 1.448% | **no** |
| `varlookup` | +0.31% | +0.28% | 0.331% | 0.259% | **no** |
| `compound` | +0.05% | +0.05% | 0.171% | 0.356% | **no** |
| `strings` | +0.04% | +0.01% | 0.273% | 0.088% | **no** |

#### The three corrections

**1. Entry 9's "four axes execute measurably more" is withdrawn.** Four of those five movements are inside the instrument's own spread on the same axis, and the fifth is at the edge of it. The honest statement is that **no axis but `arith` and `rexxcps` shows an instruction move this instrument can resolve**, and that is a statement about the instrument rather than about the change.

**2. "`varlookup` retires 136 million extra instructions for work that did not change" is withdrawn**, and with it the reading that entry 3's displacement channel ran again at a quarter of its size. The number is real arithmetic on two medians; what it is not is separated from the spread those medians sit in. Entry 3's own displacement finding is not disturbed -- it was measured on a different instrument state and reported ranges of 1.4e-8 and 3.7e-4 beside it, which this run cannot reproduce.

**3. `varlookup` and `emptyloop` are left as wall movements with no measured cause.** Their wall figures stand -- `varlookup` +0.87% and +1.13% with the sign holding in 7 of 7 rounds in each of two sittings, `emptyloop` +0.76% and +0.60% at 6 of 7 -- and the attribution to extra instructions does not. Layout is the remaining candidate and nothing here measures it.

#### What stands

`arith` at **-20.08%** and `rexxcps` at **-2.64%** both reproduce at six runs with ranges two orders of magnitude below them, so entry 9's mechanism, its disposition and its 200,000-iteration probe pair are all unaffected: the probe that reaches the predicate moved -14.9% against a control at +0.34%, and the control's own +0.34% sits in the same unresolved band as the axes above, which is consistent with everything here.

#### The method finding, which outlives this candidate

**"Reproducible to eight figures" is a property of one axis on one binary at one time, and it was quoted here as a property of the instrument.** Measured at head, the same instrument's per-axis, per-arm full range runs from 0.011% to 1.45% -- a factor of a hundred across axes, with `alloc4c` the worst and `rexxcps` the best. Something changed between entry 5 and now; the collector landing between them is the obvious suspect and this entry establishes nothing about it.

**The rule that follows: a displacement claim below about one per cent needs that axis's own range measured beside it, in the same run.** Three reps produce a median that looks authoritative and a range nobody printed. Entry 9 printed the median.

#### What this entry cannot say

* **It re-measures instructions only.** Entry 9's cycle column for the same five axes inherits exactly this problem and is equally unresolved; it was not re-run.
* **It does not explain the spread.** Whether the collector, the allocator's address-space layout or something else widened it is unmeasured, and one candidate's re-measurement is the wrong instrument for that question.
* **It changes no disposition.** Entry 9 was accepted on `arith` at -16.9% wall with -20.1% instructions behind it, and both survive at six reps.

### Entry 11 -- the second profiling pass, and the queue it re-ranks, at `2bbecac9`

**No change, no hypothesis, no disposition.**
This entry optimises nothing and attempts no candidate.
It re-profiles the state seven accepted changes have left, re-measures every candidate still on entry 2's queue, and replaces that queue.

**Why it exists.** Unit 0's re-profile cadence: "after every accepted change, re-profile the axes it moved. The shares are now different, and the next candidate is chosen from the new profile, not from the entry attribution."
Entry 2's shares were measured at `610cb4ef`.
Since then `compound`'s `//` went on the small-integer path, a builtin's counted answer comes back tagged, every simple-variable write and the loop control bind to integer slots, a collector runs where none ever did, its trigger changed signal, and the tag decision stopped rendering numbers to text.
Every ceiling on entry 2's queue was a number about a binary that no longer exists.

#### What was measured, and with which instrument

**Two instruments, and which figures come from which is stated on every table below.**
Oracle ratios are **timed** -- `rexx-bench-suite` under the configuration fixed at the top of this file.
Shares are **sampled** -- `samply` 0.13.1 `--save-only`, analysed through `pollard` with `expand_inlines`.
The probes at the end are **counted** -- `perf stat -e instructions:u`, which is a third configuration and is read for work rather than for time.

| | |
|---|---|
| repo commit | `2bbecac9d7f9a81777511ff471c2755197dc50f0`, working tree clean before and after, printed by the harness in both blocks |
| `rexx-run` | size=13849736, sha256 `35999021f97f24bb81e8c7a65083340084a8e1107087e11caf34dc8bc7f82141` -- **entry 9's HEAD binary reproduced byte for byte**, after `cargo clean -p rexx-exec -p rexx-core -p rexx-num -p rexx-bench --release` |
| `rexx-bench-suite` | sha256 `4949a32114e8bf728598a05575ff02043920a75ef68fb8b89f1972edccc0c694` -- **entry 1's harness binary exactly** |
| oracle | the same three objects entry 1 fingerprints, re-hashed here and unchanged: `bb5bb8cc...`, `42136c40...`, `3536b763...` |

**The timed sitting is gated the way entry 8's re-measurement asks**: six consecutive five-second samples of host idle read from `/proc/stat`, reading 99.4, 99.0, 99.3, 99.1, 99.3 and 99.1 per cent before the first run.
Two blocks, both `--engine ir`, 2026-08-11 19:50:59 to 20:00:38 +02:00, both exiting 0, neither printing a "not a baseline" section, every axis in both blocks reporting stdout stable within each side and identical across the two sides.
One-minute load average 0.34 at the start and 1.06 at the end.

**The arm was named on every run and a hostile inherited value was overridden**: the suite was launched with `REXX_ENGINE=bogus` in its environment and both blocks print `| this crate's engine | REXX_ENGINE=ir |` in their provenance, which is entry 1's check re-run rather than assumed.

Thirteen profiles, one per axis per side, 1 kHz on this crate and 4 kHz on the oracle.
Each from a fresh empty directory it `mkdir`s itself, `/dev/null` on stdin, `ulimit -v 8388608`, stdout and stderr as separate files, `REXX_ENGINE=ir` set in the script.
Every profile exited 0 with an empty stderr and the bytes the suite records; `unsymbolicated_pct` ran 0.0% to 0.040% across the thirteen.
Profile durations against the timed medians: `strings` 5537 ms against 5390/5416, `arith` 2486 against 2471/2474, `compound` 2820 against 2774/2779, `varlookup` 2639 against 2620/2622, `alloc4c` 1463 against 1477/1479, `rexxcps` 3933 against 3882/3906.
`rexxcps` self-calibrated to the same `100 x 100` against the oracle's `200 x 100` under the profiler as in the sitting.

**A share below is a subtree total unless it says self**, and shares from different rows must not be added.

#### The current per-axis picture

This crate's median wall time over the oracle's, recomputed at full precision from the two medians the harness prints, beside where entry 1 left each axis.

| axis | b1 | b2 | **head** | block gap | entry 1 | move | distance from 1.0, in units of this axis's own block gap | of this crate's time, the fraction that has to go |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `alloc4c` | 1.2695 | 1.2785 | **1.2740x** | 0.71% | 1.9793x | **-35.6%** | 30 | 21.5% |
| `arith` | 2.1239 | 2.1344 | **2.1291x** | 0.49% | 2.6658x | **-20.1%** | 108 | 53.0% |
| `varlookup` | 2.1521 | 2.1614 | **2.1568x** | 0.43% | 3.7409x | **-42.3%** | 124 | 53.6% |
| `compound` | 2.4139 | 2.4229 | **2.4184x** | 0.37% | 5.8982x | **-59.0%** | 157 | 58.6% |
| `strings` | 6.2419 | 6.2914 | **6.2666x** | 0.79% | 10.5770x | **-40.8%** | 107 | 84.0% |
| `rexxcps` (cps) | 6.4447 | 6.4425 | **6.4436x** | 0.03% | 7.3466x | **-12.3%** | 2426 | 84.5% |
| `emptyloop` | 2.1830 | 2.1818 | **2.1824x** | 0.06% | 3.1816x | -31.4% | 928 | not in the bar |

**All six bar-bound axes remain NOT MET and none escalates**, on Unit 0's rule that an axis far from 1.0 relative to its own spread is decided; the nearest sits 30 block-gaps away.
`rexxcps` is a clauses-per-second ratio and not a wall-clock one, for entry 1's reason -- the two sides do different amounts of work.
Median cps, oracle then this crate: 16641496 / 2582182 and 16527997 / 2565464.

**The move column crosses seven commits and a sitting boundary, so it is not attributable to any one change**, and the per-change attributions are in entries 3 to 9.
What it is good for is the denominator every ceiling below is a fraction of.

`startup` remains not comparable and no ratio is taken: oracle 6.575 and 6.440 ms, this crate 1.943 and 1.699 ms.
`alloc`, `dispatch` and `heapshape` exit 120 with `rexx-exec: a message send is not implemented (Phase 5)` in both blocks, as they have since entry 1.

#### Where this crate's time goes at head, and where the oracle's goes on the same program

Sampled. Ours as a fraction of our own run, the oracle's as a fraction of its own -- two different denominators, which is why the absolute seconds are given wherever the comparison carries a candidate.

| block | `strings` | `rexxcps` | `arith` | `compound` | `varlookup` | `alloc4c` |
|---|---:|---:|---:|---:|---:|---:|
| glibc allocator family, self | 23.8% | 37.6% | 38.9% | 12.1% | -- | 23.4% |
| this crate's arena, `alloc_with_uncollected` self | 3.7% | 2.4% | 1.0% | -- | -- | 1.9% |
| `Heap::collect` | 5.2% | 7.6% | 2.5% | **0%** | **0%** | 9.9% |
| builtin resolution (`is_builtin` + the table scan) | **17.4%** | 3.0% | -- | -- | -- | 6.4% |
| the clause epilogue, `leave_clause` self | 3.7% | 1.1% | 1.3% | 5.3% | **32.0%** | 2.9% |
| `ProgramSource::line_of` | 2.5% | 4.8% | 1.5% | 3.1% | 6.2% | **11.2%** |
| `compound_parts`, the tail re-split | -- | -- | -- | **21.3%** | -- | 3.1% |
| `Interp::slot_of`, the name-keyed map | -- | 5.8% | -- | **16.1%** | -- | 4.1% |
| `read_at`'s id-keyed `HashMap<SymbolId, usize>` | 6.9% | 0.9% | -- | -- | -- | 2.6% |

And the oracle, on the same six programs:

| block | `strings` | `rexxcps` | `arith` | `compound` | `varlookup` | `alloc4c` |
|---|---:|---:|---:|---:|---:|---:|
| `MemoryObject::newObject` | 35.9% | 22.2% | **8.3%** | -- | 52.7% | **74.9%** |
| `DeadObjectPool::findFit` self | 19.2% | 8.6% | 2.7% | 7.6% | 20.6% | 3.9% |
| `MemoryObject::collect()` | 3.1% | 5.1% | 1.4% | -- | -- | **60.3%** |
| `NumberString` and `Numerics::` self | -- | -- | **74.1%** | -- | -- | -- |
| the per-clause loop, `RexxActivation::run` self | -- | 5.3% | -- | 1.8% | -- | -- |
| builtin name resolution | **none** | **none** | -- | -- | -- | **none** |

**Five things fall out of the two tables, and three of them change the queue.**

* **`arith`'s decimal kernels are still not the gap, and the margin has widened.** Grouping every `NumberString` or `Numerics::` frame the oracle spends **74.1% self**, 92.5% total -- 0.871 s of a 1175 ms run. Grouping every `rexx_num::` frame this crate spends **21.2% self**, 82.7% total -- 0.527 s of a 2486 ms run. So this crate's kernels are about **1.65x faster** than the oracle's in absolute self time, against the roughly 1.5x entry 2 measured, and the whole of `arith`'s 2.13x sits in the allocation and copying around each `Number`. The oracle's whole `newObject` subtree on that axis is **8.3%**; this crate's glibc allocator family alone is **38.9%** self. **On `arith`, allocation is the differentiator and nothing else is.**
* **On `strings` it is not.** Entry 2 gave the allocator 45.7% of that axis and implied 5.74x. At head this crate spends 27.5% there and **the oracle spends 35.9% of its own `strings` run in `newObject`** -- a larger share than ours. Removing the whole of this crate's allocation cost leaves 4.54x. The allocator is no longer where `strings`' gap lives.
* **Builtin dispatch is.** `builtin::dispatch`'s whole subtree is 46.5% of `strings`, and 17.4 points of the axis are spent deciding *which* builtin to run. The oracle's `RexxExpressionFunction::evaluate` reaches its builtin through a translate-time jump thunk and no name resolution appears in its profile at all.
* **`memcmp` on `compound` is still shared and still not the differentiator**, reproducing entry 2: this crate 17.1% self, the oracle **17.7% self**.
* **`alloc4c`'s denominator is still inflated, reproducing entry 2 almost exactly.** `newObject` is **74.9%** of the oracle's run on that axis against entry 2's 74.5%, and `collect()` is **60.3%** against its 60.5%, with `CompoundTableElement::live` 22.1% self re-marking a growing table this crate never marks. The oracle's non-collector work is 0.397 x 1.163 s = 0.46 s against this crate's 1.477 s, so **on the work both sides actually do the axis is about 3.2x, not 1.27x**, and entry 2's instruction that no candidate be aimed at that ratio stands with a number behind it.

#### The collector, profiled for the first time

Entry 6 landed it, chose its policy on an argument rather than a measurement, and said so.
This is the first profile of what it costs.

**Marking and sweeping are not the cost. The `free()` of each reclaimed payload is the whole of it.**
`Heap::collect`'s own self time is 0.3% of `strings`, 0.6% of `rexxcps` and 0.3% of `arith`; `core::ptr::drop_glue::<rexx_core::heap::Slot>` underneath it is **4.7%, 6.7%, 7.4% and 2.2%** of `strings`, `rexxcps`, `alloc4c` and `arith`.
That is the same shape entry 6 measured on `arith` alone, now reproduced on four axes.

**This rules out most of the new candidate population the re-profile was expected to propose.**
Cheaper marking buys a fraction of half a per cent.
A generational or incremental scheme changes *when* marking happens, and marking is not what is being paid.
What is left is the payload: pooling it, reusing a swept slot's buffer for the next object of the same shape, or not giving each object a separately-allocated payload at all -- which is the same lever as "stop calling the allocator once per value", reached from the other end.

**Two axes collect zero times and are unaffected**, `compound` and `varlookup`, which is entry 7's collection counts showing up in a profile.

#### `rexxcps`, the axis nothing has been aimed at

It is second-furthest from the bar and it has moved -12.3% without a single change aimed at it, which is what the other six changes' shared work bought it.

**Its profile is the most allocator-dominated of any axis**: the glibc family is 37.6% self, this crate's arena another 2.4%, and the collector 7.6% on top -- 47.6% of the run in getting memory and giving it back.
The oracle spends 22.2% on the same program.
Beside that: `Interp::slot_of` 5.8%, `line_of` 4.8%, builtin resolution 3.0%, `spec_to_string::<i64>` 3.4%.

**Its `slot_of` is stem and `PARSE` work, not simple variables.**
The callers are `tail_key`'s `read_by_name` (31%), `stem_get` (20%), `assign_expr_target` under `exec_parse`'s `assign_targets` (34%) and `stem_set` (10%).
Entry 5 bound the simple-variable write and the loop control to plan slots and deliberately left stems, compound targets and `PARSE` resolving by name; `rexxcps` is the axis that pays for that, and `compound` pays more.

#### Whether `alloc4c` can be measured on this host: **no, not at the per-cent level**

Entry 8 recorded that five sittings produced no usable wall figure on this axis and that the sign was unstable within sittings.
This entry asks the prior question -- what can the instrument resolve here at all -- with two controls in the accept rule's own shape.
Timed, nine rounds each, one binary, order rotated every round, gated on six `/proc/stat` samples at or above 90% idle (99.2%, 99.1%, 99.3%, 99.2%, 99.1%, 99.2%).

| control | what it is | per-round range | median difference | rounds with the expected sign |
|---|---|---:|---:|---:|
| `alloc4c` null | `alloc4c.rex` against **itself** | -3.85% to +2.55% | -0.18% | 4 up, 4 down, 1 zero |
| `alloc4c` +1% | `alloc4c.rex` against `bench-control/alloc4c-101.rex` | -3.70% to +6.05% | **+1.04%** | **4 of 9** |
| `varlookup` null | `varlookup.rex` against **itself** | -0.87% to +3.21% | +0.07% | 7 up, 2 down |

**A known +1% difference on `alloc4c` does not hold its sign across the alternations**, and the accept rule's own wording is that "reproducible" means the sign holds.
Two identical arms differ by up to 3.85% in a round.
An earlier sitting of the same null control, 20 minutes before, read a median of **+0.61%** where this one reads -0.18%, so even the median's sign moves between sittings.
`varlookup` under the identical harness keeps six of its nine rounds inside 0.15%, and its two worst rounds are +3.21% and -0.87%.

**It is not a duration effect and it is not host load.** `alloc4c` is the *shortest* axis at 1.48 s and the quietest possible host was gated for.
The instrument load cannot touch says the same thing, which is entry 8's own method: user time alone, three runs, reads 1.41 / 1.42 / 1.45 s on `alloc4c` -- a 2.8% range -- against 2.61 / 2.60 / 2.60 on `varlookup`.
What is different about the axis is its live set: peak resident 180,748 KB and 55,869 minor faults per run, against 2,740 KB and 203 faults on `varlookup`, with the fault count itself stable to one fault.
So the varying quantity is what it costs to touch 180 MB, not how often it is touched -- **a mechanism this entry proposes and does not measure**; no TLB or cache counter was read.

**What would be needed.** Not more rounds under the accept rule, which forbids them for candidate selection.
Either an instrument that is not wall clock -- and `perf stat -e instructions:u` does not rescue it, entry 10 having measured `alloc4c`'s own per-arm instruction range at head as 0.90% and 1.45%, the worst of any axis -- or a variant of the program whose live set is small enough that the axis stops being a memory-system measurement.
**Until then `alloc4c` cannot be a candidate's acceptance axis**, and a change may only be accepted on it if some other axis carries the verdict.
That compounds with the finding above: its 1.27x is also not measuring the thing the axis is named for.

#### The re-ranked queue

Ranked by the largest share on a bar-bound axis, as entry 2 was, **not** by confidence and not by cost.
Every ceiling is entry 1's construction: this axis's head ratio multiplied by one minus the share, on the assumption the cost goes to zero and nothing else grows.
It is the loosest possible bound, the shares are not additive, and the sampled share is one sample of a distribution nobody characterised.

**1. Allocation: fewer of them, and cheaper.** Ceiling **39.9% `arith`** (38.9% glibc self + 1.0% arena), **40.0% `rexxcps`**, 27.5% `strings`, 25.3% `alloc4c`, 12.1% `compound`.
Implied `arith` 2.1291x -> **1.28x**, `rexxcps` 6.4436x -> 3.87x, `strings` 6.2666x -> 4.54x.
*Mechanism, three levers and they are not equivalent.* **(a) An inline digit buffer for a small `Number`**, which is where `arith`'s allocations come from -- `drop_glue::<rexx_num::Number>` is 12.2% of that axis and `Number::clone` 7.7%, both of them the digit `Vec<u8>`. **(b) An inline buffer for a short string**, which entry 4 examined and set aside because the builtins build their result in a `Vec` before `Body` sees it. **(c) A different global allocator**, entry 8's queue addition, which changes what each call costs rather than how many there are.
*Where the ceiling is real and where it is not.* On `arith` the oracle's whole `newObject` subtree is **8.3%** against this crate's 38.9% allocator family, so nearly the whole share is a differentiator. On `strings` the oracle's is **35.9%** against this crate's 27.5%, so it is not one at all, and a candidate aimed at `strings` through this lever is aimed at the wrong thing.
*What would falsify it.* A paired run of lever (a) moving `arith` by less than about 15%; or peak resident set not falling.
*Risk to the sharing rule: none.* `Heap`, `Slot` and `Number` sit below both engines.

**2. The clause epilogue's 104-byte `Failure`.** Ceiling **32.0% on `varlookup`**, 5.3% `compound`, 3.7% `strings`, 2.9% `alloc4c`, 1.3% `arith`, 1.1% `rexxcps`.
Implied `varlookup` 2.1568x -> **1.47x**, `compound` 2.4184x -> 2.29x.
*This is entry 2's candidate 5 and it has not evaporated; it has become the largest single block on `varlookup`.* Entry 2 gave it 11.4% there. `Interp::leave_clause::<RegionEnd>` is **32.0% self** at head, inlined into `leave_stepped_clause` and thence into `run_ops::<false>`, which is where entry 2 read the two 104-byte stack-to-stack copies out of the asm.
*The premise was re-read from this binary, not inherited.* `gdb -batch -ex 'print sizeof(rexx_exec::error::Failure)'` against the profiled `rexx-run` answers **104**, and `Raised` answers 104 -- so `Failure`'s width is still its cold arm's. `Flow` is 24.
*Mechanism.* `Failure::Raised(Box<Raised>)` makes `Failure` pointer-sized and `Result<T, Failure>` fit in registers; `Raised` is built on the error path only.
*What would falsify it, and it needs saying twice as loudly as entry 2 said it.* **Sampling attributes time; it does not attribute instructions**, and 32% of an axis attributed to a function whose source is three branches and a call is exactly the shape a mis-attributed stall produces. The confirming instrument is the change itself: re-read the asm to check the copies are gone, then take the paired run. If they vanish and `varlookup` moves less than about 10%, the attribution was wrong.
*The oracle's own clause boundary* is 1.8% self on `compound` and 5.3% on `rexxcps`.
*Risk to the sharing rule: none.* One type in `error.rs`; both engines return it.

**3. A compound variable's tails, re-split from its source spelling on every access.** Ceiling **21.3% on `compound`**, 3.1% `alloc4c`; implied `compound` 2.4184x -> **1.90x**.
*Mechanism.* `Interp::tail_key` is 28.5% of `compound` and `rexx_parse::ast::compound_parts` is 21.3% of the axis, reached from `tail_key`, from `read_symbol`'s `Compound` arm and from `assign_expr_target`. It takes the interned spelling of the whole dotted symbol, finds the first `.` with `str::find::<char>` (7.7% of the axis), splits the rest on `.` and collects a fresh `Vec<Tail>` (3.7%). **All of that is a pure function of the symbol id** and could be computed once when the plan is built.
*The oracle does not do it, and that is measured rather than read.* Counted: the same 400,000-pass loop with the stem spelled `a.` and with it spelled `abcdefghijklmnopqrstuvwxyzabcdefghi.` -- same tail count, same iterations, same answer -- costs this crate **+274,756,342 instructions, +9.7%**, and the oracle **+16,582 instructions, +0.001%**. Thirty-four extra characters of source spelling cost this crate **687 instructions per pass** -- across the two compound accesses that pass makes -- and the oracle nothing at all. Both sides reproduced twice, this crate's two ranges 2.9e-5 and 2.5e-5 and the oracle's 2.0e-5.
*What remains shared and must not be claimed.* The oracle's `CompoundVariableTail::buildTail` is 13.9% of its own `compound` run and `Numerics::formatWholeNumber` 7.3% self -- evaluating the tail expressions and rendering an integer tail to text, which this crate also pays (`spec_to_string::<i64>`, 4.6%). Only the **split** is this crate's alone.
*What would falsify it.* A paired run moving `compound` by less than about 12%; or a divergence where a tail's spelling is not fixed at plan-build time -- `INTERPRET`, and a compound whose stem is aliased, are the two shapes to check.
*Risk to the sharing rule: real.* The cached split belongs on the symbol table or the plan, where both engines read it, not on an `Op`.

**4. Resolve a builtin call at compile time.** Ceiling **17.4% on `strings`** (`is_builtin` 9.0% + the linear `Iter<Builtin>::find` 8.4%), 6.4% `alloc4c`, 3.0% `rexxcps`; implied `strings` 6.2666x -> **5.18x**.
*This is entry 2's candidate 6 and it has not evaporated either; its share has nearly doubled*, from 9.1%, because the axis around it shrank by 41% while the cost per call did not.
*Mechanism, and it is worse than entry 2 recorded.* `is_builtin` runs **twice on every builtin call** -- once in `Interp::resolve_call` (`run.rs:3966`) and again at the top of `builtin::dispatch` (`builtin/mod.rs:677`) -- and each run is a `std::str::from_utf8` validation plus a `HashSet<&str>` lookup with SipHash. Then `IMPLEMENTED.iter().find(...)` walks a 58-entry table comparing byte slices. `_memcmp_evex_movbe` is 14.8% of `strings`, of which **6.3 points is that table scan** and 7.7 is genuine string searching inside `CHANGESTR` and `POS`.
*The linear scan is confirmed by counting, with the oracle as the control for the bodies.* `LEFT` sits at table index 27 and `RIGHT` at 37. Over 400,000 calls this crate costs **+51 instructions per call** for the ten extra entries; the oracle costs **+18 per call** for the same two builtins, which is the body difference and nothing else. Net **about 3.3 instructions per table entry scanned**, so `SUBSTR`, at index 42, pays roughly 135 instructions of pure table walk on every call.
*What would falsify it.* A paired run moving `strings` under about 8%.
*Risk to the sharing rule: real if the resolved index goes in `Op` only.* It belongs on the plan or the instruction, where both engines read it.

**5. The two variable maps that are still hash lookups.** Ceiling **16.1% on `compound`** (`Interp::slot_of`, name-keyed), 6.9% `strings` (`read_at`'s id-keyed map), 6.7% `rexxcps`, 6.7% `alloc4c`; implied `compound` 2.4184x -> **2.03x**, `strings` 6.2666x -> 5.83x.
*Two sites, one mechanism: a hash where a dense index would do.*
**(i) The id-keyed `HashMap<SymbolId, usize>`.** `eval.rs:443`-`445` pass `None` for the slot on every `ExprKind::Variable`, `Stem` and `Compound`, so **every variable read inside a compiled expression tree** resolves through that map -- `Op::Load` is the only reader that carries a slot. On `strings` the callers are `eval_argument` (68% of it), `concat` (20%) and `eval_arithmetic` (12%): a builtin's arguments, not its statements. `SymbolId` is an index, so the map can be a `Vec` and the lookup an array index.
**(ii) The name-keyed `HashMap<Box<[u8]>, usize>` behind `Interp::slot_of`**, 16.1% of `compound` and 5.8% of `rexxcps`, reached by every stem read, stem write, compound tail resolution and `PARSE` target -- the set entry 5 deliberately left unresolved.
*What would falsify it.* Lever (i) moving `strings` under about 4%; or a frame whose slots grow under a cached index, which is the check entry 5 already wrote the probes for.
*Risk to the sharing rule: none for (i)*, which changes one data structure below both engines. **Real for (ii)**, for exactly the reason entry 2 and `compile.rs` both give: the resolution must stay an argument to the one `assign_expr_target`, never a second store path.

**6. Stop binary-searching the line table once per stepped clause.** Ceiling **11.2% on `alloc4c`**, 6.2% `varlookup`, 4.8% `rexxcps`, 3.1% `compound`, 2.5% `strings`, 1.7% `arith`; implied `varlookup` 2.1568x -> **2.02x**, `rexxcps` 6.4436x -> 6.13x.
*This is entry 2's candidate 7, and it has not evaporated: 6.2% -> 11.2% on `alloc4c`.*
*Mechanism, unchanged.* `enter_stepped_clause` calls `clause_line` -> `ProgramSource::line_of`, a binary search over the line-offset table, unconditionally, because `SIGL` must stay correct whether or not `TRACE` is on. The line is a static property of the instruction.
*Confirmed by counting, and this is the cleanest control in this entry.* The identical 400,000-iteration loop, preceded by 0, 1,000 and 20,000 comment lines, printing the identical answer: this crate executes 700,826,181 -> 762,806,995 -> 813,996,977 instructions, **+8.8%** and **+16.1%**. The oracle executes 327,489,517 -> 328,208,624 -> 341,845,566, **+0.22%** and **+4.4%** -- and its increment is not per clause, since it does not scale between the two padded files the way a per-clause cost must. **This crate's per-clause cost grows with the program's line count and the oracle's does not**, because its instruction objects carry their line. Reproduced twice per point, this crate's three ranges 1.8e-7, 1.9e-7 and 8.1e-7 and the oracle's 4.1e-6, 1.1e-5 and 6.5e-5.
*Its loudest axis is now its most trustworthy one*, which reverses entry 2's warning: `alloc4c` and `rexxcps` carry it, and `emptyloop` is no longer where it reads highest.
*Risk to the sharing rule: real.* Putting the line on the instruction keeps one implementation; putting it in `Op` alone leaves the tree-walker computing it a second way.

**7. Make the collector's reclamation not cost a `free()` per object.** Ceiling **9.9% on `alloc4c`**, 7.6% `rexxcps`, 5.2% `strings`, 2.5% `arith`; implied `rexxcps` 6.4436x -> **5.95x**.
*Mechanism, and it is the only one the profile supports.* `drop_glue::<Slot>` is 4.7% to 7.4% of the four axes that collect, and `Heap::collect`'s own self time is under 0.6% everywhere. Pool the payloads, or reuse a swept slot's buffer.
*Explicitly not queued: cheaper marking, generational collection, incremental collection.* Marking is not measurably paid on any axis here, so their ceiling is a fraction of half a per cent, which Unit 0 disposes of at this distance.
*It overlaps candidate 1 and the two must not be added*: an object that is never allocated is never freed either.
*What would falsify it.* A paired run moving `rexxcps`' clauses per second by less than about 4%.
*Risk to the sharing rule: none.* `Heap` sits below both engines.

**Not re-ranked and still queued as entry 8 left it: replacing the global allocator.**
It now has a per-axis picture rather than a single share, and it is lever (c) of candidate 1 above rather than an item of its own.
The decision it carries is unchanged and is not a measurement: `libmimalloc-sys` and `tikv-jemalloc-sys` both compile and link a C library, in a clean-room Rust reimplementation, on five platforms.

#### What evaporated

* **`required_string`'s `[u8]::to_vec`.** Entry 4 measured it at **15.5% of `strings`** and set it aside as a borrow-structure change deserving a candidate of its own. At head it is **about 3.5%** of that axis, summed over its three callers inside `dispatch`. Two accepted changes shrank the axis around it and it is no longer worth a candidate.
* **Candidate 1's ceiling on `strings`.** Entry 2 implied 5.74x from a 45.7% share. The share is 27.5% and **the oracle's own is 35.9%**, so the lever survives on `arith` and `rexxcps` and does not survive as a `strings` candidate. This is the one place the queue was aimed at the wrong axis.
* **`alloc4c` as an acceptance axis**, for two independent reasons measured above: a known 1% difference does not hold its sign on it, and 60.3% of the oracle's time on that program is the oracle's own collector.
* **Nothing else.** Entry 2's candidates 5, 6 and 7 are all still there and all three have **grown** as shares -- 11.4% -> 32.0%, 9.1% -> 17.4%, 6.2% -> 11.2% -- because seven accepted changes shrank the axes around costs that are fixed per clause and per call. That is the opposite of what a re-profile was expected to find, and it is the most useful thing in this entry: **a fixed per-clause cost gets more valuable every time something else is removed.**
* **The `smallvec` dead end is still a citation with no measurement behind it.** Not chased here; entry 2's warning stands unchanged.

#### Is the bar reachable

**Not on any axis but `alloc4c` from this queue alone, and on `strings` and `rexxcps` not from any queue of this shape.**
The arithmetic is set out rather than asserted, and it says two different things about two groups of axes.

**The reclamation row is deliberately absent from every sum below**, because the `free()` it pays is counted once already inside the glibc allocator family's self time; adding candidate 7 to candidate 1 would double-count it.

| axis | must go | the identified blocks the oracle does **not** also pay, summed | what remains if every one of them went to zero |
|---|---:|---|---:|
| `alloc4c` | 21.5% | 11.2 + 6.7 + 6.4 + 3.1 + 2.9 = **30.3** | **0.89x**, past the bar -- and unmeasurable |
| `arith` | 53.0% | 39.9 allocation (the oracle's own is 8.3) + 1.5 + 1.3 = **42.7** | **1.22x** |
| `varlookup` | 53.6% | 32.0 + 6.2 = **38.2** | **1.33x**, and 32 of those points are one candidate |
| `compound` | 58.6% | 21.3 + 16.1 + 5.3 + 3.1 = **45.8** | **1.31x** |
| `strings` | 84.0% | 17.4 + 6.9 + 3.7 + 2.5 = **30.5** | **4.35x** |
| `rexxcps` | 84.5% | 5.8 + 4.8 + 3.0 + 1.1 = **14.7** | **5.50x** |

**Read the last column first, because it is the finding: not one bar-bound axis except `alloc4c` reaches 1.0 even if every candidate on this queue lands in full.**

**`arith`, `varlookup` and `compound` land between 1.22x and 1.33x, and that is a near miss rather than a wall.**
Each has an unattributed remainder this pass did not decompose -- `varlookup`'s `run_repeating` self time is 9.4% and `run_ops`' own is 7.0%, `arith`'s `rexx_num` self time is 21.2% against an oracle that is 74.1% self and slower in absolute terms, `compound`'s `memcmp` is 17.1% and shared.
A third profiling pass on those three, taken after this queue has been consumed, is where their last 25 per cent has to come from, and nothing here says it is not there.

**`strings` and `rexxcps` are short by a factor of four at the same ceiling, and that is a wall.**
It is not a shortfall in the queue; it is a statement about what is left over, and what is left over is not diffuse.
On `strings`, after removing every block above, this crate spends about 3.9 s doing the string work the oracle does in about 0.55 s -- **7x** -- and `builtin::dispatch`'s subtree alone is about 2.5 s against the oracle's 0.52 s for the same 3,000,000 iterations.
On `rexxcps` the comparison has to be per clause, because the two sides run different iteration counts: this crate's own printed figure is 387 ns per clause against the oracle's 60 ns, of which allocation is **155 ns** here and **13 ns** there, and giving this crate the oracle's own absolute per-clause allocation cost still leaves **4.1x**.

**Stopping rule 2 has not fired and this entry does not claim it has.**
It reads "re-profiling yields no candidate whose effect survives the pairing on an axis still short", and re-profiling yielded seven, four of them large.
What the table says is narrower and must not be inflated into the rule: **no combination of the candidates this profile can name reaches 1.0 on `strings` or `rexxcps`**, and on the other four it reaches 0.89x to 1.33x, so those need this queue consumed and then re-profiling rather than a verdict now.
The rule fires when a re-profile stops proposing candidates that survive the pairing. This one is not that re-profile.

**The shape of change that would reach them, named because the plan asks for it.**
Every candidate 3 to 6 above is the same defect wearing a different coat: **a fact the parser knew is re-derived at run time**, once per clause or once per call.
The line is looked up by binary search; the builtin is found by name in a table, twice; the compound's tails are re-split from source text; the variable's slot is hashed.
Phase 4e's IR removed exactly two of these, for the statement forms it compiles -- `Op::Load` and `Op::Store` carry slots -- and stopped at the statement boundary.
`strings` and `rexxcps` are the two expression-heavy, call-heavy axes, and they are the two the IR's compile-time resolution never reached.
**So the structural change is to extend compile-time resolution from statements to expressions**: every `ExprKind` node carrying what the plan already knows, rather than four separate local fixes each rediscovering that the answer was available earlier.
It subsumes candidates 3, 4, 5 and 6, which together are 26.8 points of `strings` and 14.5 of `rexxcps`, so on its own it leaves those two axes at about **4.59x** and **5.51x**.
**So it is necessary and it is not sufficient**, and what is left after it is the object model -- how a value is allocated, held and freed -- rather than any block a profile names.
That is a Phase 4f finding to hand on rather than a candidate to attempt in it.

#### What this entry cannot say

* **It is one profiled run per axis per side.** Every share is a single sample of a distribution nobody characterised, and a re-profile will read a different small number. Entry 10's finding applies with full force: an instrument's reproducibility is a property of one axis on one binary at one time.
* **Sampling attributes time; it does not attribute instructions.** Candidate 2 rests entirely on that distinction and its own text says so; candidates 3, 4 and 6 have counted probes behind them and the rest do not.
* **The shares are not additive and no combination is predicted.** Allocator self time sits inside every other subtree, candidate 7 overlaps candidate 1, and candidates 4 and 5 overlap on the builtin call path.
* **`emptyloop` was profiled and is not reported**, because the bar does not bind it and entry 2 already recorded that its codegen sensitivity makes it the axis least able to prove anything.
* **The tree-walker was not measured at all.** Every figure here is the IR arm.
* **The mechanism offered for `alloc4c`'s variance is a proposal.** No TLB or cache counter was read; what was measured is that its user time varies 2.8% run to run and that it touches 180 MB where `varlookup` touches 2.7 MB.
* **It changed no code.** No candidate was attempted and no prototype was built; the binary profiled is entry 9's HEAD reproduced byte for byte, and the only files written were throwaway probes in the session scratchpad, run from fresh empty directories.
* **Nothing here is a correctness statement.** Both sides printed identical bytes on every axis and every probe printed the same answer on both interpreters, which guards against measuring a run that did not do the work. No differential ran.

---

### Entry 11 -- two independent consultations, both of which say this loop has been optimising the wrong layer

**Commissioned 2026-08-11 by Moritz.** Two agents were asked the same question with no sight of each other's answers, and both were required to write their recommendation **before** reading this record or the Phase 4e spec, then to add a section on where they agreed and disagreed with it. One was asked as a practitioner, one from the literature. Their full answers are in the session workspace, which is git-ignored, so what they establish is recorded here.

**Neither ran anything.** Both were forbidden to build, benchmark or profile, because the expression spike owned the machine. **So everything below is a reading of code and of the existing profile, not a measurement**, and this record's own standard applies: a mechanism asserted without a build is a hypothesis. What makes it worth an entry is that two independent readings reached the same one.

#### The convergent finding: it is representation, not dispatch

**The framing this loop has been using -- "we have an instruction stream and we are still 5 to 7 times a tree-walker" -- reads as a paradox and is not one.** An instruction stream and a tree-walk differ in how the interpreter *arrives* at an operation, not in what the operation costs. The reference's arrival cost is a few nanoseconds out of a 60 ns clause; ours is a few nanoseconds out of 387 ns. **The IR is competing for a slice that was never large on either side.**

What makes the reference fast is that its values are cheap, and both consultations cite the same code:

* `RexxString` ends in `char stringData[4]` and `NumberString` in `char numberDigits[4]` -- trailing flexible arrays, so **a string or a number is one allocation with its bytes inline**, from a segment the interpreter bump-allocates.
* Arithmetic works in `char resultBufFast[FAST_BUFFER]`, `FAST_BUFFER = 48` -- **a stack buffer, zero heap traffic for intermediates.**
* `RexxInteger::plus` adds machine integers whenever both operands are valid under the current `DIGITS`, and `-10..100` are interned.
* `CompoundVariableTail` builds a tail in a stack buffer with an inline `MAX_SYMBOL_LENGTH` array, **allocating nothing in the common case.**

Against that, one of our short strings is a **96-byte arena slot plus a separate `malloc`** for its `Vec<u8>`, reached through a generation-checked handle; `Number` holds **one heap byte per decimal digit**; and `Interp::to_number` returns an **owned clone**, so merely reading a numeric variable allocates. On `i = i + 1` with one operand off the small-integer path this system performs on the order of four allocations where the reference performs zero to one. **That is 155 ns against 13 ns, and neither engine can do anything about it, because both engines share it.**

**Inlining short payloads is what every comparable system does.** Ruby stores strings up to 23 bytes inside its 40-byte slot; CPython's compact unicode objects do the same; ooRexx does the same. For the decimal tower, decNumber -- written by Rexx's own author -- packs several digits into each coefficient unit rather than one per byte, and libmpdec, which became CPython's `_decimal`, uses base-10^9 limbs. **This system is the outlier.**

#### The mechanism neither this record nor the Phase 4e work had named

**In this codebase the borrow checker is a performance decision, and it has never been priced.**

`Interp::to_text(&mut self) -> Cow<'_, [u8]>` borrows the **whole interpreter**, because it lazily fills the `num`/`text` caches. So any code needing two operands' bytes at once, or bytes plus a later allocation, cannot hold the borrow and **must buy its way out with an owned copy**. It appears in a profile only as `malloc`; it appears in review as a locally justified line; and the comments at those sites *explain* the copy rather than flagging it as a cost. `stem.rs:281` and `builtin/mod.rs:790` are named as the clearest instances, **and the second is on the hot path of the worst benchmark.**

#### The criticism of this record's own arithmetic, which should be acted on

**Entry 10's "removing every identified cost still leaves 4.4x and 5.5x" is challenged as unsupported, and the challenge is specific.** Summing independent removals assumes the costs are additive and disjoint. The argument copy, the payload `malloc`, the 96-byte slot write, and the collection the churn eventually triggers are **one cost with four names**. If the profile attributed them to four blocks and entry 10 subtracted all four, **the residual is inflated and the real post-fix figure is better than 4.4x.** If the profile grouped them, the residual is genuine and there is a mechanism nobody has named.

**Those two cases lead to opposite decisions**, and the proposed test is cheap: land the free representation fixes, measure, and compare the delta against what the profile predicted. **If they disagree, entry 10's 4.4x and 5.5x are withdrawn.**

#### The `Op` width budget, challenged on a different ground than it was defended on

`const _: () = assert!(size_of::<Op>() == 12)` is correct discipline **in a bytecode VM where dispatch dominates**. Here dispatch does not dominate, and the budget is already distorting design: it is the stated reason `PlanSlot` is a sentinel `u32` rather than an `Option`, and the reason a call site's fields were pushed off the op into a side table.

**Entry 10 answered a different question and its answer stands**: `run_ops`' self time is 1.1% to 4.9%, so *widening* `Op` cannot buy anything on dispatch. What was never measured is what the *budget* costs elsewhere. **A width budget defended by an assertion should be defended by a measurement, and there is not one.**

#### What this changes

* **The expression spike now tests a hypothesis rather than founding a plan.** If full expression promotion measures small, it corroborates both consultations; if it measures large, one of them is wrong in an interesting way.
* **Representation work precedes the object model**, which is Moritz's own sequencing argument reinforced: the value layer sits *below* the object model, and Phase 5 will build 32 classes on whatever that layer is.
* **Optimiser passes come last**, when there is a validated dispatch site and a workload -- `dispatch.rex` is scoped out of this bar and deferred to Phase 5, so the axis that would most reward an inline cache is not currently measured at all.

---

### Entry 12 -- the value-representation design's option B attempted and **accepted**: a small `Number`'s digits held inline

**BASE is `623f093f5`**, whose `rexx-run` reproduces entry 11's profiled binary **byte for byte** -- size 13849736, sha256 `35999021f97f24bb81e8c7a65083340084a8e1107087e11caf34dc8bc7f82141` -- after `cargo clean -p rexx-num -p rexx-core -p rexx-exec -p rexx-bench --release`. **Eight commits separate entry 11's profile at `2bbecac9` from BASE, and none of them touched a crate**: `git diff --stat 2bbecac9 623f093f5 -- rust/` is `rust/CLAUDE.md` and `rust/Cargo.toml` and nothing else, which the reproduced binary independently confirms. The expression spike's own commit is in that range and its code is **not** in this tree.

This is the first option from `2026-08-11-value-representation-design.md` to be built. That document ranks it first and this entry does not re-argue the ranking.

#### The hypothesis, named before it was measured

`Number` was `{ negative: bool, digits: Vec<u8>, exponent: i32 }` -- **one heap allocation per number, one `free` to match it, and a fresh pair for every clone.** `digits` becomes `Digits`, an enum with an inline array for the small case and a `Vec<u8>` for the rest. It derefs to `[u8]`, so every read stays the slice operation it was; only construction and the in-place edits go through methods.

**This is not a kernel change and must not be read as one.** Entry 11 measured this crate's decimal kernels at about **1.65x faster than the oracle's in absolute self time**. What this removes is the allocation and copying *around* the arithmetic.

**Blast radius: `rexx-num` alone**, as the design document predicted -- `digits` is `pub(crate)`, no interpreter constructor was added, and both engines reach arithmetic through `Interp::arith_small_int` and `Interp::arith_general`. Five files changed, 54 insertions and 34 deletions, plus one new 535-line module.

#### The inline capacity is 20, chosen from the language before anything was timed

Two language bounds, and it is above both. **`NUMERIC DIGITS` defaults to 9**: every operator truncates its operands to `working_length(digits)` -- ten -- and an addition can carry into an eleventh digit, so eleven is the widest intermediate the default produces. **`Number::from_i64` is the door every tagged small integer takes into this crate**, and an `i64` is at most nineteen decimal digits; that function's own `Vec::with_capacity(20)` already wrote the bound down.

**What bounds it above is layout and it was measured before the choice, not after.** `size_of::<Number>()` at BASE is **32**, `size_of::<Body>()` **80**, `size_of::<Slot>()` **96** (`gdb -batch -ex 'print sizeof(...)'` against the BASE binary -- two of them figures the design document lists as unsettled and could not run). A `size_of` probe over candidate capacities reads 24/32 bytes for `Digits`/`Number` at capacity 15 and below, 32/40 from 16 to 30, and 40/48 at 31 and above. `Body::Num` holds a `Number` beside a `u32`, a `Form` and an `Option<Vec<u8>>` -- 64 bytes against `Body::Stem`'s 72 -- so a `Number` may reach 40 bytes without `Body` moving and may not reach 48. **Measured at HEAD: `Number` 40, `Digits` 32, `Body` 80, `Slot` 96. No arena slot widened.**

**One disclosure the number depends on, and it flatters this axis.** `bench-programs/arith.rex` switches to `NUMERIC DIGITS 20` for three of its five arithmetic assignments, and a 20-digit *result* fits a 20-digit inline arm **exactly** -- though that setting's 21-digit working intermediates still spill. The capacity was fixed from `i64`'s nineteen digits and pre-registered before the first timed run, and a capacity of 15 -- equally defensible from the default `DIGITS` alone -- would have left that half of the axis on the heap. So the axis's own second precision sits exactly on the boundary by coincidence, and `arith`'s figure below should be read knowing that.

#### Predicted movement, written down before the first timed run

| axis | predicted | measured |
|---|---|---:|
| `arith` | **-10% to -25%** wall, central -17% | **-15.41%** |
| `rexxcps` | -1% to +3% clauses per second | **+4.78%** |
| `strings` | 0%, band -2% to +2% | no usable wall figure |
| `compound`, `varlookup`, `emptyloop` | 0%, band -1% to +1% instructions | +0.24%, +0.78%, +1.65% instructions |
| `alloc4c` | no claim -- entry 11 struck it off as an acceptance axis | +1.32%, sign held in 2 of 7 |
| peak resident set | **falls on `arith`**, unchanged elsewhere | `arith` -21.6%, elsewhere unresolved |
| `size_of::<Body>()` | **80, unchanged** | 80 |

**`rexxcps` was under-predicted and the reason is the same one entry 9 recorded against itself.** The prediction was made from "most of its allocation is strings and tail keys", which is a claim about a decomposition **entry 11 says nobody has taken**. It came in at +4.78% clauses per second on -12.25% instructions.

#### Build identity

| | |
|---|---|
| BASE `rexx-run` | size=13849736, sha256 `35999021f97f24bb81e8c7a65083340084a8e1107087e11caf34dc8bc7f82141` |
| HEAD `rexx-run` | size=14003488, sha256 `753a91d6ef43e085f886784e690cc91cca5d99ecb37325310e9dfd0efb9234f5`; the binary **grew** by 153,752 bytes |
| oracle | the same three objects entry 1 fingerprints, re-hashed and unchanged: `bb5bb8cc...`, `42136c40...`, `3536b763...` |

Both binaries were built in the same working tree from a `cargo clean` of the four crates, so neither carries a path difference the other does not.

#### The accept measurement: the two binaries alternating in one loop

The accept rule's literal shape, and **not** `rexx-bench-suite`. Wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, a fresh empty working directory per run, base/head order **rotated every round**, seven rounds per axis. All 84 runs exited 0 and each axis's stdout hash is identical across all fourteen runs of it.

**Gated the way entry 8's re-measurement asks**: six consecutive five-second samples of host idle read from `/proc/stat`, reading 96.4, 96.6, 96.1, 96.4, 96.4 and 96.2 per cent before the first run.

| axis | base median | head median | **change** | rounds with that sign |
|---|---:|---:|---:|---:|
| `arith` | 2.4866 s | 2.1034 s | **-15.41%** | **7 of 7** |
| `emptyloop` | 1.9621 | 1.9173 | -2.28% | 7 of 7 |
| `varlookup` | 2.5911 | 2.5469 | -1.71% | 7 of 7 |
| `compound` | 2.9139 | 2.8658 | -1.65% | 7 of 7 |
| `strings` | 5.3636 | 5.3834 | +0.37% | 3 of 7 |
| `alloc4c` | 1.5083 | 1.5282 | +1.32% | 2 of 7 |

Per-round on `arith`: -15.62, -15.23, -15.73, -16.16, -15.07, -14.73, -15.34 per cent. There is no round in which it is not about a seventh.

`rexxcps` is not read as wall time, for entry 1's reason. Both binaries self-calibrated to the identical `100 x 100`, printed and checked every round: five rounds, order rotated, base median **2,563,956** clauses per second against head **2,686,435** -- **+4.78%**, head ahead in **5 of 5**.

#### Instructions beside wall, which is what separates the work from the layout

`perf stat -e instructions:u`, **six runs per side**, arms alternating, medians, each arm's own full range printed beside it -- which is entry 10's rule and the reason entry 9's displacement column was withdrawn.

| axis | instructions base | instructions head | change | base range | head range | wall |
|---|---:|---:|---:|---:|---:|---:|
| `arith` | 23,930,039,426 | 20,236,357,832 | **-15.44%** | 0.000% | 0.000% | -15.41% |
| `rexxcps` | 36,957,184,703 | 32,430,510,414 | **-12.25%** | 0.067% | 0.026% | +4.78% cps |
| `emptyloop` | 28,775,608,587 | 29,250,586,704 | **+1.65%** | 0.000% | 0.000% | -2.28% |
| `varlookup` | 43,700,613,687 | 44,042,611,819 | **+0.78%** | 0.000% | 0.000% | -1.71% |
| `strings` | 63,168,499,636 | 62,859,742,493 | -0.49% | 0.000% | 0.000% | +0.37% |
| `compound` | 37,229,019,302 | 37,319,026,725 | +0.24% | 0.011% | 0.007% | -1.65% |

**`arith`'s wall and its instruction count agree to three hundredths of a point.** That is work removed, not layout, and it is the whole of the case for this candidate.

**Three axes moved in wall and moved the *other way* in instructions, and their wall gains are disclaimed rather than banked.** `emptyloop` -2.28%, `varlookup` -1.71% and `compound` -1.65% all held their sign in 7 of 7 rounds, and all three retire **more** instructions at head. That is a layout effect, and this entry does not claim it -- entries 4 and 7 disclaimed theirs on the same ground. Unlike entry 9's withdrawn column, these instruction movements **are** resolved: every arm's own full range is 0.011% or below, one to two orders of magnitude under the movements themselves.

**So the displacement channel is real, it is measured, and it is about one per cent.** `emptyloop` at +1.65% is the worst of it and is the axis with no `Number` in its hot clause at all.

#### Peak resident set, which is half the falsifier

`/usr/bin/time -f %M`, same wrapper, **five runs per side**, medians with every run printed.

| program | base | head | change |
|---|---:|---:|---:|
| `arith` | 10,676 KB (10,416 to 10,944) | **8,368 KB** (8,108 to 8,628) | **-21.6%** |
| `varlookup` | 2,476 KB (2,244 to 2,756) | 2,528 KB (2,516 to 2,816) | not resolved |
| `say 1` | 2,528 KB (2,516 to 2,532) | 2,472 KB (2,448 to 2,732) | not resolved |

**`arith`'s two arms do not overlap across five runs each**, so the fall is the instrument's, not a median's. **`varlookup`'s apparent rise is not**: a first pass at three runs per side read 2,408 against 2,588 and looked like a real 7% rise; at five runs each arm's own spread is about 500 KB and the two overlap, so the honest statement is that this instrument does not resolve it here. The one-clause program is the control for the 153,752 bytes the binary grew, and it does not resolve a rise either.

From a three-run pass on the remaining axes, recorded as unresolved for the same reason: `strings` 11,448 against 11,024, `compound` 2,516 against 2,472, `emptyloop` 2,464 against 2,448, `alloc4c` 180,240 against 179,976.

#### The mechanism, measured directly rather than inferred from the axis

Three 200,000-iteration probes, `perf stat -e instructions:u`, three runs each, medians, from a fresh empty directory, each printing the identical answer on both binaries. **The first two are the same clause at two precisions, one inside the inline arm and one outside it**, which is what isolates this change from anything else that moved with it.

| probe | what it is | base | head | change |
|---|---|---:|---:|---:|
| `numeric digits 9; a = i / 3` | a nine-digit result, inside the arm | 1,067,342,015 | 1,001,378,796 | **-6.18%** |
| `numeric digits 25; a = i / 3` | a twenty-five-digit result, outside it | 1,581,183,128 | 1,633,792,759 | **+3.33%** |
| `a = i + 1` | never builds a `Number` at all | 346,183,133 | 350,002,498 | +1.10% |

**The middle row is the cost this change carries and it is not netted out.** A program working above twenty digits pays the enum's tag and a wider `Number` to copy and still pays the allocation. The bottom row is the displacement floor, and it agrees with `varlookup` and `emptyloop` above.

**`arith`'s -15.44% is larger than the isolated `/` probe's -6.18%** because that axis runs five arithmetic assignments per pass -- two at `DIGITS 9` and three at `DIGITS 20` -- where the probe runs one.

#### What the design document asked this candidate to settle, and what it says

`2026-08-11-value-representation-design.md` sets a test: *"land B, measure `arith`, and compare the delta against those two rows"* -- `drop_glue::<rexx_num::Number>` at 12.2% of the axis and `Number::clone` at 7.7%, which entry 11 forbids adding.

**The measured move is -15.44% in instructions: larger than either row and smaller than their sum of 19.9.** By that document's own rule -- *"if the measured move is materially larger than either, the blocks were overlapping"* -- the two rows do overlap, and entry 11's residual arithmetic is inflated to that extent. It is **not** materially smaller than both, so the attribution was not wrong. This is one axis and one candidate; it is a data point for that question, not a verdict on it, and entry 11's 4.35x and 5.50x are not withdrawn here.

#### Correctness, which outranks the number

**A 93,632-case differential against the oracle, aimed at the capacity boundary, on both engines, at BASE and at HEAD.** 22 generated programs -- eleven `NUMERIC DIGITS` from 1 to 100 crossed with both `FORM`s -- each running 266 literals chosen to straddle twenty significant digits through 16 operations, plus 60 lines walking one value across the capacity in both directions inside a single computation. Every case reports the rendering, its length, a re-render through concatenation and `DATATYPE(v,'W')`, and traps its own syntax condition so one overflow does not truncate the rest.

* **All 198 descriptors are byte-identical between BASE and HEAD** -- 22 programs by three arms by stdout, stderr and exit status.
* **The IR and the tree-walker agree on every one of the 94,952 output lines**, at BASE and at HEAD.
* **6,236 lines diverge from the oracle, identically at BASE and at HEAD, and every one of them is the same pre-existing gap entry 9 recorded**: `DATATYPE(v,'W')` answers `0` where the oracle answers `1` for a whole number of nineteen digits or more, because this crate asks `Number::whole_value(ARGUMENT_DIGITS)` and that constant is 18. Strip that one field and the divergence count is **zero**. The remaining 88,716 lines match the oracle byte for byte on both engines.

**Under the collector's stress mode**, because a representation change alters what is a heap object: ten of the boundary programs -- the four `DIGITS` either side of the capacity and `DIGITS 9`, both forms -- through `run_program_collect_every_alloc`, **1,696,652 collections**, stdout, stderr and exit code identical to the plain run of each. The committed corpus stress gate (`the_l0_subset_passes_again_under_collect_on_every_allocation`) is green.

**The in-crate equivalence, which is where a representation change is actually held.** `every_operation_answers_the_same_with_the_digits_on_the_heap` runs the whole public surface twice over a boundary population crossed with nine precisions -- once with the digits inline and once with the identical value forced onto the heap -- and asserts the answers are equal, through `add`, `sub`, `mul`, all three `div` forms, `compare`, `format`, `format_form(Engineering)`, `round_to`, `plain_integer`, `rendered_integer`, `abs`, `signum`, and `pow` separately. Measured by printing them once: **196 values, 56 of them already on the heap arm, and 349,272 checks.** It carries two floors -- a case count, and that some case reached the heap arm at all -- so a population that stopped generating cases, or one that never left the inline arm, fails rather than passes empty.

**Two mutations, and the pair is the finding.**

* **Dropping the shift out of `insert_front`'s inline arm -- a wrong *answer* -- reddens six existing tests**, `the_small_int_fast_path_answers_what_the_general_path_answers`, `both_engines_agree_on_every_case_file` and both exempt-set harnesses among them. So the suite as it stood catches a wrong digit.
* **Dropping the zero-fill out of `extend_zeros`'s inline arm -- stale bytes surfacing as digits -- leaves the entire workspace green without the new tests: 1442 passed, 0 failed.** Only `the_in_place_edits_agree_with_a_vector_on_both_arms` catches it, and notably the 349,272-check equivalence test does not, because nothing in the arithmetic today calls `extend_zeros` on a buffer a `truncate` has shortened. **That is the representation-only hazard, and it is reachable by an ordinary future edit.**
  **One thing weakens that mutation and is recorded rather than left out:** deleting the fill leaves `buf` unread in that arm, so `rustc` emits an unused-variable warning and the `-D warnings` gate refuses *that* spelling -- run rather than inferred: with the mutation in place `cargo clippy --workspace --all-targets -- -D warnings` exits 101 on `error: unused variable: `buf``. What the warning cannot see is the same defect written so the buffer is still touched -- filling the wrong value, or filling the wrong range -- and that is what the test is for.
  Both runs used `--no-fail-fast` and read run counts; the file was restored from a `cp` copy and `sha256sum -c`'d after each, never `git checkout --`.

Gates: `cargo test --workspace` **1449 passed, 0 failed, 4 ignored** in dev and in release, run counts read rather than exit status alone. Inside the release run the corpus runner reported 10 passed and 1 ignored, the dual-engine sweep 9, the collector stress harness 6. **BASE was re-counted rather than inherited, by running the suite at BASE before anything was edited**: **1442 passed, 0 failed, 4 ignored** in both profiles. HEAD runs seven more, all in the new module. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean **from a full `cargo clean`**, with `Checking rexx-num`, `rexx-parse`, `rexx-core` and `rexx-exec` all confirmed in the log -- and it caught one lint in the new tests on the first, failing pass, so the run examined the code.

**No `unsafe`.** The workspace lint is untouched at `forbid`, and nothing in this change wanted one: the inline arm is an ordinary enum whose `len` is a plain field, so a bug in it is a wrong digit rather than undefined behaviour, which is exactly what the mutations above demonstrate.

#### Disposition: **accepted**

Entry 11's falsifier was *"a paired run of lever (a) moving `arith` by less than about 15%; or peak resident set not falling."* `arith` moved **-15.41% wall and -15.44% instructions**, and `arith`'s peak resident set fell **21.6%** with non-overlapping ranges over five runs a side.

**It clears the first half of that falsifier by four tenths of a point, and this entry does not pretend otherwise.** Had the capacity been 15 rather than 20 -- a choice equally defensible from `NUMERIC DIGITS 9` alone -- `arith.rex`'s `DIGITS 20` section would have stayed on the heap, and the number would probably have been under the bar. That last clause is a conjecture: no binary was built at 15 and none should be, because building one to see which capacity measures better is exactly what Unit 0 forbids. The capacity that saved it was fixed from `i64`'s width and written down before the first run, which is the only reason that sentence can be written at all.

**The hypothesis is confirmed by the route it named**, and the probe pair is the evidence: the same clause is 6.18% cheaper when its result fits the arm and 3.33% dearer when it does not.

Accepted **with three costs recorded and not netted out**: a program working above twenty digits retires about 3% more instructions; the displacement floor is about 1% on axes this does not touch, worst on `emptyloop` at +1.65%; and the binary grew 153,752 bytes.

**What this did not remove, named so the next candidate is not sold twice.** Entry 11 gave candidate 1 a ceiling of 39.9% on `arith` and this took 15.4 of it. Scratch buffers that never become a `Number`'s digits still allocate, and three were read out of the source rather than enumerated exhaustively: `mul_magnitudes`' `Vec<u16>` accumulator, `long_divide`'s working remainder, and `divide_power`'s dividend. Each is the reference's `resultBufFast` in a place it has not been applied, and none was touched here because touching them would have confounded the attribution above.

Commit: `4087e9d147e22f3251c13e6a8fd7855a0f01c7b4`, read back from `git log` after committing.

#### What this entry cannot say

* **It is one gated sitting on one Linux host**, unpinned, on a machine whose governor cannot be fixed. The rule that a handful of paired runs decides it is why there is not a second.
* **It did not run `rexx-bench-suite`, so it carries no oracle ratios.** Where `arith` now sits against the oracle is unmeasured; entry 11 had it at 2.1291x.
* **`strings` and `alloc4c` have no wall figure here at all.** Their signs hold in neither, which is what entry 11 predicted for `alloc4c` and what a -0.49% instruction move predicts for `strings`.
* **`rexxcps`' +4.78% is a clauses-per-second figure**, not a wall ratio, and it rests on the -12.25% instruction count beside it. That instructions fell four times as far as throughput rose is unexplained here.
* **The tree-walker was measured only for correctness.** Every wall, instruction and resident-set figure is the IR arm; `Number` sits below both engines, so the tree-walker gets the same work removed and no benchmark here says by how much.
* **The equivalence is asserted over a generated population, not proved.** A shape nobody thought of is a shape the floors cannot miss.
* **Nothing here re-profiles.** Unit 0's cadence asks for a re-profile of `arith` and `rexxcps` before the next candidate is chosen, and this entry does not supply it.

---

### Entry 13 -- the `smallvec` citation finally measured, and it says the option was already taken

**No code change beyond three assertions, and no candidate.** Raised 2026-08-11 by Moritz after entry 12 landed: `smallvec` with its `union` and `const_generics` features is a different type from plain `smallvec`, and a measurement without them measures the wrong thing.

**Two premises corrected first, because the rest turns on them.**

**There was no `smallvec` measurement to re-take.** Entry 12 did not evaluate `smallvec`, inconclusively or otherwise; it hand-rolled a safe enum and accepted it. So this entry is the first time the crate has been measured in this repository at all -- which is what the record has been asking for since entry 2 cited a result that never existed.

**"`Number` grows, so `Body` may grow, so every slot grows" did not happen, and entry 12 measured it rather than reasoning about it.** `Number` went 32 bytes to 40 while `Body` stayed at **80** and `Slot` at **96**, because `Body::Num`'s payload had eight bytes of headroom against `Body::Stem`'s, which is the variant that sets the width. The design document's sentence that option B does not move `Body`'s width is therefore true as written, and it is true of the plain enum -- **`union` is not what makes it true.**

#### What `smallvec` actually measures, both features on

A scratch crate against the offline cache, `smallvec = { version = "1.15.2", features = ["union", "const_generics"] }`, on the workspace toolchain **rustc 1.97.1**. Both features compile without complaint, so neither has an MSRV problem here -- checked rather than assumed.

| inline capacity | `size_of::<SmallVec<[u8; N]>>` | the `Number` it yields | entry 12's hand-rolled enum |
|---:|---:|---:|---:|
| 7 to 15 | 24 | 32 | **24 / 32** |
| 16 | **24** | **32** | 32 / 40 |
| 17 to 24 | 32 | **40** | 32 / **40** |
| 30 to 31 | 40 | 48 | 40 / 48 |

**`SmallVec<[u8; 22]>` with `union` is 32 bytes, not 24.** The mechanism is in the crate's own source: `SmallVec<A>` is `{ capacity: usize, data: SmallVecData<A> }`, and with `union` that data is `union { inline: ManuallyDrop<MaybeUninit<A>>, heap: (NonNull<Item>, usize) }`. The union is `max(22, 16)` rounded to align 8 -- **24** -- and the `usize` sits beside it, not inside it. 24 bytes overall is reached at `[u8; 16]` and below.

**Three consequences, and the first two decide it.**

* **At the capacity the language asks for, `smallvec` yields exactly the `Number` entry 12 already ships: 40 bytes.** `Number::from_i64` is the door every tagged small integer takes into this crate and an `i64` is nineteen decimal digits, so the capacity has to be at least 19; every capacity from 17 to 24 gives 32/40. Switching would buy **zero bytes** in `Number`, in `Body`, or in the arena.
* **The 24-byte target and the capacity target are mutually exclusive.** 24 bytes caps the inline arm at sixteen digits, three short of an `i64`, so `from_i64` would allocate for large integers -- and it would still not shrink `Body` or `Slot`, both of which are already unmoved.
* **`union` buys exactly one capacity value over the plain enum at the 24-byte tier: sixteen rather than fifteen.** rustc niche-fills the enum using the `Vec` pointer's non-null niche, which is why the plain enum is *also* 24 bytes up to fifteen. The claim that a non-`union` `SmallVec` is necessarily wider than the `Vec<u8>` it replaces is not true of the enum shape below the niche's capacity; it becomes true above it.

**So the framing "audited dependency versus our first `unsafe` module" has a third term, and it is the one in the tree.** The overlapping layout is what would need the unsafety, and at `Number` = 40 it buys nothing to be unsafe *for*. Entry 12 shipped safe Rust with the workspace lint untouched at `forbid`, and `Cargo.lock` still does not contain `smallvec` -- which independently confirms, a third time, that entry 2's citation had no dependency behind it.

**Recommendation: do not take the dependency for this.** It is not withdrawn as an idea -- if a later capacity argument lands at sixteen digits or fewer, `union` buys eight bytes of `Number` and the crate is in the offline cache. Nothing at nineteen digits or above does.

#### The three assertions, and each was shown to fail

Moritz's other ask, and it is right on its own merits: a layout claim defended by a sentence has rotted in this repository, and `const _: () = assert!(...)` cannot.

`size_of::<Number>() <= 40` in `rexx-num`, `size_of::<Body>() <= 80` and `size_of::<Slot>() <= 96` in `rexx-core`. **Upper bounds rather than equalities**, because the claim is that nothing widened; shrinking is free and needs no decision. **Each was tightened by one byte on its own and shown to stop the build** -- `evaluation panicked: assertion failed` for all three, `Body`'s and `Slot`'s tested separately because `rexx-num` failing first would otherwise have hidden them.

**`Body`'s bound is expected to trip in Phase 5** and that is the point rather than a problem: the design document's own rule is that hot variants stay inline and narrow while a new class's instance state arrives boxed, and a widening should be a deliberate act with the boxed alternative weighed.

**Compile-time only, checked rather than assumed.** The release binary's `.text` section is **byte-identical** to the binary entry 12 measured -- 987,817 bytes, sha256 `1d9d9996f6dab700a5e62c606f511ba2450155218dcb648c3b5f00507aa8e491` -- so entry 12's wall, instruction and resident-set figures still describe this tree. The whole-file hash moves, because `debug = true` embeds line tables that shifted; that is why the section and not the file is the instrument here.

Gates: `cargo test --workspace` **1449 passed, 0 failed, 4 ignored** in dev and release, unchanged from entry 12. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean from a full `cargo clean`, all four crates re-checked. The 22-program boundary differential re-run against this binary is byte-identical to entry 12's.

Commit: `e4bf5f2cbda8364207c753c4318d6c1c95727c86`, read back from `git log` after committing.

#### What this entry cannot say

* **It measures widths, not speed.** Whether `smallvec`'s spill and growth code is faster or slower than the hand-rolled arm's is unmeasured, and at equal widths it would be a separate candidate with its own paired run rather than a reason to switch.
* **It is one toolchain.** rustc 1.97.1 niche-fills the plain enum to 24 bytes up to fifteen digits; a toolchain that stopped doing so would move the left column and not the `smallvec` one. The assertions above are what would catch that.
* **It does not revisit entry 2's citation as history.** What the citation claimed to have measured is still unknown; what is now known is that the type it names does not have the width the argument for it needs.

---

### Entry 14 -- the string-length histogram, which the design document asks for before A, E or F: it picks A, and it says A cannot touch three of the six axes

**A measurement, not a candidate.** Nothing was built to keep and nothing landed.
`2026-08-11-value-representation-design.md` puts this between B and A because *"it is not a candidate at all -- it is one instrumented run, it decides A against E, and getting it wrong means building the larger of the two twice."*
**No timed comparison was taken and none is reported**, so the host-idle gate the accept rule carries does not apply here and was not observed.

**BASE is `68e1c4681b92a12e44d9790285fd42457af537cb`.**
Its `rexx-run` is size 14003504, sha256 `cc0315ac4307c341cf0308b7705d9e127d002e199b0804bd15edcce7b128bb8d`, built from `cargo clean -p rexx-core -p rexx-exec --release`.
The three instrumented files were restored from `cp -a` copies and `sha256sum -c`'d, never `git checkout --`, and the rebuild that followed the restore reproduces that hash **byte for byte**.

#### What was counted, and where

**One counter in `Heap::alloc_with_uncollected`**, which is the single funnel every arena allocation takes, so the population is every `Body::Text` this crate creates rather than every one that happens to pass through `Interp::text_owned`.
Exact buckets for each length below 65536, plus an overflow counter and a running maximum, plus one counter per `Body` variant so that a `Text` count of zero has a denominator.
Dumped from `rexx-run`'s `main` when `REXX_TEXT_HIST` names a file.
**Counting, not sampling**, which is entry 11's warning applied: this question is about how many `malloc` calls disappear, and a length that occurs once has to be as visible as one that occurs a million times.
No `unsafe`; the counters are relaxed atomics and the workspace lint was untouched.

**Four controls, each run rather than asserted.**

* `say 'hello'` reports exactly one creation at length 5.
* `do i = 1 to 5 ; s = copies('a', i) ; end` reports ten: six at length 1 (the `'a'` literal once per pass, plus `copies('a',1)`), and one each at 2, 3, 4 and 5.
* `x = 0 ; do i = 1 to 100 ; x = x + 1 ; end` reports **zero allocations of every variant**, so an axis reading zero below is reading a real zero and not an instrument floor.
* With `REXX_TEXT_HIST` unset, no file is written at all.

**And the instrument changes no behaviour, which was measured rather than argued.**
All 70 corpus programs were run on the instrumented binary and on BASE's: **210 descriptors -- exit status, stdout hash and stderr hash for each -- and `diff` is empty.**

#### The distribution, by creation count

IR arm, `ulimit -v 8388608`, a fresh empty working directory per run, stdin closed.
Axes are `rust/bench-programs/*.rex` plus `/home/moritz/dev/repos/ooRexx/samples/rexxcps.rex`, which self-calibrated to the same `100 x 100` entry 12 recorded.
The corpus is all 70 `.rex` files under `rust/corpus`, each its own process, pooled.
Percentiles are nearest rank: the smallest length whose cumulative count reaches the rank.

| population | `Body::Text` created | <=15 | <=23 | <=31 | median | p90 | p99 | max | mean |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `strings` | 18,000,001 | 66.67% | 66.67% | 66.67% | 3 | 46 | 46 | 46 | 16.83 |
| `rexxcps` | 11,910,838 | 84.72% | 94.12% | 95.30% | 3 | 21 | 33 | 66 | 6.44 |
| `compound` | **0** | -- | -- | -- | -- | -- | -- | -- | -- |
| `varlookup` | **0** | -- | -- | -- | -- | -- | -- | -- | -- |
| `arith` | **0** | -- | -- | -- | -- | -- | -- | -- | -- |
| `alloc4c` | 2,000,000 | 100.00% | 100.00% | 100.00% | 4 | 10 | 10 | 11 | 6.94 |
| **axes pooled** | 31,910,839 | 75.49% | 79.00% | 79.44% | 3 | 43 | 46 | 66 | 12.34 |
| **corpus (70)** | 2,425 | 84.78% | 93.11% | 96.78% | 5 | 19 | 46 | 251 | 8.17 |

**Three of the six axes create no `Body::Text` at all, and that is the largest single finding here.**
It is not that they create few; it is zero, on runs of five million, nineteen million and five hundred thousand iterations.

#### What those three axes allocate instead, which is why the zeros are readable

| axis | `Text` | `Num` | `Stem` | total arena allocations |
|---|---:|---:|---:|---:|
| `strings` | 18,000,001 | 0 | 0 | 18,000,001 |
| `rexxcps` | 11,910,838 | 830,205 | 140,001 | 12,881,044 |
| `arith` | 0 | 3,469,314 | 0 | 3,469,314 |
| `compound` | 0 | 0 | 1 | **1** |
| `varlookup` | 0 | 0 | 0 | **0** |
| `alloc4c` | 2,000,000 | 0 | 1 | 2,000,001 |
| corpus | 2,425 | 111 | 16 | 2,552 |

`Array`, `Instance` and `WeakRef` are zero in every population.

* **`compound` allocates one heap object in a five-million-iteration run**, a single `Stem`, and nothing else ever.
  So entry 11's *"on `compound` the allocator family is 12.1%"* is **not** arena traffic: it is the `Vec<u8>` tail keys and the `HashMap` growth inside that one `Stem`, neither of which is a `Body` and neither of which A, E or F touches.
* **`varlookup` allocates nothing at all**, which the third control predicted and which makes it useless as an acceptance axis for any of the three.
* **`arith` is `Num` and only `Num`**, which is entry 12 aimed exactly right and A aimed at nothing.

**So the design document's "A third, aimed at `rexxcps` and `compound`" is half wrong**, and the half that is wrong is the axis whose allocator share the document quotes for it.
A's reachable axes are `rexxcps`, `strings` and `alloc4c`, and the document forbids selling it on `strings`.

#### `size_of::<Body>()` and `size_of::<Slot>()` at every capacity considered

A scratch crate outside the workspace, replicating `Body`, `Object` and `Slot` field for field, because `Slot` is private to `rexx-core::heap`.
**The replica carrying a plain `Vec<u8>` is the control and it reads 80 and 96**, which is exactly what `body.rs` and `heap.rs` assert today, so the other rows are measuring the thing they claim to.
The inline shape measured is `enum { Inline { len: u8, buf: [u8; CAP] }, Heap(Vec<u8>) }` -- entry 12's shape, in safe Rust.

| `Body::Text`'s payload | its width | `size_of::<Body>()` | `size_of::<Slot>()` |
|---|---:|---:|---:|
| `Vec<u8>`, today | 24 | 80 | 96 |
| inline capacity 7, 15 | 24 | 80 | 96 |
| inline capacity 16, 20, 22, 23, 24, 30 | 32 | 80 | 96 |
| inline capacity 31, 32 | 40 | 80 | 96 |
| inline capacity 39, 46 | 48 | 80 | 96 |
| inline capacity 47 to 54 | 56 | 80 | 96 |
| **inline capacity 55** | 64 | **88** | **104** |
| `compact_bytes::CompactBytes`, 23 inline | 24 | 80 | 96 |
| `compact_bytes::CompactBytesSlice`, 15 inline | 16 | 80 | 96 |

**Capacity 54 is the largest that costs nothing, and 55 is the first that widens both by eight.**
The boundary was found by measuring every capacity from 46 to 55 rather than by rounding a rule.
The reason is entry 13's: `Body::Stem` sets the width, `Body::Text` has headroom against it, and here that headroom is far larger than the 24-byte figure the brief for this measurement started from.

#### What the data picks

**Option A, hand-rolled, with the capacity taken from the layout ceiling and not from any axis.**

* **E is declined, and the histogram is what declines it.**
  E's entire advantage over A is the strings A does not reach.
  At capacity 54 that population is **7 creations out of 11,910,838 on `rexxcps`, none at all on `strings` or `alloc4c`, and 20 out of 2,425 in the corpus.**
  A side byte-arena would add compaction to the sweep for that.
* **F is declined for the same reason, and this is the measurement F's own bar asked for.**
  The design document is explicit that F's only advantage over A is one cache miss on a string above the inline bound.
  That is the same seven-in-twelve-million population.
* **A dependency is not warranted, and this is entry 13's shape a second time.**
  `compact_bytes`' selling point is 23 inline bytes in the width of a `Vec<u8>`, reached through a `union` and `unsafe`.
  Measured, 24 bytes is not the budget: **56 is**, and a safe hand-rolled enum at that width offers capacity 54 against `CompactBytes`' 23.
  The crate packs tightly against a constraint that does not bind, exactly as `smallvec` did for `Number`.
  Its fixed variant, `CompactBytesSlice`, is the right *shape* for an immutable Rexx string and offers 15; its growable one offers 23; neither reaches 54, and `Body` is 80 with either.

**On the capacity, and the coincidence that has to be disclosed before anyone reads the coverage figures.**
`strings.rex`'s pangram is 43 bytes and its concatenation is 46, so **a capacity anywhere in 46 to 54 inlines 100.00% of that axis and a capacity of 31 inlines 66.67% of it.**
That is entry 12's `arith.rex`-at-`DIGITS 20` hazard in a new place, and the defence has to be the same one: **the capacity must be fixed from the layout before the axis is consulted.**
The layout says 54, measured above; 54 is above 46 rather than at it, and the corpus -- an independent population -- puts 99.175% at or below 54 with 20 creations above it.
**A capacity chosen at 46 because that is where `strings.rex` stops would be fitting the representation to the benchmark and should be refused even though it measures the same.**

**What the choice is actually between**, since every capacity to 54 is free in the arena:

| capacity | axes pooled | `rexxcps` | corpus |
|---:|---:|---:|---:|
| 15 | 75.49% | 84.72% | 84.78% |
| 23 | 79.00% | 94.12% | 93.11% |
| 31 | 79.44% | 95.30% | 96.78% |
| 54 | 7 creations short of all | 7 short of all | 99.175% |

The step from 31 to 54 is worth **6,560,005 of the 31,910,839 axis creations** and costs nothing in `Body` or `Slot`.
Its only cost is that constructing a short string writes a 56-byte payload rather than a 24-byte one, which is stores against a `malloc`, and it is unmeasured.

#### Do the axes and the corpus disagree

**They disagree about capacity and agree about the option.**
Below 46 the corpus is consistently more inlinable than the axes -- 84.78% against 75.49% at 15, 93.11% against 79.00% at 23, 96.78% against 79.44% at 31 -- and the whole of that gap is `strings.rex` putting a third of the pooled axis population at 43 and 46 bytes.
At capacity 54 they converge and both are within a rounding error of complete.
**Neither population contains a long-string population worth an option of its own**, which is the answer to the design document's *"whether long strings are a population worth an `unsafe` site at all"*: measured, no.

**The corpus's tail is longer than any axis's and it is not what it looks like.**
Its maximum is 251 bytes, in `lang/address_env.rex`, whose own header says it exercises *"the 250-byte name limit"* -- a deliberate boundary probe, not a naturally occurring value.
The six corpus programs whose longest creation exceeds 60 bytes are `address_env.rex` (251), `condition_traps.rex` (103), `source_arg.rex` (88), `state_builtins.rex` (79), `parse_sources.rex` (77) and `digits_rounding.rex` (62).

#### What this entry cannot say

* **It counts surviving payloads, not `malloc` calls, and for `concat` those differ by a factor of two.**
  `Interp::concat` (`eval.rs:844-847` at BASE) builds `bytes`, then calls `self.text(&bytes)`, which is `text_owned(bytes.to_vec())` -- so one `Body::Text` from a concatenation is **two** payload allocations today, and this instrument sees one of them.
  A inlines the survivor and leaves the working buffer, so on `strings`, where all 3,000,000 of the 46-byte creations come from `||`, **A removes one allocation of two rather than the allocation.**
  That is D's territory, and this measurement makes D(ii) look larger relative to A on that axis than the design document's ranking implies.
* **It says nothing about speed.** Which capacity is faster, and whether A moves any axis at all, is a paired run this entry did not take and must not be inferred from a coverage percentage.
* **It is the IR arm only.** `Heap` sits below both engines so the population is the same by construction, and that was not checked here.
* **`rexxcps`' counts are from a self-calibrated run under instrumentation**, which reported 2,603,891 clauses per second against entry 12's uninstrumented 2,686,435 -- about 3% slower, same `100 x 100`. The shape is what is used above, not the count.
* **A benchmark axis is not a workload.** Six programs and a differential corpus written to be deterministic are what exists; neither is a sample of real Rexx, and the corpus in particular is built to probe boundaries rather than to be representative.
* **It does not re-profile.** Every share this entry compares itself against is entry 11's, taken at `2bbecac9` before both the expression spike and entry 12.

**No work commit, because nothing landed.**
Every other entry names the commit its change went in as; this one has none to name, and the instrumented build exists only in the paragraph above describing how it was reverted.
The scratch crate that measured the widths lives outside the repository and `Cargo.lock` is untouched, which is checkable: `compact_bytes` does not appear in it.

---

### Entry 15 -- `compact_bytes` is mature, the narrow question is answered yes, and the band the question is asked in has a fourth member

**No new measurement and no code.** Everything below is entry 14's data read against a question put to it afterwards, plus one fact from outside this repository.

**The maturity caveat is withdrawn, on Moritz's say-so, 2026-08-12.**
Entry 14 evaluated `compact_bytes` 0.2.1 rather than assuming it, and the version number raised a fair question about putting a young crate under every string value.
**Moritz answers it: `compact_bytes` is used in Materialize, a production database, and he considers it mature.**
So it is not an unproven dependency and must not be weighed as one.

**Entry 14's decline does not rest on maturity and never did, but one clause of it reads as though it might.**
That entry says the crate's inline arm is *"reached through a `union` and `unsafe`"*.
That is a true description of the mechanism and it was **not** a reason; it is struck here so no later reader can mistake it for one.
The reason was, and remains, the one below.

#### The narrow question: does the distribution need more than fifteen inline bytes

**Yes, and on the axis A is aimed at it needs more than twenty-three too.**
Allocations inlined, by count, since the cost being removed is one `malloc` per string:

| population | created | inlined at 15 | inlined at 23 | inlined at 54 | 15 to 23 buys | 23 to 54 buys |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 18,000,001 | 12,000,000 | 12,000,000 | 18,000,001 | **0** | 6,000,001 |
| `rexxcps` | 11,910,838 | 10,090,815 | 11,210,825 | 11,910,831 | 1,120,010 | 700,006 |
| `alloc4c` | 2,000,000 | 2,000,000 | 2,000,000 | 2,000,000 | 0 | 0 |
| axes pooled | 31,910,839 | 24,090,815 | 25,210,825 | 31,910,832 | 1,120,010 | 6,700,007 |
| corpus | 2,425 | 2,056 | 2,258 | 2,405 | 202 | 147 |

* **Fifteen is not enough**: it leaves 7,820,024 of 31,910,839 axis creations and 369 of 2,425 corpus creations on the heap.
* **`CompactBytes`' twenty-three is a real gain on `rexxcps` and the corpus** -- 1,120,010 allocations and 202 -- and **exactly zero on `strings`**, whose population has nothing at all between 3 bytes and 43.
* **Twenty-three is dominated on every population by a capacity that is also free.**

#### The band the question was asked in has a fourth member, and it is the one that decides

The question was framed as three bands: free below sixteen, `compact_bytes` alone from sixteen to twenty-three, and above twenty-three an inline arm widens `Body::Text` and has to be checked against `Body::Stem` before anyone calls it free.
**Entry 14 ran that check, and the fourth band is twenty-four to fifty-four, where a plain safe enum widens `Body::Text` and `Body` does not move.**

| shape | `Bytes` width | `size_of::<Body>()` | `size_of::<Slot>()` |
|---|---:|---:|---:|
| `Vec<u8>`, today | 24 | 80 | 96 |
| `CompactBytes`, 23 inline | 24 | 80 | 96 |
| plain enum, capacity 15 | 24 | 80 | 96 |
| plain enum, capacity 23 | 32 | 80 | 96 |
| plain enum, capacity 54 | 56 | 80 | 96 |
| plain enum, capacity 55 | 64 | **88** | **104** |

**No per-variant payload figure appears in that table on purpose, and the reason is worth recording.**
This entry's probe measures a variant's fields as a standalone struct, and that is **not** the quantity the record means by *"`Body::Stem`'s 72-byte payload"*, which is the contribution inside the enum after rustc packs the discriminant into padding.
Measured here: `ObjRef` is 8 bytes with **no niche**, so `Option<ObjRef>` is 16, and a standalone `Stem` struct is 80 where the record's in-enum figure is 72.
The two conventions differ by variant and cannot be compared, so only `Bytes`, `Body` and `Slot` -- all measured the same way, against a `Vec<u8>` control that reproduces `body.rs`'s and `heap.rs`'s own assertions -- are quoted.

**"Stays inside the current width" is true of `compact_bytes`, and it is a property of a width that costs nothing.**
`Body::Text`'s payload is not what the arena pays for; `Body` and `Slot` are.
So the premise separating band two from band three -- that widening `Body::Text` is the thing to avoid -- is the wrong invariant, and it is the whole of the disagreement.

#### What this leaves

**The recommendation is unchanged and the reason is now narrower: `CompactBytes` gives capacity 23 at zero cost, and a plain safe enum gives capacity 54 at the same zero cost.**
It is dominated, not rejected.
Nothing about the crate's quality bears on that, which is why the maturity answer changes no number here.

**Where `compact_bytes` would win, stated so this is not read as a general verdict on it.**
If `Body::Stem` were boxed, or if Phase 5 added a variant that made `Body::Text` the width-setting variant, the 24-byte tier would start to bind and 23-in-24 would be the best shape available.
Neither is true today, and the assertions `body.rs` and `heap.rs` already carry are what would announce it.
The crate is in the offline cache and this entry does not withdraw it as an idea; it records that the constraint it is good at does not currently exist.

---

### Entry 16 -- the design's option A attempted and **accepted with its falsifier tripped**: a short string's bytes held in the slot

**BASE is `4fa6c3e4d`. HEAD is `ce048e041`.**
The HEAD binary this entry's figures were taken on is byte-identical to the one `cargo build --release` produces from the committed tree, checked with `cmp` after the measurement rather than assumed.

This is the second option from `2026-08-11-value-representation-design.md` to be built, and the one entry 14's histogram selected.

#### The hypothesis, named before it was measured

`Body::Text` carried a `Vec<u8>`, so **every string was a second allocation reached through a pointer**, with a `free` to match it and a fresh pair for every clone.
`bytes` becomes a `Bytes`: a two-armed enum whose inline arm holds the payload in the slot itself and whose other arm is the `Vec<u8>` that was always there.
This is what `RexxString`'s trailing `char stringData[4]` buys the C++ interpreter.

#### The capacity is 54, and it is the layout's number rather than the benchmark's

Entry 14 disclosed the trap in advance: `strings.rex`'s pangram is 43 bytes and its concatenation 46, **so any capacity from 46 to 54 inlines 100% of that axis**, and a 54 chosen because the benchmark stops below it would measure exactly the same as a 46 chosen the same way.

54 is instead the largest capacity at which `size_of::<Body>() <= 80` still holds, and that is **checked rather than stated**.
Built at 55, three lines fail to compile: `body.rs`'s `size_of::<Body>() <= 80`, `bytes.rs`'s `size_of::<Bytes>() <= 56` and `heap.rs`'s `size_of::<Slot>() <= 96`.
Measured at HEAD: `Bytes` 56, `Body` 80. No arena slot widened.

#### Measured movement

Wall clock in the accept rule's literal shape -- `ulimit -v 8388608`, `REXX_ENGINE=ir`, a fresh empty working directory per run, base/head order rotated every round, seven rounds an axis.
**All 84 runs exited 0 and each axis's stdout hash is identical across all fourteen runs of it.**
Instruction counts are from `rexx-arms`, five rounds, IR arm.

| axis | wall | sign held | instructions |
|---|---|---|---|
| `strings` | **-8.26%** | 7 of 7 | -2.95% |
| `alloc4c` | **-11.89%** | 7 of 7 | -5.71% |
| `arith` | +4.39% | 0 of 7 | +0.03% |
| `emptyloop` | +5.91% | 0 of 7 | **+0.000%** |
| `varlookup` | +2.29% | 0 of 7 | +0.09% |
| `compound` | +1.72% | 0 of 7 | +0.02% |

`strings`, `alloc4c`, `arith`, `emptyloop` and `varlookup` have disjoint base and head ranges; `compound`'s overlap.
The six-axis geometric mean is **-1.2%**, offered as a summary and not as a gate -- no rule in this phase is stated on it.

**Peak resident set on `strings` fell from 11476 kB to 8624 kB**, three interleaved pairs, ranges disjoint at 11200..11652 against 8352..8676.

#### The falsifier, and it is tripped

The falsifier was *"`strings` moving less than about 10%, or peak resident set not falling, or `size_of::<Body>()` moving off 80."*
Resident set fell by a quarter and `Body` is still 80.
**`strings` moved -8.26% on wall and -2.95% on instructions, and both are short of about 10%.**

**The axis that moved most is not the axis the hypothesis named.**
`alloc4c` moved half again as far as `strings` on wall and nearly twice as far on instructions.
Per this file's own contract, a change reaching its target by a route other than its stated hypothesis has not confirmed it: **the mechanism is confirmed and the magnitude and the ranking are not.**

#### The four regressions are not layout, and not the width either

The obvious reading -- four axes moving in wall while their instruction counts stand still is a layout effect, which entries 4, 7 and 12 disclaimed on that ground -- **was tested and is wrong.**

* **Not layout.** Three semantically identical HEAD builds, differing only in a never-entered padding function and spanning 11.5 kB of `.text`, read +6.23%, +6.40% and +6.59% of cycles on `emptyloop`. A 0.36-point spread against a 6.23-point gap.
* **Not the width.** Built at capacity 6, where `Bytes` is 24 bytes and so **exactly the `Vec<u8>` it replaced**, `emptyloop` still costs +6.33%.
* **Not the frontend, the branch predictor, the instruction cache or the data cache.** At head `emptyloop` retires the same instructions to 0.000%, and takes 320M more cycles while issuing 51.7M *fewer* d-cache loads, 615 fewer d-cache misses, 2201 fewer branch misses and 1903 fewer icache misses. Frontend stalls rise 39% but are 8.4M cycles of a 320M-cycle gap.

**Every counter available here is flat or better at head, and the cycles are worse.**
The cost is structural -- it follows the enum in `Body::Text` and not its size -- and **it is unexplained and left open**, not disclaimed.

#### What the tests are worth, which took two mutations to establish

The first mutation was **non-discriminating and proved nothing**: an off-by-one in the inline read (`&buf[..len]` to `&buf[..len.saturating_sub(1)]`) fails 378 pre-existing tests, so a red suite under it says nothing about whether the new tests carry their own weight.
It also **allocated without bound and was OOM-killed at an anonymous resident set of 111777896 kB**, taking the session with it; the re-run was capped, and `CLAUDE.md` now carries `memcap` in Gates for that reason.

The discriminating mutation is representation-only: `<=` to `<` at both constructors, so the boundary length stops inlining and reads back identical bytes.
**Without the new tests it passes 1449 and fails 0 -- the whole pre-existing suite is blind to it. With them exactly three fail, and all three are new.**

Suite at HEAD: **1455 passed, 0 failed** in both the dev and release profiles, against 1449 at BASE.
Eight generated programs were run on the oracle, the IR arm and the tree-walker arm at both BASE and HEAD: **all 72 descriptors are identical between BASE and HEAD**, and the two arms agree everywhere. One oracle divergence on `b04_numeric` is present at BASE too and is not this change's.

#### Disposition: **accepted**, on Moritz's decision, with the falsifier recorded as tripped

The case accepted is not a wall-clock case.
It is the quarter of the resident set, the work genuinely removed on the two string axes, and that option A is the prerequisite the design document names for D(ii), whose accessor borrows from the payload type this changes.

**What this entry does not claim:**

* **Not a wall-clock win overall.** Four of six axes are reproducibly slower and the six-axis mean is negative by about a point.
* **No oracle ratios.** `rexx-bench-suite` was not run, so where any axis now sits against the oracle is unmeasured.
* **The tree-walker was measured for correctness only.** Every wall, instruction and resident-set figure above is the IR arm.
* **`rexxcps` was not measured here at all**, and entry 14's spill count of 7 creations in 11,910,838 on that axis is the only thing said about it.

**The open item this leaves is the largest one in the queue**: something about an enum in `Body::Text` costs the IR interpreter about 6% on axes that create no strings, at constant instruction count and with every cache and predictor counter improving.
Until that is understood, every later candidate measured against this HEAD inherits it.
