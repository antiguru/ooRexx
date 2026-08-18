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

---

### Entry 17 -- entry 16's four regressions **withdrawn**: a control with no semantics moves the same axes as far

**No new code and no change of disposition.** Option A stays accepted at `ce048e041`. What is withdrawn is entry 16's attribution of the four non-string regressions to the change.

#### The control that settles it

`Repr` collapsed to its single `Heap(Vec<u8>)` arm: no inline storage, no discriminant, the same 24-byte layout as the `Vec<u8>` it replaced, the same API and the same call sites.
**By construction it does what BASE did.**
Its stdout is byte-identical to BASE's and HEAD's on every axis below.

Measured by a subagent and then **reproduced independently on a different harness**, which is why it is here rather than in a scratch file:

| axis | the change | null control, subagent | null control, reproduced |
|---|---|---|---|
| `emptyloop` | +6.58% | +2.37% | +3.07% |
| `arith` | +4.45% | -0.49% | +0.78% |
| `varlookup` | +2.46% | **+6.98%** | **+6.78%** |
| `compound` | +1.77% | +1.60% | +1.53% |

Instruction counts across all three builds spread by **0.0001% to 0.0863%** -- every build does the same work.

**On `varlookup` the null control is worse than the change under investigation.**
A change with no semantics has no business producing that number, and that it does is the finding.

#### What this withdraws, and what it leaves standing

**Withdrawn:** entry 16's *"the cost is structural -- it follows the enum in `Body::Text` and not its size -- and it is unexplained and left open"*, and its framing of the four regressions as the largest open item in the queue.
The cost does not follow the enum. It follows **touching `Body` at all**.
`ce048e041`'s commit message carries the withdrawn sentence and cannot be edited; this entry is the correction of record.

**Not withdrawn:** entry 16's eliminations, every one of which survives -- not code layout, not the type's width, not the frontend, branch predictor, icache or dcache.
They were true and they were the wrong controls: all three vary something *about* the change, and none of them asks what a change of the same shape and no semantics is worth.

**Also not withdrawn:** the two string axes. `strings` -8.26% wall / -2.95% instructions and `alloc4c` -11.89% / -5.71% are instruction-backed on both, so they are work removed and not allocation. The resident-set fall of a quarter stands.

#### Why the instrument fails here

`Interp::step` is 8005 instructions and `run_loop_with_header` 2814, and under `lto = "fat"` with `codegen-units = 1` their register allocation is whole-program and sits at the edge.
The measured signature is a **1:1 opcode swap, not added work**: base's `mov %r14,%rdi` ten times becomes head's `mov <slot>(%rsp),%rbx` nine times -- a register move replaced by a stack reload, which is exactly why the instruction count does not move while the cycles do.
Of the nine functions carrying `emptyloop`'s self time, **six are byte-for-byte identical** between the binaries; `loop_advance::{closure#2}` is 20% of runtime, unchanged instruction for instruction, and costs 27% more.

The proximate stall on `emptyloop` is **store-to-load forwarding** (`r0224`, the only instrument that sees this effect -- it has no instruction-count and no cache signature): +23.79M forwarding failures against a +343.7M cycle gap, **14.4 cycles each, the whole of it**, with forwarding *hits* flat and per-run ranges disjoint.
The site is `run.rs:6088`, the `?` on a 24-byte `Result<Flow, Failure>` built by three 8-byte stores and re-read as a 16-byte load spanning two of them, which cannot forward.
**BASE already pays about 243M of these per `emptyloop` run**, roughly 9.7 an iteration; HEAD moved one more load onto the wrong side of a cliff the interpreter was already standing on.
The proximate stall on `arith`, `varlookup` and `compound` is **not** store forwarding and was not identified. That is left unguessed.

#### The consequence for this phase's accept rule, which is the real output

**These four axes cannot resolve a change that touches `Body`.** The resolution floor for such a change is about seven points, established by a control that changes nothing.

**Every future candidate touching `Body` carries a null control of the same shape, or its non-target axes are not read.**
A layout perturbation is not that control and neither is a width sweep -- entry 16 ran both and they agreed with each other and with the wrong answer.

#### Two candidates this produced, both independent of option A

* **The `Result<Flow, Failure>` at `run.rs:6088`.** Shrinking it to 16 bytes, or returning a shape LLVM keeps in registers, helps **BASE and HEAD alike** and is worth doing on its own merits rather than as a regression fix. It will not move `arith`, `varlookup` or `compound`.
* **Breaking up `Interp::step` and `run_loop_with_header`.** Until they are, every change touching `Body` reads as several per cent on axes it never executes, and no attribution to the change itself is trustworthy. This is a measurement problem before it is a performance one.

**A fix was attempted and rejected**: `#[inline(never)]` on the two `Bytes` constructors recovers about a fifth of the movement and holds the string wins, but it buys an out-of-line call on every string construction to chase a number this entry has just shown is not attributable. Not proposed.

---

### Entry 18 -- entry 17's first candidate attempted and **accepted**: the raised condition boxed, and 104 bytes off every loop iteration

**BASE for the comparison is `4fa6c3e4d`**, the same base entry 16 used, so the two entries' columns are directly comparable.
**HEAD is `46501f45f`.**

#### The hypothesis, named before it was measured

Entry 17 named this candidate from the subagent's disassembly and got one fact about it wrong, which is corrected here.
**The report called `Result<Flow, Failure>` a 24-byte value. It was 104.**
`Flow` is 24; `Loud` is 24; `Raised` is **104** -- a `Cow<'static, str>`, a `Vec<Substitution>`, two `Option<Vec<u8>>` and a `Delivery` -- and it sat inline in `Failure::Raised`, so the whole `Result` took its width.

The error arm is never taken on a hot path, and the success arm paid for it on every return.
`run_repeating`'s `loop` at `run.rs:5988` calls `run_bounded(..)?` at `run.rs:6088`, **once per DO-loop iteration**, so a 104-byte value was built through memory and immediately destructured on every pass of every loop in the language.

#### The change, and why it is two lines

`Raised` moves behind a `Box`. **`Failure` goes 104 to 24 and `Result<Flow, Failure>` goes 104 to 32.**

The blast radius was expected to be the 117 sites naming `Failure::Raised` and was **two lines**, which is worth recording because the estimate was wrong by two orders of magnitude:

* the existing `impl From<Raised> for Failure` absorbs every `?` and every `.into()`, which is how the value is constructed almost everywhere;
* `Deref` absorbs every match arm that binds the payload and reads a field.

The compiler found exactly two sites: a re-raise in `search_caller` that now **keeps** the box rather than unboxing and reboxing it, and one move into `ActiveCondition` that unboxes.

#### Measured movement

Wall clock in the accept rule's shape -- `ulimit -v 8388608`, `REXX_ENGINE=ir`, fresh empty working directory per run, side order rotated every round, seven rounds an axis, three builds interleaved in one sitting.
**All 126 runs exited 0, and each axis's stdout hash is identical across BASE, entry 16's HEAD and this HEAD.**

| axis | entry 16's HEAD | this HEAD | beats entry 16's HEAD | instructions |
|---|---|---|---|---|
| `emptyloop` | +5.73% | **-20.67%** | 7 of 7 | **-6.67%** |
| `compound` | +1.78% | **-3.21%** | 7 of 7 | -0.27% |
| `varlookup` | +2.36% | +0.10% | 7 of 7 | -1.81% |
| `arith` | +4.49% | +3.40% | 6 of 7 | -0.19% |
| `strings` | -6.74% | -5.56% | 0 of 7 | -0.93% |
| `alloc4c` | -11.87% | -10.58% | 1 of 7 | -0.83% |

**The six-axis geometric mean against BASE moves from -1.2% to -6.4%.**

**`emptyloop`'s instruction count falls 6.67% beside its 20.67% of wall**, which is what makes this attributable at all: entry 17 established that these four axes cannot resolve a change that merely perturbs the two megafunctions' register allocation, and an instrument-agreed instruction move is not that.

**`strings` and `alloc4c` give back about a point of cycles while their instruction counts fall.** Cycles moving against instructions is entry 17's floor exactly, it is not read as a cost, and both axes remain far ahead of BASE.

#### Is the two-level driver to blame

**Partly, and the split is worth stating because it decides where to look next.**

The nesting did not make the value large -- `Raised` inline in `Failure` did that, and the same 104 bytes would cross a single-level driver's returns too.
**What the nesting decides is how often it is paid.** `run_repeating` and `run_bounded` are the outer and inner halves of the driver, and the boundary between them is crossed once per loop iteration, so the cost is multiplied by exactly the quantity a benchmark loop maximises.

The evidence is in the instruction column and not in the wall column.
`emptyloop` saves **6.67%** of its instructions; every other axis saves between 0.19% and 1.81%.
`emptyloop` is the axis whose iterations do the least work besides crossing that boundary, so it pays the crossing at the highest rate and recovers the most when the crossing gets cheaper.

**So: the type's width was the cost, the two-level driver was the multiplier, and the width was much the cheaper half to fix.**
Whether collapsing the driver to one level is worth anything on top of this is **unmeasured** and is not claimed here.

#### What this entry does not claim

* **`arith` is still +3.40% against BASE and is not explained.** Entry 17 could not find its proximate stall either. It is the only axis this change leaves regressed.
* **No oracle ratios.** `rexx-bench-suite` was not run.
* **Resident set was not re-measured.** Boxing moves an error payload to the heap on a path that is not taken in any of these runs, so no movement is expected, but none was measured either.
* **The tree-walker arm was measured for correctness only**, as in entries 12 and 16.

### Entry 19 -- the design's option D(ii) attempted and **accepted**: a shared-borrow accessor for bytes, and three things that rode in with it

**BASE for the comparison is `a10164993`**, entry 18's HEAD, whose binary this sitting reproduced byte for byte (`27cbe3a2...`) before anything was changed.

#### What the option is

Entry 11's second consultation named a mechanism this record had never priced: `Interp::to_text(&mut self)` borrows the whole interpreter, because it fills two lazy caches.
So any code wanting two operands' bytes at once, or one operand's bytes and then any further call, cannot hold what it returns and buys its way out with an owned copy.
The value-representation design turns that into option D(ii): a `&self` accessor, ordered fourth and after option A, "because A changes the payload type its accessor would borrow from".

The accessor is a pair, and the pair is the part worth stating.
`try_text(&self) -> Option<&[u8]>` answers for `Nil`, a `Body::Text`, a `Body::Stem` and a rendered `Body::Num`; it answers `None` for a tagged small integer and for a `Body::Num` nobody has rendered yet, and both are "there is nothing to borrow" rather than "there is no text".
`render(&mut self) -> Rendered` turns the second cause into the first and carries the bytes for the first.
A call site does every `&mut` thing it needs first, calls `render` once per value, and then takes all its shared borrows together.

`Rendered` holds the `ObjRef` it was made for. That is not tidiness: the two halves of a converted call site are deliberately far apart -- every `&mut` use goes between them -- so pairing one value's `Rendered` with another value's read is a mistake that is available to make and would produce the wrong string silently. Holding the ref means the call site never names it twice.

**A stem's `default` redirect is chased inside `try_text`**, where `to_text` has to answer it with an owned copy: two shared borrows of `self` can coexist and two `&mut` ones cannot, so the recursion returns its result directly. The mechanism the option is about, applied to the value model's own internals.

#### What was converted, and what was not

Four sites, chosen because they are on a measured axis:

* `Interp::concat` (`||`, `Abuttal`, `Blank`) -- both operands read through shared borrows, into a buffer sized before anything is written, handed to `text_owned` rather than copied again by `text`.
* `Interp::eval_compare` -- the two parses and the two settings reads move in front of the two byte reads.
* `pos`, `substr` and `changestr` -- through a new `required_render`, with each builtin's numeric and pad arguments converted before its strings are read.
* `stem_get`'s clone of the stem's own name, which is not a `to_text` at all but is the same mechanism: a copy bought to release a borrow.

**Not converted:** `eval_logical`, and the rest of `builtin/`'s `required_string` callers.
They are the same mechanism and none of them is on any axis measured here, so converting them would enlarge the diff without moving a number. `required_string` stays as the helper for a builtin that interleaves `&mut` work with its string reads.

**Reordering a builtin's argument conversions is not observable**, and that is the load-bearing claim under the conversions: reading a string cannot fail -- `to_text` is total -- so no error can change place, and the two caches it fills are pure.

#### Measured movement

Wall clock in the accept rule's shape -- `ulimit -v 8388608`, `REXX_ENGINE=ir`, fresh empty working directory per run, arm order rotated every round, seven rounds an axis, three arms interleaved in one sitting, host-idle gate passed after 11 samples.
**All 126 wall runs exited 0, and each axis's stdout hash is identical across all three arms.**
Instructions and cycles are a separate pass, `perf stat -e instructions:u,cycles:u`, three interleaved rounds an axis, medians.

| axis | instructions | cycles | wall | rounds head beats BASE |
|---|---|---|---|---|
| `strings` | **-8.29%** | **-13.41%** | **-14.08%** | 7 of 7 |
| `compound` | **-1.42%** | +0.41% | -0.61% | 7 of 7 |
| `alloc4c` | +0.23% | +4.30% | +0.23% | 1 of 7 |
| `emptyloop` | +0.00% | -0.54% | -0.67% | 6 of 7 |
| `arith` | -0.00% | -1.14% | -1.10% | 5 of 7 |
| `varlookup` | -0.00% | +0.51% | +0.39% | 3 of 7 |

**Three axes report an instruction count identical to BASE's to two decimal places**, which is what they should report: `emptyloop`, `arith` and `varlookup` execute none of the changed code. Their wall movements, -1.10% to +0.39%, are therefore the instrument's own spread and not results.

#### The floor, demonstrated inside this sitting rather than argued

Entry 17 established that these axes cannot resolve a change that only perturbs the two megafunctions' register allocation, and put the floor near seven points.
**This sitting contains a direct instance.** The middle arm is `head` with `render` changed to always copy -- same code shape everywhere, byte-identical stdout, and a body `compound.rex` never executes, since that program has no concatenation, no comparison and no string builtin.
That arm ran **6.81% slower than BASE on `compound`** while executing **1.42% fewer instructions** than BASE.

So `compound`'s -0.61% of wall is not the result on that axis. Its -1.42% of instructions is, and it is the `stem_get` clone: a compound read that resolves used to allocate and free a boxed slice to serve a branch it never takes, and a compound read that resolves is the whole of that axis.

#### What the -8.29% on `strings` actually decomposes into, which is not what the option predicted

The three-arm sitting and one further ablation arm split it. Every figure is instructions, median of three interleaved rounds:

| mechanism | of `strings`' instructions |
|---|---|
| `changestr`'s result buffer sized before it is written, rather than grown from empty | **-5.09%** |
| the owned copies `render` stops making | -2.64% |
| `concat`'s sizing and `text_owned`, and the reordering | about -0.56% |
| **all three** | **-8.29%** |

**The largest piece is not the borrow shape.** `changestr` built its answer in a `Vec::new()` and grew it, so a 43-byte result reallocated five times; it now allocates once. That fix is independent of D(ii) -- the old code could have had it -- and it rode in because the function was being rewritten anyway. Recorded here because a correct decision carrying a false reason is a shape this loop has produced before, and the reason for this one is measured rather than assumed.

**So D(ii)'s own falsifier needs stating twice, because the two readings differ.**
As written -- "a paired run moving `strings` by less than about 3%" -- it is **cleared**, and not narrowly.
Applied to the mechanism it was written about, the copies alone are **-2.64%** and would **trip** it.

#### What this entry does not claim

* **`alloc4c` is +0.23% of instructions**, a small increase, and the likely cause is named rather than measured: `render` probes `try_text` before falling back, so a heap operand costs one extra heap lookup against `to_text`'s one. It shows on the axis whose concatenation is two operands per iteration and nothing else. Not chased.
* **`arith`'s standing +3.40% against entry 16's BASE is untouched and still unexplained.** This change does not execute on that axis.
* **No oracle ratios.** `rexx-bench-suite` has now not been run for entries 16, 18 or 19, and `strings` has moved a long way across those three.
* **Resident set was not re-measured.**
* **The tree-walker arm was measured for correctness only.**
* **The two new tests do not add wrong-answer coverage**, and this was checked rather than assumed. Three representation-only mutations of `try_text` -- drop the stem redirect, never answer for a rendered number, never answer for a defaultless stem -- were each run against the suite with the two new tests skipped, and the pre-existing suite caught all three (2, 9 and 4 failures, the last after a harness abort). What the new tests carry is the `None` contract, which nothing observable from Rexx can express.

### Entry 20 -- the oracle ratios, owed since entry 11 and taken at last, at `64a7a7aa4`

**No change, no hypothesis, no disposition.**
Entries 12, 16, 18 and 19 each recorded "**No oracle ratios.** `rexx-bench-suite` was not run", and by entry 19 that debt covered a change that moved one axis 14% of wall.
This entry pays it. The bar is stated against the oracle, and for four accepted changes nobody knew where any axis sat.

#### Provenance

| | |
|---|---|
| repo commit | `64a7a7aa4762e6f1c1ac6b667df976bf17aeced3`, tree clean |
| `rexx-bench-suite` | sha256 `4949a321...` -- **entry 1's harness binary exactly**, so the instrument has not moved across the whole phase |
| `rexx-run` | sha256 `4178780b...`, size 13848192 -- entry 19's HEAD |
| oracle | the same three objects entry 1 fingerprints, re-hashed and unchanged: `bb5bb8cc...`, `42136c40...`, `3536b763...` |
| configuration | the one fixed at the top of this file: `Wrapper::default()`, `ulimit -v 8388608` both sides, fresh empty working directory per child, 9 sampled pairs and 1 warm-up, oracle and this crate alternating |
| arm | `REXX_ENGINE=ir`, and a hostile `REXX_ENGINE=bogus` was set in the launching environment for entry 1's check -- the report prints `ir` |
| idle gate | passed at the first six samples, 98.2 to 99.4 per cent |
| same work both sides | every axis: stable within each side, identical across sides, bytes quoted in the report |

**The harness report this table comes from is committed verbatim as `phase-4f-oracle-64a7a7aa4.md`**, so every figure here can be checked against the one the instrument printed rather than retyped.

**One block, not two.** Entries 1 and 11 ran two blocks and used the gap between them as this instrument's own resolution. There is no block gap here, so a movement smaller than the roughly half a per cent those entries measured is not resolved by this sitting, and none of the movements below is that small.

#### Where the axes now sit

| axis | entry 1 (`ir`) | entry 11 | **now** | since 11 | since 1 | of entry 1's excess over the oracle, the part now gone | of this crate's time, the fraction that still has to go |
|---|---:|---:|---:|---:|---:|---:|---:|
| `alloc4c` | 1.9793x | 1.2740x | **1.1665x** | -8.4% | -41.1% | 83% | 14.3% |
| `emptyloop` | 3.1816x | 2.1824x | **1.6856x** | **-22.8%** | -47.0% | 69% | 40.7% |
| `arith` | 2.6658x | 2.1291x | **1.9258x** | -9.5% | -27.8% | 44% | 48.1% |
| `varlookup` | 3.7409x | 2.1568x | **2.1101x** | -2.2% | -43.6% | 59% | 52.6% |
| `compound` | 5.8982x | 2.4184x | **2.3667x** | -2.1% | -59.9% | 72% | 57.7% |
| `strings` | 10.5770x | 6.2666x | **4.9831x** | **-20.5%** | -52.9% | 58% | 79.9% |
| `rexxcps` (cps) | 7.3466x | 6.4436x | **5.6173x** | -12.8% | -23.5% | 27% | 82.2% |

**All seven remain NOT MET**, and the bar is "within measurement noise of the oracle or better".
The nearest axis is `alloc4c` at 1.17x and the two walls are `strings` at 4.98x and `rexxcps` at 5.62x.

`startup` remains not comparable and no ratio is taken.
`alloc`, `dispatch` and `heapshape` still exit 120 with `rexx-exec: a message send is not implemented (Phase 5)`, as they have since entry 1.

#### What this says that the per-change entries could not

**The two axes that moved most since entry 11 are the two the last two accepted changes were aimed at**, and the sizes are consistent with what those entries measured in isolation:

* `strings` -20.5%, against entry 19's -14.08% of wall on that axis. The extra is entry 12's, which had no oracle figure either.
* `emptyloop` -22.8%, against entry 18's -20.67%.

That agreement is the useful part. It is the first cross-check this phase has that the paired same-binary instrument and the oracle instrument are telling the same story, and they are.

**`rexxcps` is now the worst axis by "fraction that still has to go" and nothing this phase has done was aimed at it.**
It moved -12.8% since entry 11 as a side effect. Entry 11 already named it one of the two walls and observed that its 40.0% allocator family has never been decomposed into `Text`, `Number` and tail keys -- **that decomposition is still not taken, and it is still the cheapest thing that would reorder the candidate queue.**

**`compound` and `varlookup` have gone nearly flat** -- -2.1% and -2.2% across four accepted changes -- while sitting at 2.37x and 2.11x with more than half their time to lose. Whatever is left on those two is not what entries 12 to 19 were removing.

#### What this entry does not claim

* **No profile.** This is the timed instrument only; no `samply` run was taken and no share is attributed. Entry 11's shares are now nine commits old and should not be read as current.
* **No per-change attribution.** The move columns cross four accepted changes and a sitting boundary, exactly as entry 11's did.
* **The tree-walker arm was not run**, so this entry says nothing about the arm ratio.

### Entry 21 -- is the two-level driver avoidable: measured, and mostly it is already gone

**No change landed.** Entry 18 left the question open -- "whether collapsing the driver to one level is worth anything on top of this is **unmeasured** and is not claimed here" -- and this entry closes it.

#### The two levels are source structure, not two calls

Read out of the binary rather than the source: `nm -C --print-size` on `rexx-run` at `1767f1ff9` has **no symbol for `run_repeating` and none for `run_bounded_from_chunk`**. Both are already inlined into `run_loop_with_header`, which is 0x360e = 13,838 bytes.

What survives as a call is one thing: `run_ops::<false>`, 0x2adf = 10,975 bytes, entered once per loop iteration through `run_bounded_from_chunk`'s single statement. That call is the whole of the boundary entry 18's 104-byte `Result` was crossing.

#### What forcing it away costs and buys

`#[inline(always)]` on `run_ops` removes the symbol. Three interleaved rounds an axis, medians:

| axis | instructions | cycles |
|---|---:|---:|
| `varlookup` | **-0.83%** | -1.85% |
| `emptyloop` | **-0.64%** | -1.48% |
| `compound` | -0.34% | +6.24% |
| `strings` | -0.10% | -0.02% |

**So the remaining boundary is worth six to eight tenths of a per cent of instructions on the two clause-dispatch axes**, an order of magnitude below the 6.67% entry 18's width fix took off `emptyloop`, and `compound`'s +6.24% of cycles beside -0.34% of instructions is entry 17's floor again on an axis where nothing real moved.

**Not landed**, and the reason is in the same file it would be landed in. `ir/drive.rs` already carries a comment recording that bundling `run_ops`' four range-invariant arguments into a struct "does what it promises to `instructions:u` -- 6 fewer per range entry -- and on `cycles:u` it moves both arms of `bench-programs/emptyloop.rex` far more than that in opposite directions". This is the same size of change against the same pair of megafunctions that entry 17 measured as sitting at the edge of register allocation, and a one-line `inline(always)` that grows `run_activation`'s copy of the op loop is exactly the kind of thing that gets its instruction win back in cycles somewhere unmeasured. **It is worth about 0.7% and it is available; it should ride with a change that has its own reason to re-measure these axes, not on its own.**

### Entry 22 -- queue candidate 4's first half attempted and **accepted**: a builtin resolved to a row, not re-found on every call

**BASE is `1767f1ff9`.**

#### What the linear scan actually costs, which is the question this entry started as

`builtin::dispatch` ended with `IMPLEMENTED.iter().find(|builtin| builtin.name == name)` -- a walk of the 66-row table comparing byte slices, on **every builtin call**.

**Counted first.** `strings.rex` calls `POS` (row 37), `SUBSTR` (46), `CHANGESTR` (12) and `LENGTH` (32), so one iteration walks **131 rows** and the run walks **393,000,000**.

**Then measured, by replacing the scan with a name-to-row map and changing nothing else.** Three interleaved rounds:

| | `strings` |
|---|---:|
| instructions removed | **1,035,000,000** |
| ... as a share | **-1.87%** |
| per row visited | **2.6 instructions** |
| cycles | **-3.76%** |
| wall | -3.63% |

**Entry 11 sampled `Iter<Builtin>::find` at 8.4% of this axis. It is 1.87% of the instructions and 3.76% of the cycles.** The scan is a tight, perfectly predicted loop over about two kilobytes of static table, so it costs far less per row than a sampled share suggests -- and it costs about twice as much in cycles as in instructions, which is the memory walk showing up. **Entry 11's own warning, written for the candidate one place above this one, applies here and was right: "sampling attributes time; it does not attribute instructions."**

#### The change, which is two edits to one file

* **`Resolved`'s builtin arm gains a target.** `builtin::resolve(name) -> Option<BuiltinTarget>` answers `Row(u16)` or `Gap`; `builtin::run` indexes. The old doc said carrying "which builtin" beside the resolution would split the arity check from the code and let the two drift -- that argument is about the row's *contents*, and an index names the one row and cannot disagree with it.
* **The row map is asked before the in-scope set.** Resolution used to hash the name for `is_builtin` and then hash it again inside `dispatch`. A hit in the row map is conclusive because every row is in scope, which `every_implemented_row_names_an_in_scope_builtin` already asserted, so the common case now hashes once.

**The second edit is the larger half by instructions and the smaller by time**: one hash rather than two is **-7.06%** of `strings`' instructions but only -2.40% of its cycles, where the scan is the other way round.

#### Measured movement

Seven rounds an axis for wall, three for the counters, arms alternating within every round, host-idle gate passed, all 84 wall runs exit 0, every axis's stdout hash identical across both arms.

| axis | instructions | cycles | wall | rounds |
|---|---:|---:|---:|---:|
| `strings` | **-8.80%** | **-6.21%** | **-5.46%** | 7 of 7 |
| `alloc4c` | **-4.30%** | **-5.24%** | **-4.21%** | 7 of 7 |
| `arith` | -0.00% | -1.09% | -0.89% | 7 of 7 |
| `varlookup` | -0.00% | -0.48% | -0.53% | 5 of 7 |
| `compound` | -0.00% | -0.66% | -0.18% | 3 of 7 |
| `emptyloop` | -0.00% | -0.29% | +0.24% | 3 of 7 |

**Four axes report an instruction count identical to BASE's to two decimal places** -- they call no builtin -- so their wall columns are the floor and are not read as results. The two axes that call builtins move on both instruments.

#### Against entry 11's estimate for this candidate

Entry 11 put candidate 4 at **17.4% of `strings`** and an implied landing of 6.2666x -> 5.18x. **This half of it delivers 8.80% of instructions and 5.46% of wall.** The estimate is about twice the outcome, and the split between its two named parts is the other way round from the sampling: the set lookup it gave 9.0% is the bigger half in instructions and the scan it gave 8.4% is the bigger half in cycles.

**The other half of the candidate is not built.** What remains per call is one hash of the name, plus `resolve_call`'s `Rc::clone` of the program and its label-map lookup. Removing those needs the resolution kept at the call site, which for the expression form needs an `Op::CallExpr` -- the spike's own staging, and the next increment.

#### What this entry does not claim

* **`BuiltinTarget::Gap` is unreachable today**, and that was found by a test written to assert the opposite. Phase 4's in-scope set and this crate's table are the same 66 names. The arm stays because the two sets are derived separately at run time, and the test now asserts the gap set is empty so that widening one without the other fails here rather than answering wrongly.
* **No oracle ratios.** Entry 20's were taken at `1767f1ff9`, which is this BASE, so they are current for BASE and not for HEAD.
* **The tree-walker arm was measured for correctness only.**
* **No profile.** Whether the remaining resolution work is where the model says it is has not been re-sampled.

### Entry 23 -- queue candidate 4's second half attempted and **accepted**: a call at the root of a value keeps its resolution

**BASE is `5678d5f1b`**, entry 22's HEAD.

#### What it is

Entry 22 left one hash of the name per call, plus `resolve_call`'s `Rc::clone` of the program and its label-map lookup. Removing those needs the resolution **kept at the call site**, which the statement form has had since 4e: `Op::Call` carries a `site`, and `Chunk` holds a `Calls` table of what each site last resolved to. The expression form had no op and so no site.

`Op::CallExpr { index, slot, site, dst }` is that op, and **the one thing it does that `Op::EvalExpr` does not is skip `resolve_call`.** The argument loop with its `>A>` lines, the depth guard, the activation bookkeeping and the three `Ended` arms -- 44.1 for a routine that returns nothing among them -- are the same functions `eval.rs` calls on the same node. `eval_call` was split at the resolution so both engines enter the second half rather than one of them carrying a copy.

**Only at the root of a slot's expression.** A slot is the finest address an op has, so a call nested inside a larger expression has none to give and stays inside the `EvalExpr` covering the whole tree. `zz = f(1)` promotes; `zz = f(1) + 1` does not. That is the same fact `native_shape`'s own doc gives about operands, and a golden test pins both halves of it -- either alone is satisfied by a compiler that promotes calls everywhere or nowhere.

**Two pieces of `eval`'s per-node work had to be shared rather than skipped**, and both were found by asking what `eval` does that a native op bypasses:

* **`enter_eval_node`**, extracted from `eval`. A call's *arguments* go back through `eval`, so an op that skipped the depth bookkeeping would start them one level shallower than the tree-walker does and move the depth at which a deep argument raises 5.3.
* **`Op::TraceFunction`**, behind the call op, for the `>F>` line `eval`'s post-order hook emits and this op does not reach. The same pattern `Op::Const`/`Op::TraceLiteral` already use.

#### The width budget decided two field types

At `u32` each, `CallExpr`'s four fields make the variant 16 bytes and `assert!(size_of::<Op>() == 12)` fails. `slot` and `site` are `u16`, and the call-site index narrowed from `u32` to `u16` throughout -- guarded with `ChunkTooLarge` rather than assumed, the same answer every other index in the module gives. **The budget entry 11's consultation called undefended is still undefended, and this entry did not test it; it fitted inside it.**

#### Measured movement, with a control

Three arms interleaved within every round: BASE, **the ops present and nothing emitting them**, and HEAD. That middle arm is the control this change needs -- it separates "the op stream changed" from "the enum and the driver's match grew", which are different costs paid by different axes.

Wall and cycles are medians of three interleaved rounds; the seven-round wall sitting taken separately against BASE agrees on every sign.

| axis | instructions, control | instructions, HEAD | wall, control | wall, HEAD |
|---|---:|---:|---:|---:|
| `strings` | +2.55% | **-4.76%** | +0.19% | **-11.08%** |
| `alloc4c` | +2.25% | +2.25% | -0.10% | +2.76% |
| `compound` | +0.15% | +0.15% | +0.24% | +2.84% |
| `arith` | +0.24% | +0.24% | -2.47% | +4.69% |
| `varlookup` | +0.44% | +0.44% | -0.26% | -0.79% |
| `emptyloop` | +0.46% | +0.46% | +0.31% | +0.49% |

**On every axis that emits no call op the two instruction columns are identical.** So the whole of the instruction increase on those five -- 0.15% to 2.25% -- is two more `Op` variants and two more arms in the driver's match, paid whether or not anything is emitted. On `strings` the control is +2.55% and HEAD is -4.76%, so **the emission itself is worth 7.3 instruction points** and pays for the enum four times over on that axis.

**The wall regressions are not attributable and are not claimed.** `alloc4c`, `compound` and `arith` move +2.76%, +2.84% and +4.69% at instruction counts identical to the control's, and the control moves the same axes by -2.47% to +0.31%. `arith` is the clearest: two binaries whose instruction counts on that axis differ by **0.00%** sit **7.2 points apart** in wall clock across this sitting. That is entry 17's floor, demonstrated again and at the size entry 17 measured it.

#### What this entry does not claim

* **`alloc4c`'s +2.25% of instructions is real and unexplained.** It is the largest price the enum's growth charges any axis, it is present in the control, and no mechanism was found for why that axis pays five times what `compound` pays. `alloc4c` runs no `CallExpr`: its only call, `length(s)`, is nested inside `total + 3 + length(s)` and stays an `EvalExpr`.
* **Three of `strings`' four calls are promoted, not four.** `length(joined)` is nested and stays. What the remaining widening is worth on that axis is unmeasured here.
* **No oracle ratios.**
* **The tree-walker arm was measured for correctness only.**
* **Nothing was measured about the `Op` width budget**, which this change fitted inside rather than tested.

### Entry 24 -- the `Op` width budget measured at last, and the naive reading of it inverted by its own control

Entry 11's consultation called `const _: () = assert!(size_of::<Op>() == 12)` undefended by measurement, and entry 23 closed with "nothing was measured about the `Op` width budget, which this change fitted inside rather than tested".
Moritz then said he did not mind sixteen bytes -- it aligns better with cache lines -- which turned the question from a preference into one worth settling, because a free sixteen bytes deletes the side table the nested-call address was going to need.

#### The arms

Three binaries, all built from `1ccb81199`'s tree, `--release`, the workspace profile.

| arm | what it is | `size_of::<Op>()` |
|---|---|---:|
| A | head, untouched | 12 |
| C | head plus `Pad { a: u32, b: u16 }`, never constructed, never emitted, never driven | 12 |
| B | head plus `Pad { a: u32, b: u32, c: u32 }`, likewise dead | 16 |

**C is the arm that makes this a width measurement rather than a diff measurement.**
B differs from A in two ways at once -- it has an extra variant *and* it is wider -- and this project has already withdrawn four regressions (entry 17) after a control with no semantics moved the same axes as far.
C carries the extra variant and keeps the width, so **B against C is the width** and **C against A is having a variant at all**.
The width the assertion states is what forces the two `Pad` shapes apart: rustc packs the tag into a niche at twelve bytes, and three `u32`s carry no niche.

Both control arms are the "feature present, nothing emitting it" shape rather than "the feature removed", which is the shape entry 19 records as the one that can distinguish rather than only disclaim.

#### The sitting

The accept rule's literal shape and **not** `rexx-bench-suite`, so it is its own instrument.
Wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, a fresh empty working directory per run, `/dev/null` on stdin, stdout hashed per run.
Nine rounds per axis, **arm order rotated through all six permutations** so a drift across the sitting cannot become an arm difference.
2026-08-12 16:47 to 16:56 +02:00, behind the host-idle gate (six consecutive five-second `/proc/stat` samples at or above 90% idle; the gate refused for most of the afternoon and this is the window it passed).
Load average 1.06 to 1.19 across the sitting.
All 162 runs exited 0, and **every axis printed a byte-identical stdout under all three binaries in all nine rounds**.

| axis | A mean | C mean | B mean | C vs A -- the variant | B vs C -- **the width** |
|---|---:|---:|---:|---|---|
| `emptyloop` | 1.5096 | 1.5549 | 1.5434 | **+3.00%**, 9/9 slower | -0.74%, 0/9 |
| `compound` | 2.7465 | 2.7852 | 2.7704 | **+1.41%**, 9/9 slower | -0.53%, 3/9 |
| `strings` | 3.6669 | 3.7169 | 3.6778 | +1.36%, 6/9 slower | -1.05%, 3/9 |
| `alloc4c` | 1.2410 | 1.2530 | 1.2471 | +0.97%, 7/9 slower | -0.47%, 3/9 |
| `varlookup` | 2.6029 | 2.6044 | 2.5908 | +0.06%, 7/9 slower | -0.52%, 4/9 |
| `arith` | 2.2404 | 2.2191 | 2.2120 | -0.95%, 2/9 slower | -0.32%, 5/9 |

#### What it says

**Sixteen bytes is free on these axes, and the cost the two-arm reading found was the variant rather than the width.**
A first sitting compared A against B alone and read `emptyloop` +2.35% at 9/9 -- wait for the control and that same axis splits into +3.00% for the variant and **-0.74% for the width, with B at or under C in every one of nine rounds**.
Every axis's width column is negative.
The direction is consistent; the magnitudes are not claimed, because only `emptyloop`'s round count separates from chance and every figure here sits under entry 17's floor.

**The variant cost is the finding nobody was looking for.** Adding a dead variant -- constructed nowhere, emitted nowhere, driven nowhere -- costs `emptyloop` 3.00% at 9/9 and `compound` 1.41% at 9/9.
That is consistent with entry 23's control, which measured two new `Op` variants at +0.15% to +2.25% of instructions on the axes that emit neither.
No mechanism is established here; the plausible one is that a variant changes rustc's discriminant assignment and so every match's dispatch, but that is a hypothesis and this entry does not test it.

#### What this entry does not claim

* **No instruction or cycle counts.** Wall clock only, deliberately: `perf stat` moves the ratio on every axis (entry 1), and the question was whether the width costs under the rule the bar is stated on. What the width does to instructions per cycle is unmeasured, and it is the mechanism a cache-line argument would actually run through.
* **Nothing about a *widened variant*, which is what would actually land.** Both wide arms widen the enum by adding a dead variant; Task 3's alternative widens `Op::CallExpr` itself and adds none. The width column is the right prediction for it and the variant column is not, but the exact change was not built.
* **Six axes, one machine, one afternoon.** `rexxcps` was not run.
* **It says nothing about whether the assertion should stay.** What it says is that the number in it may be sixteen without paying for it here.

### Entry 25 -- the expression-promotion plan attempted and **accepted with a reproducible regression on three axes**: every operator native, and a call promoted wherever an address reaches it

`docs/superpowers/plans/2026-08-12-expression-promotion.md`, four tasks, `ec649c0bc..8a48bbb1d`.
Entry 23 closed with "three of `strings`' four calls are promoted, not four" and "what the remaining widening is worth on that axis is unmeasured here".
This is that measurement, plus the operators entry 23 did not touch.

#### What landed

* **`Op::Binary`** for the concatenation, comparison and logical operators, through one `Interp::apply_binary` that `eval_node` also enters, with `concat_values`/`compare_values`/`logical_values` split out of the three functions that used to own an operand prologue each.
* **`Op::Prefix`** and **`Op::TracePrefix`**, through `Interp::apply_prefix`. A separate echo op because a prefix operator's line is `>P>` and a binary operator's is `>O>`.
* **`NodePath`**, a bit-encoded route from an expression slot's root to a node, carried in `Op::CallExpr` and `Op::TraceFunction`; `Interp::chunk_node_at` is the descent. `size_of::<Op>()` went from 12 to 16 on entry 24's measurement.
* **A call promotes wherever an address reaches it inside a slot that compiles natively.** `push_value`'s root-call special case is gone.

#### The sitting

The accept rule's literal shape, **not** `rexx-bench-suite`.
Wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, a fresh empty working directory per run, stdout hashed per run.
**Three arms in one sitting** -- `ec649c0bc` (base), `87f0e0d32` (after the operators), `8a48bbb1d` (after the address and the nested call) -- so the drift cancels across all three comparisons rather than only within two separate ones.
Nine rounds per axis, arm order through all six permutations.
2026-08-12 20:12 to 20:18 +02:00, behind the host-idle gate, load average 1.06 to 1.15.
All 162 runs exited 0, and **every axis printed a byte-identical stdout under all three binaries in all nine rounds**.

| axis | base | operators | + nested call | operators | nested call | whole plan |
|---|---:|---:|---:|---|---|---|
| `strings` | 3.6673 | 3.6259 | 3.2990 | -1.13%, 6/9 | **-9.02%, 9/9** | **-10.04%, 9/9** |
| `alloc4c` | 1.2539 | 1.2478 | 1.1539 | -0.48%, 6/9 | **-7.53%, 9/9** | **-7.98%, 9/9** |
| `compound` | 2.7980 | 2.7594 | 2.7491 | -1.38%, 9/9 | -0.37%, 6/9 | -1.75%, 9/9 |
| `arith` | 2.2014 | 2.2738 | 2.2886 | **+3.29%, 0/9** | +0.65% | **+3.96%, 0/9** |
| `varlookup` | 2.4419 | 2.5407 | 2.5683 | **+4.05%, 0/9** | +1.08% | **+5.18%, 0/9** |
| `emptyloop` | 1.4420 | 1.5114 | 1.5382 | **+4.82%, 0/9** | +1.77% | **+6.67%, 0/9** |

The round counts are how many of nine rounds the later arm was *faster*, so `0/9` means the later arm lost every round.

#### Instructions, which change what the wall clock means

`perf stat -e instructions:u`, one run per arm, a different configuration from the sitting and quoted as a mechanism check rather than as a suite figure.

| axis | base | head | difference |
|---|---:|---:|---|
| `strings` | 47,943,770,024 | 44,565,789,512 | **-7.05%** |
| `varlookup` | 43,700,817,052 | 44,384,817,866 | +1.57% |
| `emptyloop` | 27,525,811,377 | 27,525,811,294 | **-0.0000003%** |

**`emptyloop` executes the same instructions and takes 6.67% longer in every round.** Eighty-three instructions separate the two binaries out of twenty-seven and a half billion, so the regression on that axis is **not the interpreter doing more work**; it is where the code landed. `varlookup` is a little of both. `strings` removed real work and the wall gain exceeds it.

#### What this entry does not claim

* **The mechanism of the regression is not established.** It has entry 24's signature -- that entry measured a **dead** `Op` variant, constructed and driven nowhere, costing `emptyloop` 3.00% at 9/9, and this plan added three real variants for a first-increment cost of +4.82% on that axis -- and it is the right order of magnitude. But the control that would attribute it was not built: head with the new ops present and nothing emitting them. Until that exists, "three more variants cost this" is a hypothesis with a matching fingerprint, not a measurement.
* **`compound`'s -1.75% at 9/9 is unexplained.** That axis holds no concatenation, no prefix operator and no call. It is small and it is consistent, and no mechanism is offered.
* **The instruction counts are one run per arm**, not an interleave. They are reproducible to a few hundred parts per billion on this machine (entry 1 measured the same counter twice at 763411912 and 763411673), which is why one run is quoted; the wall-clock figures are the nine-round ones.
* **`rexxcps` was not run.** It remains the worst axis by fraction still to lose and nothing in this plan was aimed at it.

#### Accepted, and what follows

Moritz accepted the movement on 2026-08-12.
**The variant count is now a measured cost rather than an aesthetic one**, which turns merging op variants into an optimisation candidate with a number attached: `Op::Arith` folding into `Op::Binary` with the hint field, or the trace ops collapsing into one carrying a tag. Measured against exactly these axes, with the control this entry says is missing.

### Entry 26 -- what the intermediate echo ops cost when nothing is traced, and a control span that is wider than the effect on every axis it was measured against

A price probe, not a candidate patch, and **nothing from it was merged**.
Base `0459167cc8c513eda448b5bb3a66cb0fdd64a836`.
Measured by a subagent in a detached worktree on 2026-08-12/13; the full report is at `.superpowers/sdd/2026-08-12-condition-promotion/echo-op-price.md`, which is git-ignored, so the numbers below are the record of it.
**Not reproduced by a second party.** The arms, the byte-identity checks and the round counts are that agent's; what is quoted here is quoted with that provenance.

#### The question

`Op::TraceClause`, the clause echo, is emitted only when the chunk's compile-time trace setting echoes -- `echoes()` and `push_echo()` decide it.
The *intermediate* echoes are not: `push_native` pushes `Op::TraceLiteral` behind every literal and constant symbol, `Op::TraceRead` behind every symbol read, `Op::TraceOperator` behind every binary operator, `Op::TracePrefix` behind every prefix operator and `Op::TraceFunction` behind every call, unconditionally.
So a promoted expression carries about one echo op per leaf and per operator, and each is a dispatch, a register read and a call that asks whether intermediates are traced and returns.

**Why unconditional is the current answer**: the ops being present is what lets a `TRACE I` executed mid-run start printing without recompiling the chunk.
A build that omits them is wrong in general, which is why this is a probe.

#### Four arms, not three

* **A** -- head.
* **B** -- gated at emission, the trace setting threaded into `push_native`/`push_value`/`push_read`.
* **C** -- B's plumbing exactly, flag forced so every echo op is still emitted.
* **D** -- the same again, forced the other way round.

The fourth arm was the measuring agent's own decision, taken after A-vs-C came out the same size as B-vs-C.
B, C and D differ in one line.

#### Wall clock, twenty interleaved rounds, order rotated, idle gate 90.4% to 94.8%

| axis | B vs C | B faster | A vs C (control) | A faster | C vs D | A vs D |
|---|---:|---|---:|---|---:|---:|
| `rexxcps` | -0.44% | 14/20 | -0.60% | 16/20 | -0.28% | -0.87% |
| `arith` | -8.03% | 20/20 | **-8.29%** | 20/20 | +3.48% | -5.10% |
| `varlookup` | -7.25% | 20/20 | -4.32% | 20/20 | +0.19% | -4.14% |
| `strings` | -1.76% | 19/20 | -3.48% | 20/20 | +0.72% | -2.78% |
| `alloc4c` | -1.92% | 18/20 | -0.28% | 13/20 | -0.93% | -1.21% |

**The control movement is the result.**
On `arith` the three identical-behaviour arms span 9.04%, and head beats its own control 20/20 in the same direction and with the same unanimity as the "improvement".
B lands inside the A/C/D band on `rexxcps`, `arith` and `strings`.
It is outside on `varlookup` (-3.06% against the fastest identical build, 20/20) and `alloc4c` (-1.65%, 18/20), both under the resolution floor.
A separate sixteen-round three-arm sitting replicated all of it.

#### Instructions, which the layout does not reach

`perf stat`. C and D agree to 0.00% and the run-to-run spread is at or below 0.09%, so this instrument sees the op stream rather than where it landed.

| axis | B vs A |
|---|---:|
| `rexxcps` | -1.64% |
| `arith` | -2.57% |
| `varlookup` | **-8.17%** |
| `strings` | -1.76% |
| `alloc4c` | -4.26% |

A three-probe differential prices one echo op, executed with nothing traced, at **40 user instructions for `Op::TraceLiteral`, 45 for `Op::TraceRead` and 61 for `Op::TraceOperator`**.
The model built from those three predicted `arith`'s removed instructions to 0.001% against a figure it had not been given.

#### The program's own TRACE and ADDRESS clauses

Arm A only, `rexxcps` at a fixed `count=100`/`averaging=100` because removing clauses invalidates the program's own clauses-per-second figure, which assumes a fixed count per iteration.
Removing its `trace value tracevar` and `trace value trace(); address value address()` clauses: 3.2556 s to 3.1877 s, **-2.09%, faster in 20 of 20 rounds**, instructions -1.82%.
Those clauses cost more instructions than every intermediate echo op in the whole program.

#### What this entry claims, and does not

* **The ops are real work and not measurable time.** Between 1.64% and 8.17% of retired instructions, and no wall-clock win that survives its own control.
* **It does not say the ops are free.** It says this instrument cannot see them on these axes at this size.
* **It is not a verdict on gating.** A correct version costs more than arm B, which silently stops printing intermediates when a `TRACE I` runs mid-program -- demonstrated directly, and caught by the dual-engine tests and six `trace_oracle` transcripts, which is what a real design would have to keep.
* **The finding worth acting on is the unit price**: spending 40 to 61 instructions to decide *not* to print puts the gate in the per-op path. Hoisting it helps the traced and the untraced case both and keeps a mid-run `TRACE I` correct. What the wall clock above says is how little that can be worth on these five axes.

#### Two side findings

* **No compile-time assertion requires an echo op to be present.** All six are of the form "for each echo op present, check the op before it", so an echo dropped by accident is invisible to `compile.rs`, and the corpus sweep is the only net. Worth knowing before anything is built on top of the emission decision.
* **The control span is wider than this record's stated resolution floor, on `arith`.** Entry 25 accepted a +3.96% regression on that axis at 0/9. That figure sits inside the range identical builds produced here for nothing. Entry 25's `strings` and `alloc4c` gains are far outside it and are not in question; its three regressions are, and the next measurement on these axes needs two do-nothing controls rather than one.

### Entry 27 -- the condition-promotion plan measured: a billion instructions off `rexxcps`, and a wall clock that cannot say by how much

`docs/superpowers/plans/2026-08-12-condition-promotion.md`, four tasks, `02c6b9b67..de6efa3b1`.
Every `IF` and `WHEN` condition and every `DO`/`LOOP` header value used to compile to one `Op::EvalExpr` running the tree-walker's `eval`; they now compile to native ops, and `RETURN`/`EXIT`/`PUSH`/`QUEUE` have ops of their own.

#### Why the plan aimed at `rexxcps` alone

A temporary per-clause counter over `samples/rexxcps.rex` (entry in the plan document) found `IF` the most-executed clause kind in that program, with **every** execution running an `Op::EvalExpr` for its condition, `WHEN` fourth with its condition reaching `eval` inside `scan_when`, and a `DO` header running about one `EvalExpr` per header execution.
No program in `rust/bench-programs/` holds an `IF`, a `WHEN`, a `SELECT` or a `CALL`, which the measuring agent confirmed by reading all six.
So the loop axes are a control by construction: **this plan cannot make them faster, and any movement there is layout.**

#### Four arms, two of which behave identically to another

A (base), A' (base plus a trivial no-op edit), B (head), B' (head plus the same).
Fifteen rounds, all four arms interleaved, order rotated through eight permutations, wall clock, `ulimit -v 8388608`, `REXX_ENGINE=ir`, fresh empty working directory per run, idle gate at 98.4% to 99.4%.
`rexxcps` pinned at `count=100`/`averaging=100`, with every run's `Averaged: 100 x 100` line checked so self-calibration never fired.

| axis | A to B | B won | control A/A' | control B/B' | against the rule |
|---|---:|---|---:|---:|---|
| `rexxcps` | **-2.97%** | 15/15 | 0.81% | 0.05% | exceeds both |
| `arith` | -3.41% | 14/15 | -0.44% | 0.37% | exceeds both |
| `emptyloop` | -5.12% | 15/15 | -0.35% | 0.23% | exceeds both |
| `strings` | +4.55% | 0/15 | 1.15% | -0.01% | exceeds both |
| `compound` | +3.27% | 1/15 | -0.31% | -1.71% | exceeds both |
| `varlookup` | +1.29% | 1/15 | -0.15% | 0.24% | exceeds both |
| `alloc4c` | +1.02% | 5/15 | 1.77% | 0.34% | inside a control spread |

**`rexxcps` clears both control spreads, and so do five axes this plan cannot touch**, spanning -5.12% to +4.55%.
A layout swing larger than the effect is the reason the wall-clock magnitude is not reportable, even though its direction was unanimous across all forty interleaved rounds of three sittings (-2.97%, -4.44%, -6.48%).

**The A/A' controls turned out to bound the wrong thing.** Under `lto = "fat"` and `codegen-units = 1`, four different trivial source edits all produced **byte-identical loaded images**, so those arms measure run-to-run noise and not layout at all. What bounds layout here is the loop axes, and they do not clear `rexxcps`.

#### Instructions, `perf stat -e instructions:u`, one run per arm

| axis | A | B | A to B |
|---|---:|---:|---:|
| `rexxcps` | 29,605,462,015 | 28,594,977,579 | **-3.41%** |
| `emptyloop` | 27,525,609,300 | 27,100,610,015 | -1.54% |
| `alloc4c` | 9,342,657,574 | 9,392,748,982 | +0.54% |
| `strings` | 44,565,576,116 | 44,787,576,200 | +0.50% |
| `varlookup` | 44,384,636,468 | 44,574,636,225 | +0.43% |
| `compound` | 37,064,852,737 | 37,163,438,975 | +0.27% |
| `arith` | 20,349,640,032 | 20,397,974,269 | +0.24% |

Arm-internal noise is at or below 0.005%, so this instrument sees the op stream rather than where it landed -- the same separation entry 26 found.
`rexxcps` loses **1.010 billion instructions**, 2.2 times the largest movement on any axis the plan cannot touch, and it is the one program holding the constructs the plan promoted.

**The cost side, which the wall clock hides.** Instructions rose 0.24% to 0.54% on four control axes: shared-driver codegen drift, paid by programs that gain nothing.

#### Byte-identity

All four arms produce byte-identical stdout, empty stderr and exit 0 on all six loop programs, including every `FailedN` guard and `NoValue` in `rexxcps` staying absent.
`rexxcps` differs across arms only in the two lines carrying its own elapsed time and clauses-per-second figure.
Nothing diverged and no timing was voided.

#### A measurement artifact worth more than the result

The first sitting's controls were contaminated by **the length of `argv[0]`**.
Four byte-identical copies of one binary spanned **3.47%** on wall clock, splitting cleanly by whether the basename was ten characters or eleven, with instruction counts identical to 0.0000%.
Staging every arm at one fixed path cut the control spread to 1.33%, and every number above comes from that re-run.
**A benchmark harness that names its arms by path is measuring the names.**

#### What this entry claims

* **The plan's aim is supported by the instruction counter and unresolvable on the wall clock.** A billion instructions left `rexxcps`, specific to the program holding the promoted constructs; how much time that is worth cannot be stated at this resolution.
* **It does not claim a wall-clock number for `rexxcps`.** The direction is unanimous over forty rounds; the magnitude sits under a layout swing that moved untouchable axes further.
* The static coverage fact is independent of all of this and was checked separately: after Task 1 the `evalexpr IF` row is absent from the execution profile entirely.

### Entry 28 -- the compound-name plan measured: a quarter of `compound` gone, and four axes that did not move by a thousand instructions

Plan: `docs/superpowers/plans/2026-08-13-compound-name-resolution.md`, two tasks.
Base `cb4f27d1c`, head `a93bfb548` -- Task 1 is `b3e8cf4d2` and Task 2 is that head, both read back from `git log`.

**This entry changes the wrapper, and says so rather than letting the difference pass as a result.**
The configuration block above fixes wall clock through `rexx-bench-suite`. This entry is `perf stat -e instructions:u`, one run per arm, arms interleaved per axis, every arm staged at one fixed binary path.
Entry 27 is why: on these axes the wall clock moved between -5.12% and +4.55% from layout alone, which is wider than anything this plan could produce, while the instruction counter's arm-internal spread stayed at or below 0.005%.
**No wall-clock figure is claimed by this entry**, and no wall-clock run was taken.

#### Cause, stated by the plan before either task

`perf record` over `samples/rexxcps.rex` at `5dc12a403` put `rexx_parse::ast::compound_parts` at 3.38% of self time with the `CharSearcher` its `split('.')` drives at 3.80%, and `hash_one::<&[u8]>` at 4.43% with SipHash's `write` at 3.29% and `Interp::slot_of` at 1.72%.
Both buckets are constants of the source text recomputed per execution: how a compound name splits, and which slot each variable tail piece resolves to.
`Plan::note_compound_name` already computed both at build time and discarded them.
Task 1 kept the split; Task 2 kept the slots.

#### Measured movement

| axis | base `cb4f27d1c` | head | difference |
|---|---:|---:|---:|
| `compound` | 37,163,096,047 | 27,800,230,124 | **-25.19%** |
| `alloc4c` | 9,392,724,256 | 8,456,718,073 | **-9.97%** |
| `rexxcps` | 28,594,847,258 | 25,865,205,907 | **-9.55%** |
| `varlookup` | 44,574,614,522 | 44,840,637,363 | +0.60% |
| `strings` | 44,787,587,435 | 44,928,589,222 | +0.31% |
| `emptyloop` | 27,100,609,164 | 27,150,610,085 | +0.18% |
| `arith` | 20,396,605,792 | 20,403,950,467 | +0.04% |

Every `rexxcps` run self-calibrated to `100 x 100`, checked on each arm's own output, so both arms did the same work.

**Split by task**, base to Task 1's head to Task 2's head, on the three axes that moved:

| axis | `cb4f27d1c` to `b3e8cf4d2` (the split) | `b3e8cf4d2` to head (the slots) |
|---|---:|---:|
| `compound` | -17.70% | -9.12% |
| `rexxcps` | -7.19% | -2.61% |
| `alloc4c` | not measured by Task 1 | -3.19% |

#### The control axes, and a correction to what "control" meant

**All of the +0.04% to +0.60% above belongs to Task 1.** Measured separately, Task 2 moved the four axes holding no compound variable by **43 to 903 instructions** out of twenty to forty-five billion -- `strings` -48, `varlookup` +43, `emptyloop` -903, `arith` +505 -- which is the same op stream, not small drift.

**The plan named `compound` as the only loop axis holding compound variables, and that was false.**
`alloc4c`'s inner loop is `tab.i = i`: a compound with a variable tail piece, which is exactly what the plan changes.
It was caught by measuring rather than by reading -- `alloc4c` moved 3.19% on Task 2 alone, which under the plan's own rule would have been reported as codegen drift three times larger than anything Task 1 saw -- and then confirmed by reading the program.
Task 1's table does not carry an `alloc4c` row, which is why the claim survived it.
The plan's text has been corrected in place with what was removed and why.
**A control axis that is not a control invents a cost that is really a benefit**, and the only thing that separated the two here was reading the program the number came from.

#### Disposition

**Accepted, and the hypothesis is confirmed by route as well as by outcome.**
The plan predicted movement on the programs holding compound variables and none elsewhere, and that is what the three subject axes and the four control axes read.
`compound` and `alloc4c` are the two programs whose inner loops hold a compound with a variable tail; `rexxcps` references `acompound.key1.loop` in its own innermost loop.
Nothing else moved by a measurable amount under Task 2, and only Task 1's shared-driver drift moved anything else at all.

#### What this entry does not claim

* **No time figure.** How much of a quarter of `compound`'s instruction stream is worth in seconds is not stated, for entry 27's reason.
* **The remaining name hashing is not gone.** A compound's *stem* still resolves by name at every reference (`stem_get`/`stem_set` open with `slot_of`), and a compound `DO` control variable's tail pieces still do, because the whole dotted name holds the slot and giving its parts slots would move the body's frame layout. Both are named in Task 2's report as work this plan did not take.

### Entry 29 -- corrections to entry 28, and the rule that says they belong here

**Entry 28 was twice edited in place, and this entry exists because that is not how this record works.**
`67927f216` changed a figure inside it and added a paragraph; `c993b6479` withdrew four figures and added two sections.
Both edits are good work and neither was hidden -- the second names the withdrawn figures inside the entry it edited -- but the rule at the top of this file is that an entry that turned out wrong is corrected by a later entry saying so, precisely so a reader can see what was believed and when.
Entry 28 has been restored to the text committed at `df3c34087`, and everything those two commits added is below, with who measured it.

#### `rexxcps`' Task 1 column: -7.19% withdrawn, **-7.1%** stands

Entry 28 published -7.19%, carried from Task 1's report, whose base was a single run higher than all three base runs the same report had taken minutes earlier.
Three readings of the same pair exist: Task 1's implementer re-measured at **-7.14%** over three interleaved rounds at one fixed binary path (base `cb4f27d1c` 28,598,071,071 / 28,592,601,793 / 28,588,567,089 against `b3e8cf4d2` 26,550,119,275 / 26,550,118,216 / 26,546,064,392, medians); Task 1's reviewer measured the same pair independently at **-7.12%**; entry 28's own arms imply **-7.16%**.
The base arm's spread over six runs is **0.033%**, which is why three honest readings differ in the second decimal.
**The correction is to the precision, not the value**: three digits were being claimed from an instrument that supports two here, and -7.1% is what survives re-measurement.

#### The four control axes: the signed figures are withdrawn, the conclusion is not

Entry 28 stated Task 2's movement on the four axes holding no compound variable as `strings` -48, `varlookup` +43, `emptyloop` -903, `arith` +505 instructions.
**Those four figures are withdrawn.** Arm-internal spread had been measured for `rexxcps`, where the effect is 2.61%, and not for these axes, where the claimed effect is one part in a billion.

Measured afterwards by Task 2's implementer -- three runs per arm, both arms at one fixed path, interleaved, one sitting:

| axis | same-binary span, base | same-binary span, head | arm to arm |
|---|---:|---:|---:|
| `emptyloop` | 1,333 | 1,078 | -419 |
| `strings` | 652 | 881 | +6 |
| `varlookup` | 332 | 474 | +523 |
| `arith` | 258 | 33 | +192 |

Every arm-to-arm figure is inside the span the same binary produces against itself.
Across the three sittings that exist -- entry 28's one-run figures, Task 2's reviewer's two-run replication (+229, -558, -506, -699), and the table above -- `strings`, `varlookup` and `arith` have each come out with **both signs**.
`emptyloop` has not: -903, -506, -419, negative every time. It is still inside its own 1,078-to-1,333 span, so the bound covers it, but **a repeated sign inside the spread is not evidence of a movement** and the exception is recorded rather than smoothed over.

**What these axes support is a bound**: Task 2 moves them by under about a thousand instructions out of twenty to forty-five billion, which is this instrument's resolution here.
Task 1's movement on the same four is 7.3 million to 266 million instructions, thousands of times that bound, which is why Task 1's share is attributable and Task 2's is not.

#### The cost side: `INTERPRET` pays for this plan

`Interp::fragment_plan` builds a whole `Plan` for an `INTERPRET` fragment and keeps only the id-to-slot translation, so from Task 1 onward it also builds and discards a `compounds` table on every execution of every `INTERPRET`.
A fragment's `Code` carries no plan on purpose -- its ids belong to its own `SymbolTable` -- so it splits its own spelling at every compound reference, and that split now allocates an owned name where the old code borrowed slices out of the interned one.

**Measured by Task 1's reviewer**, quoted with that provenance: `perf stat -e instructions:u`, three interleaved runs per arm at one fixed path, on `do i = 1 to 200000; interpret "x = i + 1"; end`.
Base `cb4f27d1c` 48.48 / 48.64 / 48.66 G against head 49.02 / 49.13 / 49.16 G, arms non-overlapping: **+1.0%**.
The same loop with the `INTERPRET` removed moved **+0.45%**, this plan's codegen drift on a program it cannot otherwise touch, so about half a percent is the fragment path rather than layout.

**No axis in entry 28's table holds an `INTERPRET`**, which is why nothing else saw it and why the figure needed a program written for it.
Accepted rather than fixed, against -7.1% on `rexxcps` and -25.19% on `compound`.
The cheap form, if it is ever worth taking, is a build mode that skips the compound table for a fragment plan.

#### What entry 29 does not change

Entry 28's three subject-axis figures, its split-by-task table apart from the `rexxcps` cell above, its correction of what "control" meant on `alloc4c`, and its disposition all stand, and all were reproduced independently by Task 2's reviewer at -25.20%, -9.97% and -9.56%.

### Entry 30 -- the stem's slot: the hashing bucket finally falls, and one control axis that is not one

Plan: `docs/superpowers/plans/2026-08-13-stem-slot-resolution.md`, one task.
Base `9513c8b13`, head `5eaa3c8e8`, both read back from `git log`.

**This entry changes the wrapper, for entry 27's reason and in entry 28's way**, and says so rather than letting the difference pass as a result.
The configuration block above fixes wall clock through `rexx-bench-suite`. This entry is `perf stat -e instructions:u`, **three interleaved runs per arm** on the subject axes and **six** on the control axes, every arm staged at one fixed binary path so `argv[0]` is byte-identical between them.
**No wall-clock figure is claimed by this entry**, and no wall-clock run was taken.

#### Cause, stated by the plan before the task

`perf record` over `samples/rexxcps.rex` after the compound-name plan showed its compound-name bucket down from 8.85% to 2.51% of self time while the symbol-lookup hashing bucket went **up**, 8.61% to 9.33%.
That plan removed the name hashing for a compound's *tail pieces* and left its *stem*: `stem_get`, `stem_set` and `stem_drop_tail` each opened with `slot_of(stem_name)`, so every read and every write of `acompound.key1.loop` hashed `ACOMPOUND.`.
`Plan::note_compound_name` already called `slot_for` on the stem to assign exactly that slot, and dropped it.

#### Predicted movement

The programs whose inner loops hold a compound (`compound`, `alloc4c`, `rexxcps`), and nothing else; the hashing bucket down.

#### Measured movement

Minimum of each arm's runs.

| axis | base `9513c8b13` | head `5eaa3c8e8` | difference |
|---|---:|---:|---:|
| `compound` | 27,798,509,899 | 25,052,491,453 | **-9.88%** |
| `alloc4c` | 8,456,675,211 | 8,189,737,101 | **-3.16%** |
| `rexxcps` | 25,858,071,136 | 25,297,898,179 | **-2.17%** |
| `arith` | 20,403,949,862 | 20,415,989,999 | **+0.06%** |
| `strings` | 44,928,588,505 | 44,928,588,385 | -120 instructions |
| `varlookup` | 44,840,637,218 | 44,840,636,876 | -342 instructions |
| `emptyloop` | 27,150,610,000 | 27,150,610,122 | +122 instructions |

`rexxcps` self-calibrated to `100 x 100` on every arm, checked on each arm's own `Averaged:` line.
`compound`, `alloc4c`, `varlookup`, `emptyloop`, `arith` and `strings` produce byte-identical stdout **and** stderr and exit 0 under both binaries on both engines.

#### The instrument's own spread, measured before any of the small figures were read

| axis | same-binary span, base | same-binary span, head |
|---|---:|---:|
| `compound` | 1,370,916 | 1,571,750 |
| `alloc4c` | 58,490 | 14,892 |
| `rexxcps` | 4,075,851 | 11,206,860 |
| `arith` | 1,439 | 1,250 |
| `strings` | 1,999 | 1,036 |
| `varlookup` | 607 | 954 |
| `emptyloop` | 1,430 | 1,456 |

So `strings`, `varlookup` and `emptyloop` carry a **bound and not a difference**: this change moves them by under about a thousand instructions out of twenty-seven to forty-five billion, which is this instrument's resolution there, and no signed figure survives.
Three `strings` runs came back 36 to 51 million instructions high -- two on the base arm and one on the head arm, the same signature entry 29's reviewer recorded on that axis -- and are excluded as external interference; every other run of every axis is used.

#### `arith` is not inside the spread, and that is the cost side

`arith` moved **+12,040,137 instructions**, about eight thousand times its own 1,250-to-1,439 span, with the six runs of each arm not overlapping at all.
It holds no compound variable, and that was measured rather than read: an accessor-level probe built for the correctness check fires on `compound`, `alloc4c` and `rexxcps` and is **silent** on `arith`, `strings`, `varlookup` and `emptyloop` under both engines.
So this is codegen drift on a program the change cannot otherwise touch, and it is recorded as the cost side rather than explained.
**No attribution beyond that is offered**: entry 27 measured layout alone moving these axes between -5.12% and +4.55% on wall clock, and separating drift from layout needs a do-nothing control this entry did not build.

#### The bucket the task existed for

`perf record -F 999`, `REXX_ENGINE=ir`, over `samples/rexxcps.rex`, both arms staged at the one fixed path, **three runs per arm**, one sitting, bucketed by self time.
The bucket is named rather than counted: `hash_one::<&[u8]>`, SipHash's own `write`, `Interp::slot_of`, `hash_one::<&SymbolId>` and `HashMap<&str, ()>::contains_key::<str>`.

| | base `9513c8b13` | head `5eaa3c8e8` |
|---|---|---|
| bucket, three runs | 7.56% / 7.42% / 7.75% | 5.34% / 4.68% / 5.24% |
| `hash_one::<&[u8]>` | 2.56% / 2.66% / 3.04% | 2.07% / 1.61% / 2.06% |
| SipHash `write` | 2.79% / 2.49% / 2.48% | 1.63% / 1.25% / 1.43% |
| `Interp::slot_of` | 1.47% / 1.55% / 1.25% | 0.73% / 1.04% / 0.92% |

The two arms' ranges do not overlap on the bucket or on any of its three largest members.
**These shares are not comparable with the plan's 8.61%/9.33% table**, which was taken at a different sitting with a different `perf` invocation; the comparison that means anything is base against head above, measured in one sitting with one method.
A share is a share of a total that itself fell 2.17%, so the absolute hashing work fell by rather more than the share difference suggests.

#### Disposition

**Accepted, and the hypothesis is confirmed by route as well as by outcome.**
The three axes whose inner loops hold a compound moved and no other axis moved by more than its own spread except `arith`, which holds none and is drift.
The bucket that did not fall last time fell.

#### What this entry does not claim

* **No time figure**, for entry 27's reason.
* **`arith`'s +0.06% is not attributed.** It is reported, and the control that would separate drift from layout was not built.
* **The stem is not the last name-keyed lookup on this path.** `stem_assign` and `replace_stem` -- a bare stem's own write and `DROP` of a whole stem -- still open with `slot_of`, and the slot they want is already in the plan's `by_symbol` under the `ExprKind::Stem` symbol's own id; a compound `DO` control variable's stem and tail pieces still resolve by name, because the whole dotted name holds the slot. Both are named in the task's report as work this plan did not take.

### Entry 31 -- corrections to entry 30: `arith` is a bound and not a cause, and where the profile shares may not be paired

**Entry 30 stated a cause where it has only a bound, and this entry says so rather than editing it.**
The rule at the top of this file, which entry 29 exists to restate, is that an entry that turned out wrong is corrected by a later entry.
Entry 30 is left exactly as committed at `73d470474`.

#### `arith`: "so this is codegen drift" is withdrawn; the bound stands

Entry 30 wrote, of `arith`'s +12,040,137 instructions:

> So this is **codegen drift** on a program the change cannot otherwise touch ... **No attribution beyond that is offered**: ... separating drift from layout needs a do-nothing control this entry did not build.

Those two sentences disagree by one notch, and the first is the one to drop.
Drift and binary layout are two candidate causes; the control that separates them was not built; so the evidence reaches "**not the change's semantics**" and stops there.
**What the evidence does support, completely:** `arith` holds no compound variable, measured two ways -- the accessor-level probe is silent on it under both engines, and every variable in `bench-programs/arith.rex` is simple with no period outside a comment -- so no line this change touches can run on it.
**What it does not support:** naming which of drift or layout produced the movement.
The task's own report carried the careful wording, "I offer no attribution beyond that", and that sentence did not reach the repository document. This entry is where it lands.

The movement itself is not in doubt and got stronger after entry 30 was written: the task's reviewer re-measured it on a second set of binaries in a second sitting at **+0.059%**, within 186 instructions of entry 30's figure, with same-binary spans of 1,681 and 712.

**`5eaa3c8e8`'s commit message carries the same sentence** ("so it is codegen drift and the cost side") and **cannot be edited**. This entry is the correction of record for it.

#### The profile shares in entry 30 may not be paired across sittings, and the caveat sits below the figures it disclaims

Entry 30 quotes the plan's `8.61%` -> `9.33%` bucket under "Cause, stated by the plan before the task", and its own base-against-head bucket table much further down, with the non-comparability caveat under the **table** rather than at the first mention.
A reader going top to bottom meets the caveat in time; a reader who pairs "went up, 8.61% to 9.33%" with "head 5.34/4.68/5.24%" has done so before reaching it.
**Neither of those numbers may be subtracted from the other.** They come from different sittings with different `perf` invocations, and a sampled share is a share of whatever else was running -- the same measurement taken with `--call-graph=dwarf` instead of without it moved the identical bucket from 7.42% to 7.56-7.75% on one arm.
The only comparison entry 30 makes, and the only one either entry supports, is **base `9513c8b13` against head `5eaa3c8e8` within one sitting**.

#### One more site for the work entry 30 names as left over

Entry 30's closing section says a bare stem write's slot is already in `by_symbol` and unused. It names `stem_assign` and `replace_stem`, which is where the lookup happens; it does not name the callers, and they do not all cost the same.
The task's reviewer found the one that matters most: `run.rs`'s `bind_control` reaches a bare stem write **on every pass of a stem-controlled `DO`**, where `control_slot` answers `None` by choice.
Measured, `do cv. = 1 to 3` on both engines, by the reviewer and again by the implementer in this fix round: `by_symbol=Some(0)`, `slot_of=0`, on every write the loop makes.
So the leftover work pays per iteration there and per statement elsewhere. **No bench axis has a stem control variable**, so nothing in entry 30's table would have shown it.

#### What entry 31 does not change

Every measured figure in entry 30 -- the seven-row instruction table, the seven-row spread table, the bucket table and its members, and the disposition -- stands, and the four figures the task's reviewer re-took independently reproduce them (`compound` -9.87%, `alloc4c` -3.16%, `rexxcps` -2.17%, `arith` +0.059%).

### Entry 32 -- a bare stem's slot: the family closes, and the obvious way to close it made every loop slower

Plan: `docs/superpowers/plans/2026-08-13-bare-stem-slot.md`, one task.
Base `50faeff88`, head `96d7a87a8`, both read back from `git log`.

**This entry changes the wrapper, for entry 27's reason and in entry 28's way**, and says so rather than letting the difference pass as a result.
The configuration block above fixes wall clock through `rexx-bench-suite`. This entry is `perf stat -e instructions:u`, **six interleaved runs per arm on every axis**, both arms staged at one fixed binary path so `argv[0]` is byte-identical.
**No wall-clock figure is claimed by this entry**, and no wall-clock run was taken.

#### Cause, stated by the plan before the task

Three comments in the tree said a bare stem write has no slot to resolve ahead of it. Measured on both engines, it does: `Plan::bind` puts an `ExprKind::Stem` symbol's spelling and its id on one slot, and `stem_assign` then hashes that same spelling to arrive at that same number. Three sites still did it -- `stem_assign`, `replace_stem`, and `bind_control`'s stem arm, which pays **per iteration** of a stem-controlled `DO`.

#### Two axes written for this, and why neither is joining `bench-programs/`

No axis in this record has a stem-controlled `DO`, so the plan asked for a workload before the fix. Two were written, and their text is here rather than in `bench-programs/` for the reason entry 29's `INTERPRET` program is:

```rexx
n = 3000000                    n = 6000000
do zs. = 1 to n                do i = 1 to n
  nop                            zs. = i
end                            end
say 'done'                     say 'done'
```

A stem-controlled `DO` is close to nonexistent in real Rexx -- the control variable is a whole stem and the loop writes its *default* every pass -- so a permanent axis would give it a vote in every later entry that its share of real programs cannot justify. A bare stem write in a loop already has an axis: `samples/rexxcps.rex` writes `avar.=1.0''loop` fourteen times an iteration. Both were kept as diagnostic instruments for this task and are recorded here so they can be rebuilt.

#### The result the task exists for is the one it nearly shipped

The first design forwarded the loop's kept `control_slot` answer from `bind_control`'s stem arm into `Interp::assign_expr_target`, which is what the plan describes. It replaces a compile-time `None` at that call site with a value, and that alone costs **2 instructions on every pass of every controlled loop** -- including loops whose control variable is a plain simple variable and which never enter the stem arm. Why the generated code changes was not established; the effect was.

| axis | base | first design | |
|---|---:|---:|---:|
| `emptyloop`, 25,000,000 passes | 27,150,813,233 | 27,200,813,245 | **+50,000,012** |
| `varlookup`, 19,000,000 passes | 44,840,839,859 | 44,878,839,895 | **+38,000,036** |
| `strings`, 3,000,000 passes | 44,928,791,773 | 44,934,791,309 | +5,999,536 |
| `stemloop` | 11,402,819,601 | 9,695,819,491 | -14.97% |

Exactly +2 a pass on three different programs, thousands of times each axis's own span. Isolated by partial revert, three runs each: reverting that **one line** puts `emptyloop` back on base exactly (27,150,813,214). Recomputing the slot inside the stem arm instead is worse still, `emptyloop` +100,000,000.

**What shipped instead takes the slot from the plan's own `CompoundName` entry**, so that call site keeps its literal `None`. `Plan::bind` already assigned the slot; a stem-shaped name has no stem half distinct from itself, so it now records that slot on the entry it was already building, and `assign_expr_target`'s stem arm and the loop's re-test read it back. It works on **both** engines, where the compiled-op route would have worked on one.

#### Measured movement

Minimum of six interleaved runs per arm.

| axis | base `50faeff88` | head | difference |
|---|---:|---:|---:|
| `stemloop` | 11,402,819,545 | 9,743,819,810 | **-14.55%** |
| `stemwrite` | 13,379,958,659 | 11,729,958,718 | **-12.33%** |
| `rexxcps` | 25,298,108,095 | 25,259,319,732 | **-0.153%** |
| `varlookup` | 44,840,840,190 | 44,802,839,869 | -0.085% |
| `compound` | 25,052,694,664 | 25,028,455,741 | -0.097% |
| `strings` | 44,928,791,694 | 44,913,791,298 | -0.033% |
| `alloc4c` | 8,189,876,434 | 8,183,867,131 | -0.073% |
| `arith` | 20,416,193,212 | 20,413,693,451 | -0.012% |
| `emptyloop` | 27,150,812,848 | 27,150,812,765 | -83 instructions |

Every `rexxcps` run of both arms self-calibrated to `100 x 100`. Every axis produces byte-identical stdout **and** stderr and exit 0 under both binaries on **both** engines, `rexxcps` excepted in its own two timing lines.

#### The instrument's own spread, measured before any of the small figures were read

| axis | same-binary span, base | same-binary span, head |
|---|---:|---:|
| `stemloop` | 1,573 | 835 |
| `stemwrite` | 1,227 | 834 |
| `compound` | 1,599,903 | 2,598,819 |
| `alloc4c` | 117,496 | 92,743 |
| `rexxcps` | 6,428,894 | 7,160,383 |
| `arith` | 924 | 746 |
| `strings` | 1,025 | 1,354 |
| `varlookup` | 495 | 1,576 |
| `emptyloop` | 1,348 | 752 |

`emptyloop`'s -83 is inside both spans and is a **bound, not a difference**. Every other arm pair is non-overlapping.

#### What is attributable, and what is only a bound

An accessor-level probe -- `stem_assign_at` and `read_stem_at` instrumented -- was run over every axis on both engines, which answer identically: `stemloop` 6,000,001 bare-stem operations, `stemwrite` 6,000,000, `rexxcps` 140,000, `compound` 1, and `alloc4c`, `arith`, `strings`, `varlookup` and `emptyloop` **zero**.

* **`stemloop` and `stemwrite` are this change's semantics**: 6,000,001 and 6,000,000 name resolutions removed.
* **Five axes execute no bare-stem operation at all and still moved**, by up to -38,000,321 instructions, thousands of times their spans. That is not this change's semantics, and **no attribution beyond that is offered** -- entry 31 is the standing correction for naming drift or layout without the control that separates them, and this task did not build one either. It happens to be favourable; that is luck, not a result.
* **`rexxcps` is only partly attributable.** It executes 140,000 bare-stem writes and moved -38,788,363 with non-overlapping arms. 140,000 removed resolutions of a five-byte name cannot be 38 million instructions, so most of that figure is the same drift. **The direction and the bound are claimed; a per-write cost is not.**

#### The bucket, and what is left in it

`perf record -F 999`, `REXX_ENGINE=ir`, over `samples/rexxcps.rex`, both arms at the one fixed path, three runs per arm, one sitting, bucketed by self time.

| | base `50faeff88` | head |
|---|---|---|
| bucket | 5.55% / 6.50% / 5.99% | 6.25% / 5.95% / 7.14% |
| `hash_one::<&[u8]>` | 1.72 / 2.30 / 1.71 | 2.02 / 1.65 / 2.62 |
| SipHash `write` | 2.13 / 1.81 / 1.75 | 1.90 / 2.20 / 1.94 |
| `Interp::slot_of` | 0.57 / 1.06 / 0.98 | 1.02 / 0.74 / 1.15 |
| `hash_one::<&SymbolId>` | 0.90 / 0.88 / 0.85 | 0.83 / 0.90 / 0.93 |

**The bucket did not fall, and the arms overlap completely on it and on every member.** The instruction counter reads `rexxcps` down 38.8 million with non-overlapping arms; the sampled share cannot resolve that, because the within-arm spread on this instrument is over a percentage point and the work removed is 140,000 resolutions out of 25 billion instructions. **The sampled share is the wrong instrument for what is left**, which is a finding about the instrument rather than about the change.

**What is left was then measured directly**, by instrumenting `Interp::slot_of` at head and running `rexxcps`: **3,080,203 calls, and not one is a stem or a compound.**

| what | calls |
|---|---:|
| `PARSE` targets (`A1`..`A4`, `B1`..`B3`, `C1`..`C3`, `P1`..`P8`, `V1`, `V2`) | 2,800,003 |
| `RESULT`, through `extra` | 140,199 |
| `SIGL`, through `extra` | 139,999 |
| one-offs at startup | 2 |

**So the stem-and-compound family is finished on this path**, and its successor is named and measured. `parse_template.rs` passes `None` for every `PARSE` target because nothing promotes the instruction -- and with that arm instrumented, **2,800,003 of 2,800,003** `PARSE` writes on `rexxcps` come back `by_symbol=Some(n)` with `slot_of` computing that same `n`, on both engines. That is 91% of what is left and it is this family's finding one instruction further along. The rest is `RESULT`/`SIGL`, which start from a run-time byte string with no symbol to hang a slot on; `hash_one::<&SymbolId>` at 0.83-0.93%, the *precomputed* side, since `by_symbol` and `Code::slots` are `HashMap<SymbolId, usize>` and every slot this family saves is bought with a SipHash of a `u32`; and the stem's own tails map, which is the data structure and not a symbol lookup.

**The lesson from this entry applies directly to the successor**: `parse_template.rs`'s call site passes a literal `None`, and replacing a literal `None` with a value at a call site is precisely what cost 2 instructions a pass here. Whoever takes it should measure `emptyloop` and `varlookup` before believing the win.

#### Disposition

**Accepted.** The two axes the change can reach fall 14.55% and 12.33%, no axis moved up, and the design that would have moved two axes up was measured and rejected rather than shipped.

#### What this entry does not claim

* **No time figure**, for entry 27's reason.
* **No cause for the five compound-free axes' movement**, for entry 31's reason. A do-nothing control still does not exist, and this is the second entry in a row to say so.
* **The bucket is not claimed to have fallen.** It is claimed to be unresolvable at this instrument's resolution, with the direct count of `slot_of` calls given instead.
* **`rexxcps`' -38.8 million is not decomposed** into removed resolutions and drift.

### Entry 33 -- corrections to entry 32: one multiple that does not hold, one caption, and a span this task measured and did not carry

**Entry 32 overstated one quantifier and mis-captioned one table, and this entry says so rather than editing it.**
The rule at the top of this file, which entries 29 and 31 exist to restate, is that an entry that turned out wrong is corrected by a later entry.
Entry 32 is left exactly as committed at `866d07d1b`.
Found by this task's review, and the arithmetic below was redone rather than taken from it.

#### "thousands of times their spans" does not hold for every axis it was said of

Entry 32 wrote, of the axes executing no bare-stem operation:

> **Five axes execute no bare-stem operation at all and still moved**, by up to -38,000,321 instructions, thousands of times their spans.

Against the spans entry 32's own table gives, the multiple of the larger of each axis's two spans is:

| axis | move | spans | multiple |
|---|---:|---:|---:|
| `varlookup` | -38,000,321 | 495 / 1,576 | 24,112x |
| `strings` | -15,000,396 | 1,025 / 1,354 | 11,079x |
| `arith` | -2,499,761 | 924 / 746 | 2,705x |
| `alloc4c` | -6,009,303 | 117,496 / 92,743 | **51x** |
| `emptyloop` | -83 | 1,348 / 752 | **inside the span** |

So the quantifier holds for three of the five and not for `alloc4c`, whose move is fifty times its span rather than thousands, and not for `emptyloop`, which entry 32 elsewhere correctly calls a bound.
**Fifty times a span is still outside it**, so nothing in entry 32's disposition moves: those axes still moved by more than the instrument's resolution, and the attribution entry 32 declines to make is still declined.
What is withdrawn is the word "thousands" as a claim about all of them.

#### `strings` moved by less than a span this task itself measured, and entry 32 carries neither

The `strings` row is the one worth having.
Entry 32's spread table gives that axis 1,025 and 1,354, from the interleaved sitting, and its move of -15,000,396 is 11,079 times that.
But **this task's own Step 1 base measurement, six runs of the one base binary, recorded a same-binary span of 36,000,462 on `strings`** -- larger than the move -- and entry 32 does not carry that figure or the caveat that goes with it.
The large span is one run out of six coming back 36 million instructions high, the interference signature entry 29's reviewer and entry 30 both recorded on that axis.
**So `strings` carries a bound and not a difference**, and its -15,000,396 should not have been grouped with `varlookup`'s.
The honest statement is that `strings` is the axis on which this instrument has twice produced an excursion larger than any effect measured on it, and no signed figure for it survives.

#### The spread table's caption names the wrong sitting

Entry 32 captions its spread table "measured before any of the small figures were read".
The figures in that table are the **Step 5** spans, from the same six interleaved rounds the differences come from.
The spans measured before anything changed are the **Step 1** ones, taken on the base binary alone, and they are not in the entry: `stemloop` 1,236, `stemwrite` 1,076, `compound` 2,901,211, `alloc4c` 50,817, `rexxcps` 6,448,443, `arith` 3,216, `strings` 36,000,462, `varlookup` 1,508, `emptyloop` 622.
Both sets were measured, and the Step 1 set was measured first; the caption attaches the second set's provenance to the first set's virtue.
The discipline the caption is reaching for -- measure the instrument's spread before reading a difference off it -- was followed, and the evidence for that is the Step 1 row above, not the table the caption sits on.

#### What entry 33 does not change

Entry 32's instruction table, its accessor-probe attribution, its bucket table, its direct `slot_of` count, its `PARSE`-target successor and its disposition all stand.
The `alloc4c` and `strings` rows are the only figures whose reading changes, and neither carries any part of the disposition.

### Entry 34 -- what entry 33's own review found in entry 33

Correcting entry 33 by appending, in the shape entry 33 used on entry 32.
Found by the scoped re-review of `866d07d1b..851b18fce`, verified against the tree at `851b18fce` by the controller.
No measurement is withdrawn and no disposition moves.

#### The withdrawn quantifier came back one paragraph later

Entry 33 withdraws entry 32's "thousands of times their spans" for `alloc4c` and `strings`, and then closes with "those axes still moved by more than the instrument's resolution".
That sentence covers `strings`, whose move of -15,000,396 is smaller than the 36,000,462 span entry 33 itself restores two paragraphs above it.
**`strings` supports a bound and nothing else, in entry 33's closing sentence as much as in entry 32's table.**
What survives for the rest of the group is that their moves are outside their own spans, which is what the multiples table shows and what the closing sentence should have said.

#### A set count, in the entry that removed set counts

"So the quantifier holds for three of the five" counts a set whose members the table directly above it names.
The count carries nothing the table does not, and it was written in the round that took the same construction out of the plan file.

#### The plan file's counts were not all removed, and the fix section says they were

`2026-08-13-bare-stem-slot.md` still read "The first two were named by the implementer; the third by its reviewer" after the round that reported the plan file's counts gone.
Corrected in the plan file at this entry's commit; the completeness claim in the fix-round report is what was wrong, not the correction it describes.

#### An equality that was never measured

"Reverting that one line puts `emptyloop` back on base exactly" appears in entry 32, in `control_slot`'s doc and in the plan file's Status block.
The partial-revert table it cites reads **27,150,813,214 against a base of 27,150,813,500**, a difference of 286 -- inside that axis's own span, and so indistinguishable from base rather than equal to it.
The reading does not change: the +2 a pass is attributed to that one line either way.
**The word does.** An exact equality is a stronger claim than the instrument can make, and this entry's family has spent four rounds learning that the sentence beside a correct number is where the false claims live.
Corrected in `control_slot`'s doc and in the plan file; entry 32's copy stands where it is, corrected here.

#### Where this leaves the family

Approved at `851b18fce`, gates re-run unpiped by the controller: `cargo fmt --all --check` 0, `cargo clippy --workspace --all-targets -- -D warnings` 0 with no warning lines, `cargo test --workspace --no-fail-fast` rc 0 at 1483 passed, 0 failed, 4 ignored.
Entry 32's successor stands unchanged: `PARSE` targets, whose call site in `parse_template.rs` passes the literal `None` that this task measured the price of replacing.

### Entry 35 -- the clause's line, from a table: the first structural unit, and a spike conclusion that did not survive its own fix

Plan: `docs/superpowers/plans/2026-08-13-clause-line-table.md`, one task, Unit 1 of `2026-08-13-phase-4g-structural.md`.
Base `9b2d416d3`, head `5b3be9536`, both read back from `git log`.

**This entry changes the wrapper, for entry 27's reason and in entry 32's way**, and says so rather than letting the difference pass as a result.
The configuration block above fixes wall clock through `rexx-bench-suite`; this entry is `perf stat -e instructions:u`, six interleaved rounds per arm on every axis, both arms staged at one fixed binary path so `argv[0]` is byte-identical, from a fresh empty directory, minimum of each arm quoted.
**No wall-clock figure is claimed and no wall-clock run was taken.**

#### Cause, stated by the plan before the task

`ProgramSource::line_of` is a `partition_point` over the line starts, and `Interp::enter_stepped_clause` asked it for the answer on every stepped clause **unconditionally**, because `SIGL` has to stay correct whether or not `TRACE` is on.
A loop's header asks again on every pass.
The fix is the move `Plan::indents` already is: a constant of the source text computed once by the upfront pass that already walks the body.

#### What the search was costing, counted before anything changed

`Interp::clause_line`'s call into `line_of` instrumented with a counter, release build, both engines, at base.
The two engines answer **identically on every axis**, which is the part worth having: the compiled engine's promoted clauses do not reach `Interp::step`, and they reach `enter_stepped_clause` anyway.

| axis | searches at base | body lines |
|---|---:|---:|
| `emptyloop` | 50,000,005 | 7 |
| `varlookup` | 57,000,006 | 8 |
| `strings` | 18,000,007 | 12 |
| `compound` | 15,001,011 | 14 |
| `arith` | 4,000,006 | 14 |
| `alloc4c` | 4,000,006 | 51 |
| `rexxcps` | 10,161,437 | 198 |

**No axis in this record executes none of it**, so this entry has no control axis and offers none.

#### Measured movement

| axis | base `9b2d416d3` | head `5b3be9536` | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `emptyloop` | 27,150,609,938 | 25,150,609,522 | -2,000,000,416 | **-7.366%** | 550 | 1,521 |
| `varlookup` | 44,802,636,286 | 42,408,615,813 | -2,394,020,473 | **-5.343%** | 1,296 | 647 |
| `rexxcps` | 25,259,112,106 | 24,296,505,166 | -962,606,940 | **-3.811%** | 11,497,720 | 4,212,461 |
| `alloc4c` | 8,183,701,509 | 7,882,695,240 | -301,006,269 | **-3.678%** | 55,696 | 85,333 |
| `compound` | 25,027,491,124 | 24,233,277,999 | -794,213,125 | **-3.173%** | 1,679,877 | 2,822,324 |
| `strings` | 44,913,587,939 | 43,950,588,042 | -962,999,897 | **-2.144%** | 1,317 | 1,005 |
| `arith` | 20,413,489,949 | 20,203,990,984 | -209,498,965 | **-1.026%** | 681 | 1,914 |

Every arm pair is non-overlapping, by at least a factor of eighty against the wider of its two spans.
Every `rexxcps` run of both arms self-calibrated to `100 x 100`, checked on each arm's own `Averaged:` line.
**No axis moved up**, and no figure here is a bound: each axis executes the changed code and each moved past its own spread.

The measurement was taken twice, because a comment-only correction after the first sitting changed the binary's bytes and this project has measured layout alone moving these axes.
The two sittings agree to three decimal places on six of the seven axes and to 0.004 percentage points on `compound`; the table above is the second, taken with the binary that was committed, `sha256` checked against it.

#### The spike's design conclusion did not survive the fix it justified

The plan states, as the finding the unit turns on, that **the search depth is not the cost** -- from probe arms saving 105.0 instructions per pass on a 7-line program against 52.8 per clause on a 198-line one.
Dividing each axis's saving by its own count above says the opposite:

| body lines | saved per removed search |
|---:|---:|
| 7 | 40.0 |
| 8 | 42.0 |
| 12 | 53.5 |
| 14 | 52.9 and 52.4 |
| 51 | 75.3 |
| 198 | 94.7 |

Monotone in the body's line count and more than doubling across the range.
The spike's contrary reading came from its `rexxcps` arm, which the plan **itself** disowns as contaminated: both probe arms returned deliberately wrong line numbers, which reach downstream consumers of the clause line.
The plan disowned that arm's *table* and kept its *design conclusion*, and the conclusion was the part built on it.

**The decision is unaffected and the reasoning for it is now different.**
A table is not preferred to a faster search because the search is shallow; it is preferred because a search that is not made costs neither its call nor its depth, which bounds anything a faster search could return.
A correct decision shipped with a reason that does not hold is a shape this record has caught before.
What is new here is where the false reason was: in the plan the task was given, not in the change the task wrote.

#### The tripwire, and what it is worth measured rather than assumed

`Plan::line_at` carries a permanent `debug_assert_eq!(cached, source.line_of(span.start))`.
Whole workspace, `--no-fail-fast`, under `memcap 8G`: **1487 passed, 0 failed, 4 ignored, zero firings.**
Inverted to `debug_assert_ne!`, it fails **403 tests across 400 distinct names**, so the zero is a live zero.

| mutation | result | distinct failing names |
|---|---|---:|
| the accessor tripwire inverted | 1084 passed, 403 failed | 400 |
| the index/instruction pairing assert inverted | 1085 passed, 402 failed | 399 |
| **T1** the table filled one line high | 1083 passed, 404 failed | 401 |
| **T1'** the same, tripwire deleted | 1453 passed, 34 failed | 34 |
| **T2** the table never filled at all | 1485 passed, **2** failed | 2 |
| **T3** the table consulted before `clause_line_override` | 1486 passed, **1** failed | 1 |

* **T1 minus T1' is 367 distinct names** that catch a wrong line table **only** because the tripwire is there. It is a net catcher by a wide margin, on this source as on the two the stem-and-compound family measured it on.
* **T1' still fails 34**, `both_engines_agree_on_every_case_file` and `both_engines_agree_on_every_branch_shape` among them, so a wrong line is observable to the differential gates without the tripwire -- it is the *margin* the tripwire buys, not the whole catch.
* **T2 is caught by this task's own two tests and nothing else**, which is the correct shape: a table that misses falls back and is right. Only a table that lies is dangerous.
* **T3 is caught by this task's own wiring test and nothing else.** The override outranking the table has no other witness in the workspace, and the state it guards -- a body carrying a plan stepped while the override is in force -- was **not reached** by anything measured below. The test guards a future arrangement rather than a live defect, and says so.

Every mutation was snapshotted from the live tree immediately before it was applied, restored, and verified against the `git diff` hash recorded once when the change was complete -- a derivation from `HEAD` plus the tree rather than from the backup the restore came out of.
**That check caught a real leak**: a probe had modified `bin/rexx-run.rs`, which was clean when its snapshot was taken and so was never in it, and the restore left the file modified. A same-snapshot comparison would have passed.

#### The `BodyKey` question, answered by running

A table of line numbers is valid only for the source it was built from, and `plan_for` caches by `BodyKey`.
`Plan` was given a probe field recording the `ProgramSource` address it was built from, and `plan_for` was made to assert on a cache hit that the source it is being asked with is that same one.

* **381 corpus, bench and oracle-sample programs, both engines, plus the whole workspace suite: zero mismatches**, and `Interp::programs` never held more than one program on any of them.
* **The probe is live**: inverted to `assert_ne!` it fires on a program that calls one `::ROUTINE` twice, on one that calls a routine from inside an `INTERPRET`, and on a nested-`INTERPRET`-plus-loop shape.
* **The corpus is a weak witness for this and the numbers say so.** Inverted, only **2 of those 381 programs** and **7 of 1487 tests** reach the plan-cache hit path at all. The hand-written probes are what exercised it.
* The two production `plan_for` callers take body, symbols and source out of one `Rc<Program>` reached through the id the key carries, so the pairing holds by construction as well as by measurement -- but `::REQUIRES` is a Phase 5 gap and an external routine file a Phase 7 one, and **both are routes by which a second program could be loaded**. The tripwire is what will still be standing when either lands.

#### The profile

`perf record -F 999`, `REXX_ENGINE=ir`, `samples/rexxcps.rex`, both arms at the one fixed path, three runs per arm, one sitting, self time.

| | base | head |
|---|---|---|
| `Interp::run_ops::<false>` | 12.41 / 12.00 / 12.25% | 9.02 / 9.44 / 10.12% |
| `Interp::step_in_temps_frame` | 2.56 / 2.77 / 2.13% | 1.10 / 1.51 / 1.21% |

Both fall with non-overlapping ranges.
**`ProgramSource::line_of` appears as a symbol in neither arm** -- it is inlined into both -- so the sampled instrument could never have named this cost, and the two driver symbols are where it was living. That is a finding about the instrument as much as about the change: the phase document attributes about 20% of `rexxcps` to "the driver", and a per-clause search inside it was invisible to the tool that produced that share.

#### What a program observes

Base and head binaries compared byte for byte on stdout, stderr and exit status, **both engines**, over the corpus, `bench-programs/` and the oracle's `samples/` tree.
Identical everywhere except `samples/rexxcps.rex`, whose `Performance:` line is the clauses-per-second figure the program exists to print.
Two infinitely recursive samples differed only in the PID inside `memcap`'s own kill message, which is the harness and not the interpreter; filtered, they are identical at rc 137.
`tests/ir_dual.rs` and `tests/corpus.rs` are green at head. **This unit spends none of the phase's divergence licence.**

#### Disposition

**Accepted.** Seven axes down, none up, the smallest move eighty times its own spread, and the largest -- 7.366% on the clause-dispatch floor -- above the plan's own honest expectation of about 5.6%.

#### What this entry does not claim

* **No time figure**, for entry 27's reason. `rexxcps`' own printed clauses-per-second line moved and is reported above as an *output* difference, not as a measurement.
* **No control axis.** Every axis executes the changed code, so there is nothing here that separates this change's semantics from drift the way entry 31 asks for. What stands in for it is that the saving is monotone in the search depth removed, on seven axes, which drift has no reason to be.
* **No claim that a body with a plan is unreachable while `clause_line_override` is in force.** It was not reached by anything measured; that is not the same statement, and this project has been wrong about the difference before.

### Entry 36 -- the map the slot family was buying its answers from

Plan: `docs/superpowers/plans/2026-08-13-parse-target-slots.md`, Task 1 of two.
Base `8c54b17cd`.
Taken by the controller directly rather than by an implementer, after two dispatched agents stopped without writing anything; that is a note about this session's tooling and not about the change.

#### Cause

Every application in the stem-and-compound family replaced a hash of a name's bytes with a slot the plan already held.
The slot was then read out of `Plan::by_symbol`, a `HashMap<SymbolId, usize>`, so **each saved byte-string hash was bought with a SipHash of a `u32`**.
Entry 32 measured that side at 0.83-0.93% of `rexxcps` self time, the same order as `Interp::slot_of` itself.
`plan.rs` has carried the fix as a comment since `180875a9`: `SymbolId` is a dense, table-local, zero-based index, `SymbolId::index()` exposes it, and the field wants an array.

#### The change

`Plan::by_symbol` is now `Vec<Option<usize>>` sized by the body's symbol table and indexed by `SymbolId::index()`, in the shape `Plan::compounds` beside it already had.
`Code::slots` is a slice rather than a map reference.
Both resolutions go through accessors: `Code::slot_for` at run time, `Plan::slot_for_symbol` at compile time.

**The compiler found a site that reading for it did not.**
The surface was mapped by reading first, and the build then rejected a `debug_assert!` inside a `matches!` guard at `ir/drive.rs` that no search had surfaced.
That is the standing instruction working on its first application: change the field, build, and take the error list as the answer.

#### The tripwire, and why it checks one way

`Code::slot_for` asserts that when it answers `Some(at)`, `Plan::names` maps that symbol's own spelling to that same `at`.
`Plan::bind` fills both from one `slot_for` call, so the name map is an independent recomputation rather than a second copy.

**A `None` is safe and a wrong `Some` is not**, which is why the check is one-directional.
A `None` falls through to `Interp::slot_of`, which resolves the name and answers the same number.
A wrong `Some` would silently address another variable on every read and write of the name, and no program that does not already know the answer could notice.
The reverse implication does not hold and its failing would not be a defect: `note_compound_name` puts a stem and its variable tail pieces in `names` without binding their ids, so `say v.i` leaves a name carrying a slot that no symbol's entry does.

Inverted, the assert reddens **191 tests across 124 distinct names**; restored and verified with `sha256sum -c`, the workspace is green at 1487 passed, 0 failed.

#### Measurement

`perf stat -e instructions:u`, six interleaved rounds per arm, both arms staged at one fixed binary path, from a fresh empty directory, minimum of each arm.

| axis | base `8c54b17cd` | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 43,950,587,952 | 41,904,585,446 | -2,046,002,506 | **-4.655%** | 874 | 685 |
| `alloc4c` | 7,882,703,252 | 7,727,714,204 | -154,989,048 | **-1.966%** | 12,018,171 | 64,260 |
| `rexxcps` | 24,296,501,015 | 23,989,877,961 | -306,623,054 | **-1.262%** | 15,950,319 | 7,577,086 |
| `varlookup` | 42,408,615,575 | 42,123,633,031 | -284,982,544 | -0.672% | 1,694 | 1,487 |
| `compound` | 24,232,438,164 | 24,157,429,618 | -75,008,546 | -0.310% | 2,870,174 | 1,680,096 |
| `emptyloop` | 25,150,609,391 | 25,075,608,842 | -75,000,549 | -0.298% | 1,642 | 748 |
| `arith` | 20,203,990,946 | 20,178,736,951 | -25,253,995 | -0.125% | 858 | 931 |

**No axis moved up**, and every difference is outside both of its arms' spans -- the tightest ratio is `rexxcps` at 19.2 times its wider span, and `alloc4c` at 12.9.

**Behaviour**: every axis produces identical stdout, identical stderr and identical exit status under both arms, on **both engines**, checked with the engine names the binary actually accepts.
The first attempt at that check used a spelling the binary rejects, and both arms failed identically at rc 2 -- a comparison that cannot fail, caught only because the exit status was read.

#### What this entry does not claim

**No control axis exists.** Every axis resolves variables, so nothing here executes none of the changed code, and entry 31's ask cannot be discharged by an axis that sees nothing.
The differences are stated as differences because each is outside its own spans by more than an order of magnitude, not because drift has been excluded.

**Why `strings` is four times the next axis is not established.** It is the axis that reads the most distinct variables per clause, which is a hypothesis and not a measurement, and this entry does not decompose it.

#### What is left

Task 2 is the `PARSE` target's own slot, which is what this plan exists for: `parse_template.rs` passes a literal `None` for 2,800,003 of the 3,080,203 `slot_of` calls entry 32 counted on `rexxcps`.
That call site now reads a slice rather than a map, so the win Task 2 measures is the removal of the name hash alone.

### Entry 37 -- the `PARSE` target's own slot, and the first real control group

Plan: `docs/superpowers/plans/2026-08-13-parse-target-slots.md`, Task 2 of two.
Base `f803fbf96`, which is entry 36's head.

#### The change

`parse_template.rs`'s target arm passed a literal `None` for the slot and said in its own comment why: no compiler resolves a `PARSE` target, because nothing promotes the instruction.
True about the compiler and false about the plan.
`Plan::build` walks every instruction and binds the names each one writes, `note_parse` included, so the answer was already there and was being recomputed from the name on every firing.
The arm now reads it, **for a `Variable`-shaped target only**: a stem-shaped or compound-shaped target takes its slot from the plan's `CompoundName` entry inside `assign_expr_target`, whose stem and compound arms assert this argument is `None`.
Those asserts were landed by entry 33's fix round against exactly this call site, and they did not fire.

#### Six axes that execute none of it, which is what makes this the cleanest measurement of the family

Every earlier entry in this family had to say that no control axis existed, because every axis resolves variables.
This change is reached only through `PARSE`, `ARG` and `PULL`, and of the workloads only `samples/rexxcps.rex` contains any.

`perf stat -e instructions:u`, six interleaved rounds per arm, both arms staged at one fixed binary path, from a fresh empty directory, minimum of each arm.

| axis | base `f803fbf96` | head | difference | | span base | span head |
|---|---|---:|---:|---:|---:|---:|
| `rexxcps` | 23,989,868,293 | 23,242,134,238 | -747,734,055 | **-3.117%** | 6,186,396 | 6,457,240 |
| `emptyloop` | 25,075,609,140 | 25,075,609,257 | +117 | bound | 622 | 711 |
| `varlookup` | 42,123,633,595 | 42,123,633,402 | -193 | bound | 995 | 651 |
| `arith` | 20,178,736,547 | 20,178,736,720 | +173 | bound | 1,316 | 1,174 |
| `strings` | 41,904,584,463 | 41,904,584,940 | +477 | bound | 1,851 | 865 |
| `compound` | 24,157,809,748 | 24,158,269,459 | +459,711 | bound | 2,980,540 | 2,759,654 |
| `alloc4c` | 7,727,666,884 | 7,727,682,076 | +15,192 | bound | 131,951 | 65,088 |

**Every control axis moved less than its own same-binary spread**, so each is a bound and none is a difference, and the layout drift earlier entries had to leave unattributed does not appear here at all.
`rexxcps` moved 115 times its wider span.

**The trap this plan was written around did not recur.** Entry 32 measured a literal `None` replaced by a value at `bind_control`'s call site costing 2 instructions on every pass of every controlled loop, including loops that never entered the changed arm. `emptyloop` and `varlookup` were gates on this task for that reason, and both sat inside their spreads.

#### The count afterwards, and what is left

`Interp::slot_of` instrumented at head and run on `samples/rexxcps.rex` at `count=100`/`averaging=100`: the count lands between 280,000 and 281,000, against **3,080,203** when entry 32 measured it.
Entry 32 predicted the remainder by naming its parts -- `RESULT` at 140,199 and `SIGL` at 139,999, plus the startup one-offs, which is 280,200.
**The prediction and the measurement agree**, so what is gone is precisely the `PARSE` targets and nothing else went with them.

What remains is the shape entry 32 called different in kind: `Interp::set_sigl` and the `CALL` return path reach `assign_by_name` from a run-time byte string with no symbol to hang a slot on.
Removing those means giving the activation dedicated fields or well-known slots, which is not a member of this family.

#### Behaviour

`samples/rexxcps.rex` under both arms and **both engines** produces identical output once its own timing lines are excluded, identical stderr and identical exit status.
Every other axis was compared byte for byte under entry 36 and is untouched here.

#### What this entry does not claim

The saving divided by the resolutions removed is not quoted, because the count at this base was measured only at head; entry 32's 2,800,003 was taken at a different commit, and dividing one entry's numerator by another's denominator is the arithmetic this record has already corrected once.

### Entry 38 -- two copies taken for lines that do not print

Base `08c3e3d64`.
The first work directed by `heaptrack` rather than by `perf`, following the Unit 4 premise check recorded in the phase document.

#### Cause

Both sites build something on every execution and hand it to a formatter that returns at once unless a trace setting is on.

* `assign_expr_target`'s compound arm copied the symbol's spelling with `to_vec()` and copied the stem name, then joined stem to tail key to make a resolved name, for `trace_compound_name` -- which returns unless intermediates are on. The *value* beside it was already gated: `rendered` arrives as `None` when no line would print it, and `trace.rs` carries the reasoning for the value half. The name half never got it.
* `Interp::condition_value` rendered the condition's value and copied it with `to_vec()` on **every condition evaluated**, for `trace_result`/`trace_keyword` -- both of which return unless `results` is on. `heaptrack` named this the largest single allocation site in the interpreter.

**Neither copy in the compound arm was necessary at all**, which is the part reading for it would have missed. `Code::stem` and `SymbolTable::name` answer with the lifetime of the `Code` -- the program -- and not of the `Interp`, so both survive the `&mut self` calls that appeared to force the copies. The `Variable` arm above has always passed its name borrowed.

In `condition_value` the copy is real but conditional: it exists only because `to_text` borrows `self` while the formatters need it mutably, so off the tracing path the borrow suffices and the answer is decided from it. The decision now happens before `pop_frame` rather than after, which is what lets that arm hold the borrow; it touches neither `self` nor the roots, so no answer and no lifetime changes.

#### Allocations, which is the instrument this entry is measured on first

`heaptrack`, `samples/rexxcps.rex` at `count=20`/`averaging=20`, which is 400,000 clauses.

| | allocations | temporary |
|---|---:|---:|
| base | 29,775,401 | 10,902,311 |
| head | 24,475,400 | 7,822,310 |
| | **-17.80%** | **-28.25%** |

#### Instructions

`perf stat -e instructions:u`, six interleaved rounds per arm, both arms staged at one fixed binary path, from a fresh empty directory, minimum of each arm.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `compound` | 24,157,811,838 | 21,827,976,772 | -2,329,835,066 | **-9.644%** | 1,678,737 | 1,710,667 |
| `alloc4c` | 7,727,671,785 | 7,372,628,404 | -355,043,381 | **-4.594%** | 59,970 | 86,032 |
| `rexxcps` | 23,242,133,514 | 22,619,173,459 | -622,960,055 | **-2.680%** | 4,086,394 | 8,145,126 |
| `varlookup` | 42,123,633,699 | 42,009,633,877 | -113,999,822 | -0.271% | 997 | 572 |
| `arith` | 20,178,736,662 | 20,171,236,302 | -7,500,360 | -0.037% | 1,109 | 1,814 |
| `strings` | 41,904,584,654 | 41,859,584,939 | -44,999,715 | **bound** | 1,098 | 126,000,263 |
| `emptyloop` | 25,075,608,884 | 25,075,609,575 | +691 | **bound** | 947 | 754 |

**`strings` is a bound and not a difference**, and the reason is its own arm rather than the change: one head run came in 126 million high, which is larger than the difference itself. That axis has carried this interference signature in entries 29, 30, 32 and 36. The direction is favourable and nothing is claimed from it.

`emptyloop` moved inside both spreads, which is what an axis with no compound write and no condition evaluated should do.

#### Behaviour

Every axis produces identical stdout, identical stderr and identical exit status under both arms on **both engines**.

**And the paths this change gates were exercised rather than assumed.** A `TRACE R` program with a compound write and an `IF` inside a loop -- the two gated sites together -- produces identical output on both engines, and that output is **identical to the oracle's**, byte for byte.

#### What this entry does not claim

No attribution of the instruction differences to the removed allocations specifically. Every moving axis executes both changed sites, so there is no control axis here, and `compound`'s -9.6% is not decomposed into the two cuts.

### Entry 39 -- a small integer's digits, written where they are wanted

Base `d73b24310`, entry 38's head.
The second piece of `heaptrack`-directed work, from re-attributing the profile at that head rather than reusing the one entry 38 started from.

#### Cause

`Interp::to_text` answers `Cow::Borrowed` for `Body::Text` and for `Body::Num`, whose rendering it caches on the object.
A tagged small integer has no object to borrow from, so that arm builds a `String` and hands it back owned.
A caller that only wants the bytes appended somewhere therefore allocated that `String`, copied it, and dropped it.

`Interp::tail_key` is exactly such a caller and it is on the compound path: `key.extend_from_slice(&self.to_text(value))`, once per tail piece of every compound reference.
`samples/rexxcps.rex` writes `acompound.key1.loop` in its innermost loop, whose tails are small integers, so that was the whole of that path's allocation.

`Interp::write_text` renders the digits into the caller's buffer instead, through `unsigned_abs` so `i64::MIN` has a form, and falls back to `to_text` for everything else.

#### Allocations

`heaptrack`, `samples/rexxcps.rex` at `count=20`/`averaging=20`, 400,000 clauses: **24,475,400 to 23,075,399**.

The temporary count rose, 7,822,310 to 9,922,309, and that is an artifact of the instrument rather than a regression: heaptrack calls an allocation temporary when it is freed with no other allocation in between, so removing allocations reclassifies their neighbours into that bucket.
The total is the figure this entry claims.

#### Instructions

`perf stat -e instructions:u`, six interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each arm.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `compound` | 21,828,005,970 | 20,398,359,587 | -1,429,646,383 | **-6.550%** | 2,983,697 | 1,298,286 |
| `alloc4c` | 7,372,614,362 | 7,235,717,916 | -136,896,446 | **-1.857%** | 117,106 | 117,305 |
| `rexxcps` | 22,619,175,433 | 22,412,088,593 | -207,086,840 | **-0.916%** | 4,071,669 | 29,559 |
| `emptyloop` | 25,075,608,929 | 25,075,611,643 | +2,714 | up | 1,104 | 860 |
| `varlookup` | 42,009,633,106 | 42,009,635,313 | +2,207 | up | 1,109 | 1,539 |
| `arith` | 20,171,236,535 | 20,171,238,613 | +2,078 | up | 1,052 | 1,198 |
| `strings` | 41,859,584,993 | 41,859,586,941 | +1,948 | up | 1,017 | 1,090 |

**Four axes moved up, and the entry states it rather than rounding it away.**
Each is outside its own spread, so each is a difference and not a bound.
Each is also about two thousand instructions against runs of twenty to forty-two billion, and **the size does not scale with the axis's length** -- `emptyloop` runs 25,000,000 passes and `varlookup` 19,000,000, and they moved by the same couple of thousand.
A per-pass cost would differ between them by millions.
So this is a fixed cost paid once, not a per-clause one, and **no further attribution is offered**: separating a one-time startup difference from binary layout needs the do-nothing control this record has wanted since entry 31 and still does not have.

#### Behaviour

Bench axes: identical stdout, stderr and exit status under both arms on both engines.

A compound-tail probe covering a positive tail, a negative one, zero, a fourteen-digit tail, a defaulted tail, a loop-driven tail and a two-piece tail is **identical to the oracle**, byte for byte, on both engines.

**The first version of that probe was wrong and is recorded because it looked right.** It wrote `say zz.-3` intending a negative tail; that parses as `zz. - 3`, a subtraction against the stem's default, and the program failed at error 41 on both the oracle and here. The arms still agreed, so a check that had stopped there would have reported success while never testing a negative tail at all.

### Entry 40 -- a buffer lent for a tail key, and two axes that paid for it

Base `dbc0b79a2`, entry 39's head.
Found by the allocation-size histogram rather than by the call-site attribution the previous two entries used.

#### Cause, and how it was located

The call-site attribution had stopped being useful: at that head the largest remaining site was `step_in_temps_frame`'s own closure, which is where every clause's work inlines, so it names a region and not a cause.

The size histogram named one instead. Allocations of **exactly 19 bytes numbered 4,220,217**, more than any other width, on a 400,000-clause run.
Nineteen bytes is the width of `ACOMPOUND.Key Bee.14`, and `samples/rexxcps.rex` references `acompound.key1.loop` throughout its inner loop.
`INLINE_BYTES` is 54, so this never went through `Bytes` at all: it was `Interp::tail_key`'s own `Vec`, built for one lookup and dropped, about ten times a clause.

#### The change

`Interp` gains a `key_buffer` that the compound read and the compound write borrow and hand back, through `take_key_buffer` and `give_key_buffer`.

**Lent and returned rather than borrowed in place**, because every caller uses the key while calling back into `&mut self` -- to read a stem, to set one -- which a live borrow of a field forbids.

**Losing it is safe and costs only the reuse**, which is what makes this sound with no reentrancy guard: a caller that returns early leaves the field empty and the next taker allocates a fresh one, so an inner build gets its own buffer rather than corrupting an outer one. The compound read's `?` sits after the return for exactly that reason.

`join_tails` was split so the owning form delegates to the appending one rather than the loop existing twice. For this function two copies drifting apart would not be a slow path but a wrong variable.

#### Allocations

`heaptrack`, `samples/rexxcps.rex` at `count=20`/`averaging=20`: **23,075,399 to 20,015,402**.

Across entries 38, 39 and 40 the same measurement reads **29,775,401 to 20,015,402**, a fall of 32.8%.

#### Instructions

`perf stat -e instructions:u`, six interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each arm.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `compound` | 20,397,597,897 | 18,912,520,891 | -1,485,077,006 | **-7.281%** | 3,739,274 | 2,059,761 |
| `alloc4c` | 7,235,726,882 | 7,086,704,717 | -149,022,165 | **-2.060%** | 59,634 | 112,552 |
| `rexxcps` | 22,412,094,340 | 21,998,634,495 | -413,459,845 | **-1.845%** | 4,071,957 | 4,061,286 |
| `arith` | 20,171,239,065 | 20,178,260,558 | **+7,021,493** | +0.035% | 735 | 1,294 |
| `strings` | 41,859,586,953 | 41,865,584,696 | **+5,997,743** | +0.014% | 1,385 | 1,062 |
| `varlookup` | 42,009,636,058 | 42,009,633,592 | -2,466 | -0.000% | 845 | 756 |
| `emptyloop` | 25,075,611,272 | 25,075,608,874 | -2,398 | -0.000% | 1,161 | 1,151 |

**Two axes regressed, well outside their spans, and neither executes any of the changed code.**
`arith` and `strings` contain no dotted name, so they build no tail key and never reach `take_key_buffer`.
The regressions are also proportional rather than fixed -- about 14 instructions per iteration on `arith`'s 500,000 and about 2 on `strings`' 3,000,000 -- so they are not a one-time startup cost either.

**What is offered is the observation and no cause.** A field was added to a struct every hot path reaches, which changes its layout, and this record has wanted a do-nothing control since entry 31 precisely so a claim like "layout, not semantics" could be earned rather than asserted. That control still does not exist. What can be said is that the axes that regressed cannot see this change's semantics, and that the trade is heavily favourable: the three axes that do see it fall by between 149 million and 1.49 billion, against a combined 13 million elsewhere.

#### Behaviour

The compound-tail probe of entry 39 -- positive, negative, zero, fourteen-digit, defaulted, loop-driven and two-piece tails -- is **identical to the oracle** on both engines at this head.

### Entry 41 -- entry 40 named the wrong allocations

Correcting entry 40 by appending, in the shape entries 33 and 34 use.
**The change stands and every figure in that entry stands. What it identified as the cause is wrong.**

#### What was claimed

Entry 40 says the histogram named the tail key: allocations of **exactly 19 bytes numbered 4,220,217**, nineteen being the width of `ACOMPOUND.Key Bee.14`, and concludes "it was `Interp::tail_key`'s own `Vec`".

#### What the same instrument says after the change

The 19-byte bucket is **unchanged**, to the allocation: 4,220,217 before, 4,220,217 after.
A change that removed the tail-key `Vec` did not touch it, so it was never the tail-key `Vec`.

Differencing the two histograms, the whole of the fall is two other widths:

| size | before | after | delta |
|---:|---:|---:|---:|
| 8 bytes | 2,500,637 | 560,640 | **-1,939,997** |
| 16 bytes | 1,400,406 | 280,408 | **-1,119,998** |

That sums to 3,059,995 against a total fall of 3,059,997.
**A tail key is built by pushing into an empty `Vec`**, so it takes an 8-byte allocation and then a 16-byte one as it grows through the first capacity -- two allocations per key, at the two smallest capacities, and nothing at the finished key's own width.

#### Why the reasoning failed, since the arithmetic was right

Nineteen *is* the width of that resolved name, and a resolved name *is* built on that path.
The step that was never taken is the one that would have falsified it: **the histogram was read once, before the change, and never differenced against itself afterwards.**
A single reading can only suggest a cause; the difference is what tests it, and it was available for the price of the run that was already made.

This is the shape this record has recorded before -- a correct decision carrying a false reason, which passes review on the decision's merits.

#### What is still true

The change removed 3,059,997 allocations and moved `compound` by -7.281%, `alloc4c` by -2.060% and `rexxcps` by -1.845%, and the two regressions entry 40 reports stand as reported.
`Interp::key_buffer`'s doc comment carried the same false identification and is corrected at this entry's commit.

#### What this opens

**The largest single allocation width in the interpreter is unidentified**, and it is 4,220,217 allocations of 19 bytes on a 400,000-clause run -- more than any other width, and untouched by every entry so far.
Naming it is the next allocation question, and the method is now differencing rather than reading.

### Entry 42 -- the 19-byte allocations, named

Entry 41 left the largest single allocation width in the interpreter unidentified: 4,220,217 allocations of 19 bytes on a 400,000-clause `samples/rexxcps.rex` run, untouched by every change before it.
This entry names them. **No code changed; this is a measurement.**

#### The site

`Interp::render` (`value.rs`), reached from `Interp::compare_values` (`eval.rs`), reached from `apply_binary`.

Comparing a tagged small integer renders it to a `String` first, through `to_string`, and 19 is the width that pre-sizes: `SMALL_INT_MAX` is 2,305,843,009,213,693,951, which is nineteen digits.
`compare_values` renders **both** operands unconditionally, before knowing whether the comparison will be numeric -- and when both operands are small integers, the rendering is never read for anything but a comparison the integers themselves could answer.

#### How it was found, including the probe that was void

`heaptrack`'s call-site attribution could not answer it: the largest site was `step_in_temps_frame`'s closure, which is where every clause's work inlines, and it names a region rather than a cause. The size histogram named a width, not a place. The per-stack flamegraph export tops out at 980,000 for a single stack, so the width is spread across many.

A conditional breakpoint on the allocator answered it, but **only after the first three attempts were discovered to be void**:

* `break *0x4e780` -- the binary is position-independent, so a raw address from `nm` is not where the code loads. Rejected outright by gdb, which is the harmless failure.
* `break __rust_alloc if $rdi == 19` on `samples/rexxcps.rex` -- ran to completion, never fired, and **that proved nothing**: the same breakpoint with the condition `$rdi == 8` also never fired, on a program that certainly makes 8-byte allocations. The symbol exists but link-time optimisation left no call site reaching it.
* `break __rust_realloc if $rcx == 19` -- same, and void for the same reason.

**The control is what turned a conclusion into a non-conclusion.** Two "never fired" results had already been read as evidence that the allocation was neither an alloc nor a realloc. Breaking on `malloc` in libc, where the control does fire, produced the stack above on the first hit.

The first hit on a small program is parse-time -- `SymbolTable::intern` upcasing a symbol, through `to_ascii_uppercase` -- which is a one-off and cannot account for millions. Ignoring the first hits and catching a later one, inside the loop, gives `render`.

#### The fix this points at, and what it must not get wrong

Both operands being tagged small integers means the answer is available from the integers.
It is **not** simply "compare the two `i64`s", and the guards are the substance:

* **`NUMERIC FUZZ` must be zero.** Fuzz makes a numeric comparison compare fewer digits, so two distinct integers can be equal under it.
* **Both magnitudes must sit inside `NUMERIC DIGITS`.** A numeric comparison rounds to significant digits first, so at `DIGITS 9` two distinct ten-digit integers can compare equal.
* **The strict *ordering* operators must be excluded.** `>>` and `<<` compare strings, not numbers: `9 >> 10` is true where `9 > 10` is false. Strict *equality* is safe, because a small integer renders canonically and two equal renderings mean equal values.

So the fast path is: both `Decoded::SmallInt`, `fuzz` zero, both magnitudes below ten to the `digits`, and the operator outside the strict ordering family.

### Entry 43 -- the 40-byte allocations, named

The second-largest width at entry 41's head, measured the same way entry 42's was, and stated here because the previous two entries show what happens when a width is attributed by reading instead.

**`Box<Number>`, filled by `Interp::to_number` into `Body::Text`'s `num` cache** (`value.rs`), caught on a `malloc` breakpoint conditioned on 40 with the control that fires.

`Number` is 40 bytes, defended by a `const` assertion in `rexx-num`, and it is boxed here rather than held inline because `Body`'s width is every arena slot's width -- `Body::Num` already carries a `Number` inline, and a second one inside `Body::Text` would widen every slot in the heap.

So **the price of the parse cache is one heap allocation per string that is ever asked for its numeric value**, and the cache is what makes the second such question free. The trade is deliberate and documented on the field; what was not measured until now is what the boxing side of it costs, which on `samples/rexxcps.rex` at 400,000 clauses is 1,670,604 allocations.

Not a defect and no change is proposed here. It is recorded because the two widths above it are now named and this is the next one down, and because any future attempt to widen `Body` or to inline this `Number` has a figure to be measured against.

#### The state of the allocation profile at this head

Total on that run is 20,015,402, from 29,775,401 before entry 38.
Named: 19 bytes at 4,220,217 (entry 42), 40 bytes at 1,670,604 (here).
**Unnamed: the 1, 2, 3, 7, 24 and 33-byte widths**, which together are of the same order as the two named ones. None has been attributed, and on this record's experience none should be guessed at.

### Entry 44 -- the small widths are builtin results, allocated to be copied and dropped

Moritz asked why the 1-byte allocations were not simply avoidable. They are, and so are several widths beside them.
Measured, not inferred; **no code changed here**.

#### What they are

`builtin::string::substr` (`string.rs`), reached through `builtin::run` and `Interp::invoke_call`, caught on a `malloc` breakpoint conditioned on 1 inside a loop calling `substr(1234, 1, 1)`.

Every such builtin builds its answer in an owned `Vec<u8>` from the shared `buffer(len)` helper and hands it to `Interp::text_owned`.
**`Bytes::from_vec` then copies anything of `INLINE_BYTES` or less into the inline buffer and drops the `Vec`.** `INLINE_BYTES` is 54.
So for every short builtin result the sequence is: allocate, fill, copy into the object, free.

The widths line up with what `samples/rexxcps.rex` asks for in its inner loop, and each is its own bucket in the histogram:

| width | count | what the program asks for |
|---:|---:|---|
| 1 | 1,960,482 | `substr(1234,1,1)` |
| 2 | 1,120,188 | `substr(1234"5678",6,2)` |
| 3 | 970,381 | `word(key1,1)` giving `Key` |
| 7 | 980,088 | `key1` itself, `Key Bee` |

That correspondence is a reading of the program beside the histogram, not a measurement of each width -- only the 1-byte case was caught in the debugger. The mechanism is measured; the per-width attribution of the other three is not, and this record has been wrong about exactly that before.

#### The probe, and the skip that hid it

The first attempt ignored the first 400 hits and never fired, on a program whose one-byte allocations are all parse-time and number fewer than that.
**A skip large enough to pass the whole population reads exactly like an absence.** Re-running with no skip fired immediately, in `SymbolTable::intern` -- parse-time, a handful, and not the answer either.
What identified it was a program written to isolate the suspect: a loop calling `substr` and nothing else.

#### The fix this points at

The lending discipline already in the tree for `Interp::key_buffer`, applied to builtin results: take a buffer, build into it, and either hand a slice to `Interp::text` for the inline case or give the buffer up with `text_owned` when the result exceeds `INLINE_BYTES`.

That is 34 `text_owned` call sites across the builtin modules, all funnelling through one `buffer(len)` helper, and each is independent -- so it can be taken a module at a time rather than as one change.

### Entry 45 -- the builtin result buffer, and the prediction it did not meet

Base `519618b02`. Entry 44 predicted that the 1, 2, 3 and 7-byte widths were builtin results and together about a quarter of every allocation. **The change removed 420,000, not five million.**

#### The change

`Interp::result_buffer`, lent to `builtin::buffer` and handed back by `Interp::text_built` when the finished result is `INLINE_BYTES` or less -- the case where `Bytes::from_vec` copies into the object and drops the `Vec` anyway.

**A `Cell` where the key buffer is a plain field**, and that is the design's one real constraint: a builtin reads its argument's bytes out of the heap first, which borrows the `Interp`, so a take needing `&mut self` is refused while that borrow is live. `Cell::take` needs only `&self`.

Two callers keep a fresh allocation. `pack_hex` has no `Interp`, and **`padding_width` uses the reservation itself as the resource check and then drops it** -- a lent buffer that already had the capacity would make that check succeed without asking the allocator anything, which is the one place reuse would be a defect rather than a saving.

#### What the histogram says, against what entry 44 predicted

| size | before | after | delta |
|---:|---:|---:|---:|
| 1 | 1,960,482 | 1,680,481 | -280,001 |
| 2 | 1,120,188 | 980,189 | -139,999 |
| 3 | 970,381 | 970,381 | **unchanged** |
| 7 | 980,088 | 980,088 | **unchanged** |

Total 20,015,402 to 19,595,402.

**Entry 44's correspondence between widths and program text was wrong, and it said so in advance.** It marked the 2, 3 and 7-byte attributions as a reading beside the histogram rather than a measurement, because only the 1-byte case had been caught in the debugger. The 3 and 7-byte widths do not come through `buffer` at all, and even the 1-byte width is mostly something else: 1,680,481 of it survives.

So **the largest remaining widths are unidentified again**, and the method that works is the one entry 42 used -- a breakpoint with a control that fires, on a program written to isolate one suspect.

#### Instructions

Four axes, five interleaved rounds per arm, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 41,865,584,723 | 41,679,584,559 | -186,000,164 | -0.444% | 54,000,556 | 830 |
| `rexxcps` | 21,998,632,970 | 21,967,951,137 | -30,681,833 | -0.139% | 25,990 | 4,090,468 |
| `compound` | 18,912,523,312 | 18,933,361,767 | **+20,838,455** | +0.110% | 2,438,551 | 2,060,660 |
| `arith` | 20,178,260,479 | 20,184,260,167 | **+5,999,688** | +0.030% | 1,172 | 948 |

Every difference is outside both spans. **Two axes improved and two regressed**, for a net of about 190 million across the four, and the regression is on `compound`, which is the axis this phase has moved furthest.

**Kept on that net, and the entry records the mixed sign rather than the total alone.** No cause is offered for either regression: the change adds a field to a struct every path reaches, and the do-nothing control that would separate layout from semantics still does not exist.

#### Behaviour

A sweep over `substr`, `word`, `words`, `delstr`, `c2x`, `x2c`, `d2x`, `x2d`, `b2x`, `left`, `right`, `copies`, `translate`, `reverse`, `strip`, `format`, `trunc`, `abs`, `sign`, `insert`, `overlay`, `space`, `d2c`, `c2d`, `bitand`, `lower`, `upper` and `compare` is **identical to the oracle**, and the two engines agree.

### Entry 46 -- two small integers answer their own comparison

Base `f76e6f134`. The change entry 42 identified and specified.

#### The change

`Interp::compare_values` rendered both operands to text before knowing whether the comparison would be numeric. When both are tagged small integers the integers settle it, and `small_int_compare` answers from them.

The guards are the substance, and each is a case where comparing the integers would give a different answer from what the general path compares:

* **`NUMERIC FUZZ` must be zero**, since fuzz compares fewer significant digits and can make distinct integers equal.
* **Both magnitudes must sit inside `NUMERIC DIGITS`**, since a numeric comparison rounds first: at `DIGITS 9`, `1000000001 = 1000000002` is **true**.
* **The strict ordering operators are excluded**, because they compare strings: `9 >> 10` is true where `9 > 10` is false. Strict equality is included, because a small integer renders canonically, so equal renderings and equal values imply each other.

#### Allocations, and the prediction again overshooting

`heaptrack` on `samples/rexxcps.rex` at `count=20`/`averaging=20`: **19,595,402 to 19,015,400**, a fall of 580,002.

Entry 42 named 4,220,217 allocations of nineteen bytes as this site. The fall is an eighth of that, and the reason is visible in the program: `rexxcps` compares `flag` against a compound (`flag` is unset, so a string) and `j` against a compound (`j` is `1.1`), and neither pair is two small integers. **What was measured was the site; what the fast path can take is the subset where both operands qualify.** That distinction was not made in entry 42 and is made here.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `rexxcps` | 21,967,950,082 | 21,589,027,590 | -378,922,492 | **-1.725%** | 14,505 | 24,709 |
| `compound` | 18,932,520,676 | 18,932,524,574 | +3,898 | bound | 3,742,352 | 837,807 |
| `strings` | 41,679,584,746 | 41,682,584,267 | +2,999,521 | bound | 54,000,139 | 669 |
| `varlookup` | 42,123,632,954 | 42,123,633,496 | +542 | bound | 852 | 1,198 |
| `arith` | 20,184,260,395 | 20,184,260,573 | +178 | bound | 1,300 | 826 |
| `emptyloop` | 25,025,608,795 | 25,025,608,762 | -33 | bound | 1,360 | 736 |

`rexxcps` moved 15,335 times its wider span and **every other axis moved less than its own spread**, so each is a bound and none is a regression. That is the cleanest shape any entry in this allocation series has produced.

#### Behaviour, and the witness that the path is taken

A probe over every guard -- the `DIGITS 9` rounding case, a non-zero `FUZZ`, `>>` against `>`, `<<` against `<`, strict equality, negative values, zero, both ends of the tagged range, and numeric-looking strings such as `'09' = 9` and `' 9 ' = 9` -- is **identical to the oracle** and the two engines agree.

**And the probe was shown to reach the new code before being trusted.** Inverting the fast path's equality arm changes eight lines of that probe's output, so a green run of it is evidence about this change rather than about the general path.

#### A measurement that was taken with the wrong binary, and caught

The first attempt at the table above ran `cd rust` in a shell whose directory had already been reset, so the `cargo build --release` behind the `&&` never ran and `target/release/` still held the **mutated** binary from the witness above.
It reported `rexxcps` at **-99.973%**, because a build with comparisons inverted fails `rexxcps`' own self-checks and exits early.
The absurdity is what exposed it; a smaller mutation would have produced a plausible number. **A stale binary outliving its revert is a hazard this record already knew about, and knowing it did not prevent it** -- what did was reading the shell's own error line rather than only the figures beneath it.

### Entry 47 -- the `PARSE` source read, borrowed instead of copied

Base `9f5ae5fbc`. Found by taking the 33-byte width to the debugger, as entry 46 said the next one should be.

#### Locating it

Neither a compound-heavy program nor a decimal-loop program produced a single 33-byte allocation, with the probe run unskipped so that "never fired" meant something.
`samples/rexxcps.rex` itself did: `Interp::read_parse_var`, on the `PARSE` path, which those probe programs had no reason to reach.

That is the third width in a row where the site was found by writing a program to isolate a suspect and the first two guesses were wrong.

#### The change

`read_parse_var` copied the symbol's spelling with `to_vec()`, and its compound arm additionally copied the stem name, cloned it, joined it to a freshly built tail key, and rendered the value -- all before handing them to `trace_variable` and `trace_compound_name`, which return at once unless intermediates are on.

The name is now borrowed, for the reason it can be: `SymbolTable::name` answers with the `Code`'s lifetime, which is the program and not the `Interp`. The renderings and the joined name are built only when a line will print them, and the tail key uses the lent buffer.
**Every one of these is a shape already fixed elsewhere in this record** -- entry 38 for the copies and the gating, entry 40 for the key buffer -- reaching a function those entries did not touch.

#### Allocations

`heaptrack`, `samples/rexxcps.rex` at 20/20: **19,015,400 to 17,335,400**, a fall of 1,680,000.

#### Instructions

Five interleaved rounds per arm, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `rexxcps` | 21,589,027,950 | 21,366,715,796 | -222,312,154 | **-1.030%** | 23,800 | 8,125,190 |
| `arith` | 20,184,260,716 | 20,185,260,540 | **+999,824** | +0.005% | 331 | 921 |
| `compound` | 18,932,524,693 | 18,932,901,230 | +376,537 | bound | 2,406,505 | 1,302,517 |
| `strings` | 41,682,584,516 | 41,682,584,306 | -210 | bound | 126,000,102 | 1,056 |
| `emptyloop` | 25,025,609,073 | 25,025,609,045 | -28 | bound | 1,197 | 1,741 |

`arith` executes no `PARSE` and regressed by about a million instructions, outside its spans. No cause is offered; the do-nothing control this record has wanted since entry 31 would be what earns one.

#### Behaviour, and a probe whose first reading was wrong

A `TRACE R` program parsing from a simple variable, a compound and a bare stem is identical to the oracle.

**The first comparison said it differed.** It redirected both streams into one file, and trace goes to stderr while `SAY` goes to stdout, so the two sides interleaved differently and the diff showed lines moved rather than changed. Compared stream by stream, **stdout and stderr are each identical to the oracle**, and the base and head binaries are byte-identical on that program anyway -- which is the check that settled it, since this change could not have caused a difference that base also shows.

### Entry 48 -- the call path's two buffers, and the instrument that was measuring a moving workload

Base `6de24daa6`.
The first entry in this series to profile every benchmark program rather than `samples/rexxcps.rex` alone, and the widening is what found both of the things below.

#### The instrument was wrong, and every allocation figure entries 38 to 47 quote is affected

`samples/rexxcps.rex` **adjusts its own workload to the speed of the interpreter running it**.
Line 163 is `count=(1%total + 1) * count`, inside `do trial=1 to 2`: if the first trial takes under a second, the count is scaled and the whole measurement is run again.

Under `heaptrack` the run is slow enough that the scaling lands differently from one run to the next.
Three runs of one binary on one program, `count=20`/`averaging=20`:

| run | count reached | allocations |
|---|---:|---:|
| 1 | 60 | 2,775,674 |
| 2 | 60 | 2,775,676 |
| 3 | 40 | 2,082,427 |

**A third of the figure, from nothing but the machine's load.**
Every step in the series entries 38 to 47 report -- 5.7%, 13%, 3%, 8.8% -- is smaller than that swing.
The direction of those changes is still supported by the instruction counts, which are a separate instrument and were not affected; what is withdrawn is the precision of the allocation *deltas*, not the finding that each change removed allocations.

**The instruction axis is not affected, and that was checked rather than assumed.** At the default `count=100`/`averaging=100` the first trial takes 2.287788 seconds, so the `total>1` arm leaves at once: one trial, count still 100, workload fixed. The adaptation only fires when a trial comes in under a second, which at 100/100 it does not.

The instrument from here is the same program with `do trial=1 to 1`, which pins the workload. Two runs of it: 695,620 and 695,620.

#### What the widening shows

`heaptrack` on every benchmark program, each scaled down so the profiled run is short, at this entry's base:

| axis | allocations | iterations | per iteration |
|---|---:|---:|---:|
| `emptyloop` | 383 | 500,000 | ~0 |
| `varlookup` | 417 | 400,000 | ~0 |
| `compound` | 100,512 | 100,000 | 1.0 |
| `alloc4c` | 250,519 | 50,000 | 5.0 |
| `strings` | 600,548 | 60,000 | 10.0 |
| `arith` | 528,794 | 20,000 | **26.4** |

**The clause loop itself is allocation-free**, which `emptyloop` and `varlookup` settle: two axes that run 500,000 and 400,000 iterations allocate a few hundred times between them, nearly all of it before the program starts.
Everything in this series is therefore about what a *clause's work* allocates, not about the loop.

Folding the stacks and attributing each to the deepest frame in this workspace's own crates:

* `strings`: `invoke_call` 480,000, `changestr` 60,000, `concat_values` 60,000. **Exactly eight per iteration from `invoke_call` for four builtin calls** -- two per call, in the call machinery rather than in any builtin.
* `alloc4c`: `invoke_call` 100,000, `stem_set_at` 50,001, `concat_values` 50,000.
* `compound`: `stem_set_at` 100,000, one per iteration and essentially the whole of it.
* `arith`: `arith_general` 178,318, `rexx_num::muldiv::mul` 94,271, `numeric_operand` 80,000, `rexx_num::digits::spill` 56,414, then `round_to`, `add_signed`, `truncated_to`. **`rexx-num` allocates heavily here**, which is worth stating plainly because the Unit 4 premise check concluded it allocated seven times in a whole run -- that check was run against `rexxcps`, whose arithmetic is on tagged small integers, and it does not carry to an axis that runs `NUMERIC DIGITS 20`.

#### The change

`invoke_call` allocated a `Vec<Option<Argument>>` for the evaluated arguments, and on the builtin path a second `Vec<Option<ObjRef>>` collected from it for `builtin::dispatch`. Both were freed on the way out.
Both are now lent from the `Interp` and handed back, in the shape `key_buffer` and `result_buffer` already use.

The buffers are given back **before the outcome is read**, so a builtin that raises keeps them for the next call rather than losing them to the error path.
A non-builtin call still gives its argument buffer up, because the callee is handed the arguments and keeps them -- the next builtin call allocates once into the empty `Vec` left behind and hands the capacity back for the one after it.

#### Allocations

Each program at the same scale as the table above, base against head:

| axis | base | head | |
|---|---:|---:|---:|
| `strings` | 600,548 | 120,550 | **-79.9%** |
| `alloc4c` | 250,519 | 150,521 | -39.9% |
| `rexxcps` (pinned) | 695,620 | 617,056 | -11.3% |
| `compound` | 100,512 | 100,512 | unchanged |
| `arith` | 528,794 | 528,794 | unchanged |

`compound` and `arith` are unchanged **exactly**, and that is the control this change wants: neither program calls a builtin, so neither can reach the changed lines.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each. The head binary was rebuilt from the restored sources and compared byte for byte with the copy measured, so the arm measured is the arm described.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 41,682,584,760 | 40,290,587,659 | -1,391,997,101 | **-3.340%** | 1,368 | 792 |
| `alloc4c` | 7,098,726,800 | 6,985,764,469 | -112,962,331 | **-1.591%** | 54,866 | 71,495 |
| `rexxcps` | 21,366,758,338 | 21,263,611,609 | -103,146,729 | **-0.483%** | 10,515 | 2,388,480 |
| `arith` | 20,185,260,462 | 20,176,736,907 | -8,523,555 | -0.042% | 1,460 | 706 |
| `compound` | 18,933,360,889 | 18,932,523,451 | -837,438 | bound | 2,523,132 | 1,219,087 |
| `emptyloop` | 25,025,609,624 | 25,025,608,928 | -696 | bound | 349 | 993 |
| `varlookup` | 42,123,633,794 | 42,123,633,706 | -88 | bound | 568 | 1,395 |

`arith` moved outside its own spans again and in the opposite direction from entry 47, on an axis whose allocation count this change leaves *exactly* unchanged and which executes no builtin call at all. Both movements are the size this record has measured a do-nothing control producing. That control still does not exist, and this is now the second consecutive entry that would have used it.

#### Behaviour, and the witness that the probe reaches the change

A probe over the argument shapes this path carries -- nested builtin calls, an omitted interior position, a builtin with fewer arguments than the call before it, `ARG()`, a `USE ARG >` reference argument, a builtin raising 40.x into `SIGNAL ON SYNTAX`, and `TRACE I` so every `>A>` line is compared -- is **identical to the oracle on stdout, on stderr, and in exit status**, compared stream by stream rather than merged.

**And the probe was shown to fail before being trusted.** Dropping the `clear()` from `take_value_buffer`, so a call sees the previous call's trailing values, makes that probe exit 216 and diverge from its second line. The sources were then restored from the backup, re-verified with `sha256sum -c`, rebuilt, and the probe re-run against the oracle.

### Entry 49 -- the tail key, looked up before it is inserted, and the axis that pays for it

Base `7964f9d1c`. Entry 48's widening named `stem_set_at` as the whole of what `bench-programs/compound.rex` allocates: one per iteration, five million of them.

#### The change

`tails.insert(key.to_vec(), Some(value))` allocated an owned key on **every** compound assignment.
`insert` needs an owned key whether or not it keeps one, so on a tail that already exists the copy is made, handed over, found redundant and dropped again.
The write now probes with `get_mut` first and only builds an owned key when the tail is new.

#### Allocations

Same scaled programs as entry 48:

| axis | base | head | |
|---|---:|---:|---:|
| `compound` | 100,512 | 1,012 | **-99.0%** |
| `rexxcps` (pinned) | 617,056 | 595,069 | -3.6% |
| `alloc4c` | 150,521 | 150,521 | unchanged |
| `strings` | 120,550 | 120,550 | unchanged |

`compound` is now within a thousand allocations of the allocation-free floor `emptyloop` and `varlookup` sit at.

**`alloc4c` is unchanged for a reason worth stating**, because it is the reason for the regression below: that program writes a *fresh* tail every iteration, so every write is a genuine insert and no allocation can be removed from it at all.

#### Instructions, and a regression that was predicted before it was measured

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each. The head binary was rebuilt from the restored sources and compared byte for byte with the copy measured.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `compound` | 18,932,521,974 | 18,239,083,646 | -693,438,328 | **-3.663%** | 2,519,987 | 1,954,591 |
| `rexxcps` | 21,263,610,188 | 21,189,086,260 | -74,523,928 | **-0.350%** | 7,977,459 | 2,099,434 |
| `alloc4c` | 6,985,682,651 | 7,210,799,815 | +225,117,164 | **+3.223%** | 110,742 | 156,093 |
| `strings` | 40,290,587,263 | 40,290,587,445 | +182 | bound | 907 | 963 |
| `arith` | 20,176,736,400 | 20,176,737,053 | +653 | bound | 1,642 | 214 |
| `emptyloop` | 25,025,609,505 | 25,025,608,671 | -834 | bound | 897 | 1,641 |
| `varlookup` | 42,123,633,490 | 42,123,633,736 | +246 | bound | 573 | 534 |

**This is a trade, not a win, and both halves are mechanism rather than layout.**
A hit pays one probe and no allocation where it used to pay one probe and an allocation: `compound` saves 138 instructions per iteration, which is about what a `malloc`/`free` pair of a short key costs.
A miss pays *two* probes: `alloc4c` loses 225 per iteration, which is a hash plus a second random access into a table that by then holds a million entries and does not fit in cache.

So a stem written once per tail and never revisited is slower, and a stem revisited -- an accumulator, a counter, anything indexed in a loop that runs longer than the index's range -- is faster. `rexxcps`, the most mixed program measured here, is faster.

**The shape that would win both is a key that stores a short tail inline**, so that the miss path allocates nothing either and there is nothing to trade. `rexx-core`'s `Bytes` is that shape already and would need `Hash`, `Eq` and `Borrow<[u8]>` to serve as a map key; at `INLINE_BYTES` it also makes each entry 72 bytes against 40, which on a million-tail stem is its own regression in a dimension this table does not measure. That is a change with its own design question and its own measurement, and it is not this entry's.

#### Behaviour

A probe over the tail-write shapes -- a fresh tail, the same tail overwritten, a tail dropped and then revived, a compound tail with a variable index, a nested tail, a loop that fills tails and a second loop that updates them, a whole-stem reassignment, `DROP` of the stem, and `SYMBOL()` afterwards, all under `TRACE R` -- is **identical to the oracle on stdout, on stderr, and in exit status**, compared stream by stream.

**And the probe was shown to fail before being trusted.** Making the hit arm do nothing, so an overwrite silently keeps the old value, makes that probe exit 215 and diverge from its first line. The source was then restored from the backup, re-verified with `sha256sum -c`, rebuilt, and the probe re-run against the oracle.

### Entry 50 -- the shared buffer, and the two callers that were defeating it

Base `010e741be`. Entry 48 left `strings` at two allocations per iteration and entry 49 did not touch them.
Both were the *same* buffer, taken and given back correctly, and reallocated on every pass anyway.

#### Three changes, one mechanism

`concat_values` built its join in a fresh `Vec::with_capacity` and finished with `text_owned`. A join that fits `INLINE_BYTES` is copied into the object and the `Vec` dropped, so the allocation was pure waste; it now builds in the lent buffer and finishes with `text_built`, which is entry 45's shape reaching a function entry 45 did not touch.

That alone moved `strings` by **nothing at all**, and the reason is the second change.
`builtin::buffer` reserved with `try_reserve_exact`, which resizes the shared buffer to precisely one call's need. `strings` asks `changestr` for 43 bytes and then joins 46, so the buffer was resized down and grown again on every iteration and the lending bought nothing. It reserves with `try_reserve` now, so the capacity settles at the longest result the program asks for.

The third is `changestr` itself, which built in a fresh `Vec::with_capacity`. That was not one allocation but two: `text_built` hands whatever it is given back to the pool, so a fresh buffer *replaced* the shared one at a tighter capacity, and the next caller wanting one byte more grew it again.

**Reading the histogram would not have found the second or third of these.** The site attribution named `changestr` and a `RawVec` grow; what connected them was that removing the first allocation changed the count by zero.

#### Allocations

| axis | base | head | |
|---|---:|---:|---:|
| `strings` | 120,550 | 552 | **-99.5%** |
| `alloc4c` | 150,521 | 100,523 | -33.2% |
| `rexxcps` (pinned) | 595,069 | 539,066 | -9.4% |
| `compound` | 1,012 | 1,012 | unchanged |
| `arith` | 528,794 | 528,794 | unchanged |

`strings` began this session at 600,548 and runs 60,000 iterations. It is now inside the few hundred allocations `emptyloop` and `varlookup` sit at, which is the whole of what a program costs before it starts.

Its temporary allocations fell from 120,092 to 93 -- and the `try_reserve` change is what did that, before `changestr` was touched at all: a buffer reallocated to a different exact size every pass is freed with nothing allocated in between, which is heaptrack's definition of temporary.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each. The head binary was rebuilt from the restored sources and compared byte for byte with the copy measured.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 40,290,587,151 | 40,017,586,486 | -273,000,665 | **-0.678%** | 935 | 640 |
| `alloc4c` | 7,210,779,797 | 7,165,829,774 | -44,950,023 | **-0.623%** | 179,109 | 230,363 |
| `rexxcps` | 21,189,081,392 | 21,127,484,961 | -61,596,431 | **-0.291%** | 4,070,430 | 5,522,603 |
| `compound` | 18,238,268,125 | 18,238,725,943 | +457,818 | bound | 4,918,852 | 2,821,357 |
| `arith` | 20,176,736,922 | 20,176,737,336 | +414 | bound | 858 | 573 |
| `emptyloop` | 25,025,609,180 | 25,025,608,921 | -259 | bound | 1,310 | 887 |
| `varlookup` | 42,123,633,786 | 42,123,633,699 | -87 | bound | 652 | 933 |

**An allocation removed is worth about 45 instructions here, and that is worth writing down** because this series has been chasing allocation counts as a proxy. `strings` lost six million allocations across its full run for 273 million instructions. Entry 49's `compound` figure gives 138 for the same trade. The proxy is real but the exchange rate is small, and a change that removes allocations while adding a probe can come out behind -- entry 49 is the instance.

#### A process abort, removed on the way past

`changestr` sized its result with `Vec::with_capacity`, which is an infallible reservation of a length computed from user input. Measured at the project's own `ulimit -v 1048576`, `changestr('a', copies('a',200000000), 'bb')` **aborts the interpreter** at `010e741be`: `memory allocation of 400000000 bytes failed`, rc 134. Through `buffer` it raises 5.1 at rc 251 instead.

At a 2 GB cap both binaries answer `400000000`, as the oracle does at 1 GB. The remaining gap is that this program needs more memory in this representation than in the oracle's, which is a footprint difference and not a new one.

#### Behaviour

A probe over `changestr` at every shape -- no occurrence, a replacement shorter and longer than the needle, a count limit, an empty needle, an empty replacement, a result crossing `INLINE_BYTES` in both directions, and joins of operands on either side of that boundary -- is **identical to the oracle on stdout, on stderr, and in exit status**, compared stream by stream.

### Entry 51 -- what `arith` actually allocates, and it is not arithmetic

Base `d865fefba`. `arith` was the last axis entry 48 left untouched, at 26.4 allocations per iteration.

#### Twelve programs, one operation each

Rather than read the site attribution and guess -- which this series has now done wrongly three times -- each operation was isolated in its own program at a fixed 20,000 iterations and profiled alone. The empty-loop baseline is 432.

| program | allocations | per iteration |
|---|---:|---:|
| baseline | 432 | 0.02 |
| `numeric digits` switched twice | 160,402 | **8.02** |
| `+` at DIGITS 9 or 20 | 441 | 0.02 |
| `*` at DIGITS 9 or 20 | 441 | 0.02 |
| `**` at DIGITS 20 | 440 | 0.02 |
| `//` at DIGITS 20 | 441 | 0.02 |
| `/` at DIGITS 9 | 60,459 | 3.02 |
| `/` at DIGITS 20 | 100,459 | 5.02 |
| `*` on two 20-digit operands | 80,464 | 4.02 |
| `/` on two 20-digit operands | 120,461 | 6.02 |

**Addition, multiplication, power and remainder on operands that fit `INLINE_DIGITS` allocate nothing at all.** Only two things allocate: division, at three to six per operation, and the `NUMERIC` clause -- which is not arithmetic and was the single largest item on the axis.

That also settles the standing question about `rexx-num`: it does not allocate per operation, it allocates per *spill*, and a spill happens when a working value needs more than the twenty digits that fit inline. A 20-digit multiply produces forty digits before rounding, which is why that row is the one that moves.

#### The change

`exec_numeric` built `rexx_num::DEFAULT_DIGITS.to_string()` before every `NUMERIC DIGITS` clause, including the ones that supply an expression and never read it -- the restore value, allocated to be discarded.
`numeric_operand` then copied the operand's rendered bytes into an owned `Vec`, and copied that again into an owned `String` because `set_digits_str` takes `&str`.

The default is now built only in the arm that uses it; the operand is built in the lent result buffer, taken *after* the expression is evaluated so a builtin inside that expression gets the buffer for its own result first; and the `&str` comes from `String::from_utf8_lossy` borrowing in the caller, which allocates only for an operand that is not valid UTF-8 and therefore cannot parse as a count anyway.

#### Allocations

| axis | base | head | |
|---|---:|---:|---:|
| `numeric digits` switched twice | 160,402 | 40,403 | **-74.8%** |
| `arith` | 528,794 | 408,795 | -22.7% |
| `/` at DIGITS 20 | 100,459 | 100,457 | unchanged |

The division control is unchanged, which is what says the fall came from the clause and not from the arithmetic beside it.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each. The head binary was rebuilt from the restored source and compared byte for byte with the copy measured.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `arith` | 20,176,737,115 | 19,859,504,395 | -317,232,720 | **-1.572%** | 1,904 | 419 |
| `emptyloop` | 25,025,608,957 | 24,975,609,204 | -49,999,753 | **-0.200%** | 1,458 | 973 |
| `rexxcps` | 21,127,489,866 | 21,113,344,315 | -14,145,551 | -0.067% | 4,071,352 | 2,933,494 |
| `strings` | 40,017,586,533 | 40,017,586,317 | -216 | bound | 1,059 | 956 |
| `alloc4c` | 7,165,786,401 | 7,165,795,683 | +9,282 | bound | 164,281 | 194,714 |
| `compound` | 18,238,265,805 | 18,238,218,880 | -46,925 | bound | 2,102,057 | 2,148,841 |
| `varlookup` | 42,123,633,183 | 42,123,633,502 | +319 | bound | 857 | 851 |

**`emptyloop` executes no `NUMERIC` clause and moved by 49,999,753 on a loop of 25,000,000 passes**, which is exactly two instructions per pass. That is the same signature the bare-stem task measured for a change to a call site's constant argument, and it is codegen rather than this path. It went the favourable way this time; entries 47 and 48 recorded the same size of movement going the other way on `arith`. The do-nothing control remains the thing that would settle all three.

#### Behaviour

A probe over `NUMERIC DIGITS` with an expression, bare (restore), from a variable, from an expression, `NUMERIC FUZZ` both ways, `NUMERIC FORM` both ways, and the three error shapes (26.5 on a non-whole operand, on zero, and on a fuzz not below digits) trapped by `SIGNAL ON SYNTAX`, all under `TRACE R` so every `>K>` line is compared, is **identical to the oracle on stdout, on stderr, and in exit status**, compared stream by stream.

**And the probe was shown to fail before being trusted.** Truncating the operand buffer to one byte makes it exit 223 and report `2 0.33` where the first line should read `20 0.33333333333333333333`. The source was then restored from the backup, re-verified with `sha256sum -c`, rebuilt, and the probe re-run against the oracle.

#### Where this leaves the axis

`arith` is at 20.4 allocations per iteration, and what remains is division (three to six per operation) plus one per `NUMERIC` clause. Division allocates working buffers that outlive `INLINE_DIGITS`, and removing those needs a scratch buffer inside `rexx-num` -- a crate with no `Interp` to lend from, so the lending shape used everywhere in this series does not carry across without a thread-local or an explicit scratch parameter. That is its own design question.

### Entry 52 -- ten digits of inline capacity for nothing, and a stack buffer that cost more than it saved

Base `569381852`. Entry 51 left `arith` at division, and the question that opened this was whether the multiply's working width is gated on the operands' real width or assumed to be the inline capacity.

#### It is gated on the real width, and that was not where the spill came from

`mul_magnitudes` sizes its product from the operands' own lengths. The spill came from somewhere narrower: a division and a multiply both keep `digits + 1` digits, and at `NUMERIC DIGITS 20` that is **21** against an `INLINE_DIGITS` of 20. Every kept product and every division working value at that setting missed the inline buffer by one digit.

#### The capacity was ten below its own ceiling

`INLINE_DIGITS`' doc gave the layout bound: `Number` must stay at 40 bytes or it widens `rexx-core`'s `Body` and with it every arena slot.
What it did not say is that the bound is not tight at twenty. `Digits` is an enum over a 24-byte `Vec`, so the vector arm sets its width until the inline buffer passes it. Measured with `size_of` across candidate capacities:

| `INLINE_DIGITS` | `Digits` | `Number` |
|---:|---:|---:|
| 20 | 32 | 40 |
| 23 | 32 | 40 |
| 24 | 32 | 40 |
| 30 | 32 | 40 |

**Twenty through thirty are the same object.** The capacity is now thirty, and the layout assertion in `the_capacity_is_the_one_the_language_asks_for` is joined by one pinning `size_of::<Digits>()` at 32, so a future capacity that does cost a word fails the test rather than passing quietly.

#### The residue nobody read

`long_divide` returned its remainder as a third value built with `split_off`, which allocates. Its one caller bound it and then discarded it with `let _ = rem;`, because a remainder good enough to report is recomputed at exact precision rather than read off the division -- the comment beside that line has always said so. The return value is gone.

#### The change that was measured and then taken out again

The obvious next step was to hold the division's working remainder in `Digits` rather than a `Vec`: it is scratch that never leaves the function, grown one digit at a time from empty, so a vector reallocates up its capacity ladder on every division. That change works, and it removes every allocation division makes at `DIGITS 9` and `DIGITS 20`:

| program | before | with `Digits` |
|---|---:|---:|
| `/` at DIGITS 9 | 3.02 | **0.02** |
| `/` at DIGITS 20 | 5.02 | **0.02** |
| `/` on two 20-digit operands | 6.02 | 2.02 |

**And it costs 3.104% of `arith`.** Four arms, five interleaved rounds each, minimum of each, on `arith`:

| arm | instructions | against base |
|---|---:|---:|
| base | 19,859,503,963 | -- |
| inline capacity only | 19,585,536,031 | **-1.380%** |
| `Digits` remainder only | 20,475,943,361 | **+3.104%** |
| both | 20,097,910,660 | +1.200% |

`Digits::push` and `Digits::as_mut_slice` branch on which arm holds the value, and the division's inner loop touches both per digit, where a vector hands out a pointer. **The allocations were the cheaper side of that trade**, and the combined arm would have shipped a 1.200% regression while removing 57% of the axis's allocations -- which is the clearest instance yet of the exchange rate entry 50 measured at about 45 instructions per allocation.

The remainder stays a `Vec`, and the reason is now a comment beside it so the next reader does not repeat the experiment.

#### What shipped

The inline capacity and the residue removal. `arith` falls **408,795 allocations to 253,128**.

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each. The head binary was rebuilt from the restored sources and compared byte for byte with the copy measured.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `arith` | 19,859,503,529 | 19,337,465,546 | -522,037,983 | **-2.629%** | 992 | 1,222 |
| `rexxcps` | 21,113,341,332 | 21,100,854,525 | -12,486,807 | -0.059% | 7,151 | 7,977,905 |
| `compound` | 18,237,906,046 | 18,238,726,672 | +820,626 | bound | 2,461,285 | 3,279,943 |
| `strings` | 40,017,586,636 | 40,017,586,055 | -581 | bound | 984 | 928 |
| `alloc4c` | 7,165,806,721 | 7,165,775,664 | -31,057 | bound | 217,487 | 202,788 |
| `emptyloop` | 24,975,609,250 | 24,975,609,180 | -70 | bound | 958 | 603 |
| `varlookup` | 42,123,633,904 | 42,123,633,093 | -811 | bound | 1,024 | 1,424 |

`rexxcps`' move is only 1.6 times its own wider span, which is the weakest claim in this table and is reported as the number rather than as a finding.

#### Behaviour

A sweep of every ordered pair from twenty-one values -- including 20-digit integers, values at both exponent extremes, halves that round either way, and zero -- across `/ * % // + -`, both signs, and twelve `NUMERIC DIGITS` settings from 1 to 100, with each operation trapped so an error becomes a printed line rather than an exit: **63,504 lines, byte-identical to the oracle on stdout, on stderr, and in exit status.**

**And the sweep was shown to fail before being trusted.** Seeding the division's working remainder with a stray digit makes it exit 101 and diverge on essentially every line. The sources were then restored from the backups, re-verified with `sha256sum -c`, rebuilt, and the sweep re-run against the oracle.

### Entry 53 -- the builtin name, resolved twice per call

Base `19f381125`. `strings` was the axis furthest from the oracle, and by this point it was allocation-free, so the cost had to be work rather than churn.

#### What the profile said

`perf record` on `bench-programs/strings.rex`, samples over 1.2%:

| | |
|---:|---|
| 17.94% | `run_ops::<false>` |
| 12.48% | `builtin::string::changestr` |
| 11.93% | `invoke_call` |
| 4.98% | `__memcmp_evex_movbe` |
| 3.48% | `hash_one::<&[u8]>` |
| 3.21% | `eval_node` |
| 3.14% | `builtin::string::pos` |
| 1.95% | `sip::Hasher::write` |

**Five percent in SipHash over byte slices, on a program that touches no stem and no symbol table at run time.** The only thing it hashes is builtin names.

#### Two lookups per call, and a doc that described the fix as already made

`Interp::resolve_call` asked `builtin::is_builtin(name)` -- a hash into the in-scope set -- and answered `Resolved::Builtin`, **a variant carrying nothing**. `builtin::dispatch` behind it then called `builtin::resolve(name)`, hashing the same name a second time into the row map, to decide which builtin.

`BuiltinTarget`'s own doc has said since it landed that a row index "names the one row and cannot disagree with it, and it is what lets a call site keep its answer". Nothing ever carried it. `Resolved`'s own doc stated the opposite as a design decision: "Which builtin is `builtin::dispatch`'s own lookup rather than something carried here". Both doc comments are now true of the same code.

`Resolved::Builtin` carries a `BuiltinTarget`. Resolution calls `resolve` instead of `is_builtin` -- **the same one lookup**, since every row's name is in scope (`every_implemented_row_names_an_in_scope_builtin`) and a name in scope with no row answers `Gap`, so `resolve(name).is_some()` and `is_builtin(name)` agree on every name -- and keeps what it found.

**A compiled call site records its `Resolved` against the op position**, so the second and every later execution of a builtin call now hashes the name **not at all**, where before it hashed once no matter how many times the site ran.

Two things fell out. `builtin::dispatch` had no production caller left and is now `#[cfg(test)]`, kept because six modules' unit tests reach builtins by name. And the arm in `invoke_call` that handled resolution and dispatch disagreeing about whether a name is a builtin is gone: with the row carried over, that case cannot be stated.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 40,017,586,271 | 36,870,641,821 | -3,146,944,450 | **-7.864%** | 1,409 | 1,171 |
| `alloc4c` | 7,165,837,984 | 6,909,768,300 | -256,069,684 | **-3.573%** | 127,800 | 102,455 |
| `rexxcps` | 21,100,844,360 | 20,758,930,368 | -341,913,992 | **-1.620%** | 4,076,567 | 4,205,617 |
| `compound` | 18,238,265,030 | 18,237,907,657 | -357,373 | bound | 3,742,356 | 1,641,927 |
| `arith` | 19,337,465,592 | 19,337,467,747 | +2,155 | +0.00001% | 917 | 1,590 |
| `emptyloop` | 24,975,609,380 | 24,975,611,114 | +1,734 | +0.00001% | 733 | 1,054 |
| `varlookup` | 42,123,633,279 | 42,123,635,594 | +2,315 | +0.00001% | 2,261 | 1,747 |

The last three exceed their own spans and are reported rather than called bounds, but each is about one part in ten million and none of those axes calls a builtin.

**The largest single change in this series**, and it removed no allocation at all -- which is worth setting beside entry 52, where removing 57% of an axis's allocations cost 3.104%.

#### Behaviour

A probe over the resolution order this touched -- a builtin called as a function and as a `CALL`, a `::ROUTINE` sharing a builtin's name, the quoted lowercase and uppercase spellings that select the routine and the builtin respectively, an unknown name reaching 43.1, and two builtins sharing one row (`CENTER`/`CENTRE`) -- is **identical to the oracle on stdout, on stderr, and in exit status**. The string and argument probes from entries 48 and 50 were re-run against this binary and are identical too.

The first draft of that probe also called `STREAM` and `RXFUNCADD`, and this crate exits loudly on a Phase 4 declared gap rather than raising a condition a `SIGNAL ON SYNTAX` could trap, so it stopped there. Every line it did produce matched the oracle. The gap is Phase 7's and not this entry's; the two calls were removed rather than worked around.

**And the probe was shown to fail before being trusted.** Making resolution carry `BuiltinTarget::Gap` instead of the row it found makes it exit 120 on the first line with `routine "MAX" is not implemented (4c)`.

#### Where the axis stands

`strings` against the oracle, three interleaved rounds, wall clock and therefore orientation rather than this phase's instrument: **3.57x to 3.05x**. `changestr` and `pos` themselves are the next thing on it, at 12.48% and 3.14% of samples.

### Entry 54 -- a `memcmp` at every position, and a `POS` divergence found beside it

Base `5b632d60b`.

#### The change

`find_forward` is `window.windows(needle.len()).position(|candidate| candidate == needle)`.
On byte slices `==` is a `memcmp` call, so the naive form makes one **at every position the needle could start at**. `perf` on `bench-programs/strings.rex` put `__memcmp_evex_movbe` at 4.98% of samples, above `pos` itself at 3.14%.

The candidate's first and last bytes are compared before the whole. The first alone would do; the last costs one more load only on candidates that pass the first test, and rules out the needles whose interior repeats.

`find_backward` beside it has the identical shape and is not changed, because nothing measured here reaches it.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 36,870,642,238 | 32,925,639,260 | -3,945,002,978 | **-10.700%** | 615 | 1,694 |
| `rexxcps` | 20,758,925,894 | 20,758,922,067 | -3,827 | bound | 4,080,650 | 7,386,242 |

Three lines, and the largest single figure in this series.

#### Behaviour, and what the probe found that was not this change

A probe over the search primitives -- ten needles against a haystack chosen to repeat them, each with `POS`, `LASTPOS`, `COUNTSTR` and two `CHANGESTR` forms, swept across eight start positions and five ranges, plus the empty needle, the empty haystack, overlapping needles, a replacement longer and shorter than the needle, a count limit of zero, `WORDPOS` and `VERIFY` -- is **518 lines, and identical to this entry's base on every one of them**, which is what says this change is behaviour-neutral.

**Against the oracle it differs on one line, and the base differs on the same line.** `pos('an', 'banana bandana abracadabra', 6, 4)` is 9 on the oracle and 0 here. The window is positions 6 through 9, `"a ba"`; the match at 9 runs into position 10, outside it. **The oracle bounds where a match may begin; this crate requires the match to fit.**

That is a real divergence and it is not this entry's to fix -- it is recorded here because this probe is what found it, and because `find_backward`'s own doc comment carries the *opposite* rule for `LASTPOS`, measured against the oracle at the time: "the match has to end within the window, not merely begin there". The two builtins do not share a rule, and only one of them has it right.

**And the probe was shown to fail before being trusted.** Dropping the full comparison and keeping only the two end-byte tests makes fourteen of its lines diverge. The source was then restored from the backup, re-verified with `sha256sum -c`, rebuilt, and the probe re-run to the byte.

### Entry 55 -- the builtin call path stops building arguments it never wanted

Base `52104500b`. Entry 54 moved `changestr` from 12.48% of `strings` samples to 6.50% and took `memcmp` out of the profile entirely; what surfaced underneath was `invoke_call` at **14.76%**, above every builtin it dispatches.

#### The change

Every call built a `Vec<Option<Argument>>`, and the builtin arm then copied it into a `Vec<Option<ObjRef>>` to hand over. **A builtin has no use for an `Argument`**: its `Reference` variant carries the caller's slot and the variable's spelling for `USE ARG >`, which no builtin has. So the builtin path was building a larger value, moving it into a buffer, walking the buffer, and copying the one field it needed out of each.

The builtin path now has its own loop, evaluating straight into the value buffer and returning before the general path starts. The per-argument step -- evaluate, root, trace -- is shared as `eval_traced_argument`, so the `>A>` lines and the `>O>` line a `>p` reference argument traces are the same on both paths by construction rather than by two copies agreeing.

**And `argument_buffer` is gone with it.** Entry 48 lent that buffer to every call; only the builtin arm ever handed it back, because a label or routine callee keeps the arguments it is given. With the builtin path no longer building `Argument`s at all, nothing returned it, so the general path was taking an always-empty `Vec` and allocating into it -- a pool with no source. It is a plain `Vec::with_capacity` again, and the field, its two accessors and their doc comments are removed. The lending that entry 48 measured was two buffers; one of them is now unnecessary rather than merely unused.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 32,925,639,585 | 32,571,639,859 | -353,999,726 | **-1.075%** | 542 | 1,038 |
| `alloc4c` | 6,909,807,492 | 6,862,793,486 | -47,014,006 | **-0.680%** | 143,207 | 119,009 |
| `rexxcps` | 20,758,919,921 | 20,717,500,441 | -41,419,480 | -0.200% | 7,030 | 11,481,785 |
| `compound` | 18,238,627,980 | 18,237,904,979 | -723,001 | bound | 2,556,500 | 2,459,290 |
| `arith` | 19,337,465,223 | 19,341,750,679 | **+4,285,456** | +0.022% | 717 | 932 |
| `emptyloop` | 24,975,609,086 | 24,975,608,981 | -105 | bound | 956 | 592 |
| `varlookup` | 42,123,633,398 | 42,123,633,803 | +405 | bound | 907 | 776 |

**Smaller than 14.76% of the axis would suggest, and that is the point of measuring rather than reading a profile share**: most of what `invoke_call` costs is evaluating the argument expressions, which this change does not touch -- it removes the container they were being put into.

`arith` calls no builtin and regressed outside its spans, the third entry running to show a movement of that size with no mechanism on that axis. The do-nothing control this record has wanted since entry 31 is now the single most useful measurement not yet made.

#### Behaviour

Six probes re-run against this binary: the argument probe (entry 48, omitted positions, nested calls, `USE ARG >` references, `TRACE I`), the string probe (entry 50), the `NUMERIC` probe (entry 51), the arithmetic sweep (entry 52, 63,504 lines), the resolution probe (entry 53) and the search probe (entry 54, 518 lines). **All six are identical to the oracle**, except the search probe's one known line -- `pos('an', h, 6, 4)`, the divergence entry 54 recorded, which is present in every build tested and is not this change's.

### Entry 56 -- the `PARSE` trigger operand, copied for a line that rarely prints

Base `7ecd6f510`. `PARSE` is the largest identifiable cause of allocation in `samples/rexxcps.rex`: at that base, of 539,048 allocations, `exec_parse` accounted for about 123,000 -- 95,205 through `parse_strings` and 28,001 through `apply_trigger`.

#### The change

`apply_trigger` copied the operand's rendering with `to_vec()` on **every** trigger, then handed it to `trace_result`, which returns at once unless `results` is on, and to `Cursor::search` for the two string-shaped kinds.

The copy is now made only where one is needed, which is neither of the hot arms:

* the trace is behind `self.trace_mode().results`, the same gate `trace_result` itself applies;
* a search reads the bytes where they already are, because `Cursor` is a local of `exec_parse` rather than a field of the `Interp`, so a shared borrow of the value can be live while the cursor is written to;
* the one owned rendering left is the 26.4 raise's substitution, on a path that ends the clause.

**`rexxcps`' templates are mostly numeric triggers**, which never looked at the bytes at all.

#### Allocations and instructions

`heaptrack` on the pinned program: **539,048 to 505,447**.

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `rexxcps` | 20,717,483,212 | 20,602,987,622 | -114,495,590 | **-0.553%** | 8,523 | 4,064,252 |
| `compound` | 18,238,265,878 | 18,237,905,198 | -360,680 | bound | 1,283,157 | 1,642,936 |
| `strings` | 32,571,639,348 | 32,571,639,794 | +446 | bound | 1,260 | 713 |
| `alloc4c` | 6,862,811,139 | 6,862,813,371 | +2,232 | bound | 113,453 | 111,420 |
| `arith` | 19,341,750,901 | 19,341,750,872 | -29 | bound | 1,154 | 1,343 |
| `emptyloop` | 24,975,608,710 | 24,975,609,628 | +918 | bound | 1,183 | 420 |
| `varlookup` | 42,123,633,376 | 42,123,632,994 | -382 | bound | 1,335 | 1,421 |

Every axis that executes no `PARSE` is a bound, which is the control this change wants.

#### Behaviour

A probe over the trigger kinds -- a string pattern, a caseless one, absolute, relative forward and back, both length forms, a parenthesised variable operand in two positions, the placeholder period, a comma fence, `UPPER` and `LOWER`, `PARSE VALUE`/`VAR`/`SOURCE`/`VERSION`/`ARG` with an omitted argument, and the 26.4 raise -- was run **twice, once plain and once under `TRACE R`**, because the gate this change adds is exactly the difference between those two runs. Both are identical to the oracle on stdout, on stderr and in exit status, the traced arm across 162 lines of trace output.

#### What is left on this path, and it is the larger half

`parse_strings` allocates twice per `PARSE` clause and neither is touched here: an owned copy of the source string, which `Cursor` holds across the calls back into `&mut self` that the assignments make, and the one-element `Vec` returned to carry it. About 95,000 of the pinned program's remaining 505,447.

The copy could be lent and handed back at the end of the clause, and the outer `Vec` could be a lent buffer indexed rather than consumed by `into_iter`. Both are real changes to `exec_parse`'s shape rather than gating, and neither is attempted here.

### Entry 57 -- the do-nothing control, and what `arith` has been doing

Base `34ded1fed`. Entries 47, 48 and 55 each recorded an axis moving outside its own spans with no mechanism on that axis, and each said so without being able to say what the movement was worth. This entry measures the floor those readings needed.

#### What the control is

**A field on `Interp` and its two accessors, present, initialised, and called from nowhere.** `parse_buffer: Vec<u8>` beside `result_buffer`, `take_parse_buffer`/`give_parse_buffer` beside the `PARSE` code, both `#[allow(dead_code)]`. It is the exact shape of every buffer-lending change in this series, with the lending removed: the struct grows, every field after it moves, the initialiser and the drop glue grow, and no instruction on any benchmark path is added or removed.

**Two earlier controls are ruled out first, by measurement rather than by argument.**

* **A rebuild is not a control.** Touching a source file and rebuilding gives a **byte-identical** binary -- `1b6c59bf085e788265ef571d633b8004da1ffed81da448cdbab640e53c6697b7` before and after, and again after the control edit was reverted. Under `lto = "fat"` with `codegen-units = 1` this build is reproducible, so a no-op *edit* measures nothing.
* **The control edit does change codegen**, which is what makes it a control rather than a second copy of the same binary: `.text` goes from 994,921 to 995,001 bytes.

Its stdout, stderr and exit status on `arith` are identical to the arm it is a control for, which is the claim that it does nothing.

#### The floor, per axis

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each.

| axis | head | null control | difference | | span head | span null |
|---|---:|---:|---:|---:|---:|---:|
| `arith` | 19,341,750,567 | 19,337,465,108 | **-4,285,459** | **-0.022%** | 1,424 | 1,173 |
| `alloc4c` | 6,862,766,886 | 6,862,701,437 | -65,449 | -0.001% | 146,760 | 182,706 |
| `compound` | 18,237,906,590 | 18,238,265,993 | +359,403 | +0.002% | 3,179,477 | 2,459,291 |
| `rexxcps` | 20,602,982,299 | 20,602,981,098 | -1,201 | bound | 7,144,790 | 4,072,693 |
| `strings` | 32,571,639,760 | 32,571,639,126 | -634 | bound | 1,289 | 1,381 |
| `emptyloop` | 24,975,608,923 | 24,975,609,326 | +403 | bound | 738 | 1,112 |
| `varlookup` | 42,123,633,679 | 42,123,633,691 | +12 | bound | 462 | 1,283 |

#### `arith` has two states and nothing semantic chooses between them

The control's `arith` figure is not noise and it is not small: **-4,285,459 instructions against a within-arm span of about 1,200**, and it is the same on every round. Five rounds of the head arm all read 19,341,75x,xxx; five rounds of the control all read 19,337,46x,xxx. The axis is bimodal, deterministic per binary, and the two states are 4.285 million apart.

**Entry 55 measured that difference and reported it as a regression.** Its `arith` line was base 19,337,465,223, head 19,341,750,679, **+4,285,456** -- the same two states, the same gap, to the last hundred thousand of the third significant figure. The control moves the identical distance in the opposite direction with no mechanism whatsoever.

So entry 55's `arith` line is **not attributable**, and this entry withdraws the reading rather than the change: the change stands, and the sentence claiming a 0.022% cost on an axis that calls no builtin does not. Entries 47 and 48 recorded movements on the same axis at 4.3M and 1.0M; the first is this artifact and the second is inside the same floor.

#### What the floor is for

**On `arith`, nothing smaller than 4.3 million instructions means anything**, whichever direction it points, and a change that touches `Interp`'s size should be *expected* to move it. The next `PARSE` change adds a field to `Interp` and will land in the low state; that is a bound and not a saving.

The other six axes give their run-to-run spans and no second state: `strings`, `emptyloop` and `varlookup` resolve to about a thousand instructions in tens of billions, and `alloc4c`, `compound` and `rexxcps` are bounded by their own spans, which are large for their own reasons and were large before this control.

**What this does not license.** A movement larger than an axis's floor is still not automatically the mechanism you have in mind -- the floor is a necessary bar, not a sufficient one. And this is one control of one shape: it says what a struct-width perturbation is worth, not what every layout perturbation is worth.

### Entry 58 -- `PARSE`'s two allocations, one of which was worth having

Base `e71794398`. Entry 56 named this the larger half of `PARSE` and `2026-08-14-parse-source-buffers.md` carries the plan: `parse_strings` allocated twice for every clause, an owned copy of the source string and the one-element `Vec` that carried it, together about 95,000 of the pinned `samples/rexxcps.rex`' 505,447 allocations.

**Both halves were built, and only one of them is kept.**

#### The half that ships: the container

`parse_strings` returned `Vec<Vec<u8>>` and `exec_parse` consumed it through `into_iter`. **Every source but `ARG` produces exactly one string**, so that was an allocation per clause for a container never asked to hold a second element.

It returns a `ParseStrings` now: `One(Option<Vec<u8>>)` for the single-string sources, `Many(std::vec::IntoIter<Vec<u8>>)` for `PARSE ARG`, which is the one source with more than one string and keeps the `Vec` it has to build anyway. A template past the first takes an emptied slot and gets `None`, which is exactly the null-string rule `next_template` already wanted from the iterator running out.

#### The half that does not: lending the source copy

A `parse_buffer` field on `Interp` in the shape `key_buffer` uses, taken in `parse_strings` after the source is evaluated, handed to the `Cursor`, and given back at the end of the clause and at every comma fence -- with `Cursor::into_string` added to get it out again.

**It works, it removes the allocations it was built to remove, and it does not pay.** It is measured below and reverted.

#### Allocations, `heaptrack` on the pinned program

| arm | allocations | |
|---|---:|---:|
| base | 505,446 | |
| container only | 466,246 | -39,200 |
| container and lending | 432,646 | -33,600 |

#### Instructions

Five interleaved rounds per arm, all three arms staged at one fixed binary path, minimum of each. The two changes were measured **separately and in combination**, which is entry 52's rule and is what decided this entry.

| axis | base | container | lending too | base to container | | container to lending | |
|---|---:|---:|---:|---:|---:|---:|---:|
| `rexxcps` | 20,602,982,072 | 20,512,857,013 | 20,521,367,273 | **-90,125,059** | **-0.437%** | +8,510,260 | +0.041% |
| `compound` | 18,238,626,507 | 18,238,267,543 | 18,237,905,600 | -358,964 | bound | -361,943 | bound |
| `alloc4c` | 6,862,847,429 | 6,862,714,945 | 6,862,722,412 | -132,484 | bound | +7,467 | bound |
| `arith` | 19,341,750,662 | 19,341,751,078 | 19,337,465,189 | +416 | bound | **-4,285,889** | **artifact** |
| `strings` | 32,571,639,072 | 32,571,639,419 | 32,571,639,729 | +347 | bound | +310 | bound |
| `varlookup` | 42,123,633,548 | 42,123,633,317 | 42,123,633,936 | -231 | bound | +619 | bound |
| `emptyloop` | 24,975,609,170 | 24,975,608,961 | 24,975,609,157 | -209 | bound | +196 | bound |

Spans, in the same order of arms: `rexxcps` 1,474,888 / 3,327,572 / 5,424,389; `compound` 925,335 / 2,100,515 / 2,820,599; `alloc4c` 99,915 / 209,525 / 265,716; the other four are under 1,100 throughout.

**The container is worth -0.437% of `rexxcps` and the lending is worth nothing.** The second arm removes 33,600 more allocations than the first and comes out 8.5 million instructions *behind* it. At entry 50's exchange rate those allocations were worth a few million; the take, the give, the `mem::replace` at the fence and the extra field cost at least that back.

Whether the lending arm truly *costs* is not attributable, and the previous entry is why: it adds a field to `Interp`, and entry 57's control moved an axis 4.3 million instructions by adding a field to `Interp` and nothing else. The honest reading is that it does not pay, which is enough to revert it and is a smaller claim than saying it is slower.

#### Entry 57's control predicting a reading before it was taken

Entry 57 said the next `PARSE` change would add a field to `Interp`, would land `arith` in its low state, and that this would be a bound rather than a saving. **It did, and the split measurement shows it cleanly**: the container arm changes no struct and moves `arith` by +416; the lending arm adds the field and moves it by -4,285,889. The same 4.285 million, on the arm that has the mechanism the control identified and not on the arm that does not.

That is the control being used the way it was built to be used, rather than a movement being explained after the fact.

#### One refinement to entry 57, from a thing found here

Entry 57 says a no-op edit measures nothing. **A doc-comment edit does change the binary** -- correcting one comment in this file gave a different `sha256` -- because `debug = true` puts line tables in and the comment moves every line after it. Its `.text` is byte-identical, which is why the claim about *instructions* stands and why `.text` rather than the file is what a control should be compared on.

#### Behaviour

Four probes, all byte-identical to the oracle on stdout, on stderr and in exit status: entry 56's `PARSE` probe plain and under `TRACE R`, and a new probe written for this change's own hazard -- a comma fence on a single-string source, a trigger operand whose expression calls a routine that itself parses, two levels of that nesting, a source expression that parses, the same clause repeated in a loop, a raise between a take and a give followed by a clause that has to work anyway, and a long source followed by a short one -- also plain and under `TRACE R`.

**And the new probe was shown to be necessary, not merely able to fail.** Making `ParseStrings::One` clone its slot instead of taking it diverges on the new probe and its traced twin, and **entry 56's probe does not notice at all** -- `1[alpha][beta GAMMA delta 42 epsilon][alpha][beta GAMMA delta 42 epsilon]` where the oracle gives two null strings. The lending arm got its own witness while it existed: dropping the `clear()` from `take_parse_buffer` diverges on all four probes. Both mutations were reverted from a backup, `sha256sum -c`'d, rebuilt, and the probes re-run to the byte.

#### What is left on this path

`PARSE ARG` still allocates its `Vec` and a copy per argument, and that is the one source where the `Vec` is doing real work. The source copy is still made for every clause; it is not obviously worth removing, and the arm that removed it is measured above.

### Entry 59 -- the last free tag, and two thirds of every string leaving the heap

Base `91b80a608`. The largest single movement in this series, and it came from asking what the heap is actually asked to hold rather than from making any part of it faster.

#### What the measurement found first

Every allocation on every axis, instrumented and counted by `Body` variant and by text length:

| axis | allocated | Text | Num | Stem | text p50 | p90 | max | over `INLINE_BYTES` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `rexxcps` | 515,444 | 92.5% | 6.4% | 1.1% | 3 | 21 | 65 | 7 |
| `strings` | 18,000,001 | 100% | -- | -- | 3 | 46 | 46 | 0 |
| `alloc4c` | 2,000,001 | 100% | -- | 1 | 4 | 10 | 11 | 0 |
| `arith` | 3,469,314 | -- | 100% | -- | -- | -- | 20 digits | 0 |
| `compound` | 1 | -- | -- | 1 | -- | -- | -- | 0 |

The arena is 65,536 slots, 6.29 MB, on every one of them.

**Two readings, and the second is the change.** The spill path barely exists -- seven values out of 476,598 exceed the inline capacity on `rexxcps` and none do anywhere else, so the uniform slot is not paying for long strings. It is paying **96 bytes to hold a three-byte median payload**. And the distribution is bimodal rather than smooth: `strings` produces texts of 3 bytes and of 46, nothing between.

#### The change

`ObjRef`'s tag had three kinds and four values. `0b11` named nothing, and now names a byte string carried in the handle: three bits of length and seven bytes of payload, with three bits over. **Nothing is taken from the slot index or the generation**, so neither budget moves.

`Interp::text_bytes` is the single point a `Body::Text` is built, so the interception is one branch in one function. A string short enough costs no slot, no mark byte and no sweep pass.

**The three sites that had to change were named by the compiler, not by a search.** There are 34 `Decoded::` matches in the tree; making the enum non-exhaustive broke `to_text`, `try_text` and `to_number`, and nothing else.

* `try_text` answers `None`, for the reason it already answers `None` for a small integer: there is nothing outliving the call to borrow from.
* `render`/`Rendered` already existed for exactly that case, so a caller wanting two operands' bytes at once copies into its own `Rendered`. Its `owned: Option<Vec<u8>>` becomes a three-arm `Carried`, and the inline arm allocates nothing.
* `to_text` copies into a scratch field and hands back a borrow of it. **One slot is enough because `to_text` takes `&mut self`**: the borrow it returns holds the interpreter exclusively, so no second call can run to overwrite it while the first is live. The signature that makes that function awkward to call is the same one that makes a single slot sound, and the compiler enforces it rather than a convention.

What is given up is the `num` parse cache, because a handle is `Copy` and has no shared mutable home for a lazy fill. That was measured before it was given up: `to_number` on a `Body::Text` is called 117,205 times on `rexxcps` and **zero times on `strings`, `arith`, `compound` and `varlookup`**, and of those 117,205 only 22,400 hit a cache the other 94,805 had filled.

#### Instructions

Five interleaved rounds per arm, both arms staged at one fixed binary path, minimum of each.

| axis | base | head | difference | | span base | span head |
|---|---:|---:|---:|---:|---:|---:|
| `strings` | 32,571,638,992 | 30,471,323,584 | **-2,100,315,408** | **-6.448%** | 1,962 | 274 |
| `rexxcps` | 20,512,852,132 | 19,250,488,265 | **-1,262,363,867** | **-6.154%** | 3,345,895 | 4,093,074 |
| `alloc4c` | 6,862,773,537 | 6,466,233,994 | -396,539,543 | **-5.778%** | 194,977 | 200,100 |
| `varlookup` | 42,123,633,322 | 41,800,633,586 | -322,999,736 | -0.767% | 1,185 | 694 |
| `compound` | 18,238,266,421 | 18,073,255,843 | -165,010,578 | -0.905% | 823,277 | 2,101,343 |
| `emptyloop` | 24,975,608,723 | 24,925,608,275 | -50,000,448 | -0.200% | 1,072 | 1,307 |
| `arith` | 19,341,751,036 | 19,314,768,132 | -26,982,904 | -0.140% | 944 | 874 |

**Every axis improves, including the four that were bounds for every change in this series.** `varlookup` and `emptyloop` move 323 million and 50 million instructions against spans of about a thousand -- and both were flat through entries 45 to 58.

`arith` lands on a **third** state: entry 57 measured its two at 19,341,75x and 19,337,46x, and this reads 19,314,768,132, some 27 million below either. This change adds a field to `Interp`, so entry 57's 4.3 million artifact is inside this figure and cannot be separated from it; what can be said is that the movement is six times the artifact's size, not that all of it is work.

`heaptrack` on the pinned `rexxcps` reads 466,244 to 410,615, and **that number understates the change by a lot**: a short `Bytes` was already inline in its slot, so it cost no `malloc` at all. What this removes is slots, mark bytes and sweep passes, which `heaptrack` does not count.

#### Behaviour

**1493 tests pass, and six had to be restated first -- every one of them asserting the old representation.**

Three used "this is a heap object" as a proxy for something else. `a_counted_answer_is_tagged_rather_than_a_heap_string` is the clearest: it pins that `SUBSTR('012345',1,3)` is *not* a counted answer, because `012` is its own bytes and no `SmallInt` renders it. That rule is untouched; the spelling asserted the storage, so it now asserts "not a `SmallInt`" instead.

Three said "this program allocates" and no longer did. **The GC stress fixtures went quiet rather than red** -- `zz = 'x' || k`, `value('a.j')`, `yy = 'abc'` all stopped reaching the heap, so collect-on-every-allocation had nothing to fire on. `collect_policy.rs` had already been bitten by this once and says so in its own header: the tails are `'v' || i` rather than `i` "because a small integer is a tagged immediate and never reaches the heap at all -- this file would be measuring nothing." Same hazard, second immediate. Every literal in those fixtures is now wider than the handle's capacity, each new expected output checked against the oracle, and the reason is written beside the widths.

The committed list of subset programs that allocate nothing grew from four entries to nineteen. That assertion exists to force a decision when it moves, and the decision is that short strings are most strings.

Five probes against the oracle are byte-identical on stdout, stderr and exit status: the `PARSE` probe plain and traced, the nesting probe plain and traced, and the argument, number, stem and string probes. The search probe differs on **one** line, `pos('an', h, 6, 4)` -- entry 54's divergence, and the base arm was run on the same probe and differs identically, which is what says it is not this change's.

**Two mutation witnesses, and the second is the one worth having.** Decoding the length as the capacity rather than reading the field fails 337 tests and takes an allocation unbounded, which `memcap 8G` caught at rc 137 rather than the machine. Refusing the last admissible length -- `>` to `>=`, so a seven-byte string takes a slot again -- **changes no output anywhere** and is still caught, by the encoding's boundary test, by `a_value_short_enough_is_the_handle_and_has_no_slot`, and by the zero-collection list. A representation change that no behaviour can see is exactly what this series risks shipping unnoticed, and those three assertions are what see it.

#### What is deliberately not done

The other free space is `0b10`: `NIL` is one value occupying a whole 62-bit tag, and tightening `decode` to "zero payload is `NIL`" would open a second inline kind -- five bytes plus a seventeen-bit value, which is the widest split that lets the value field cover any string the byte field can hold. That would restore a parse cache for short numeric strings. It is not built here because the cache it would restore was measured at 22,400 hits on one axis and none on four, and because this change is worth measuring alone before a second one is laid on top of it.

### Entry 60 -- fourteen changes on a different instrument, and the axes as the control

**Read the instrument note before the numbers.** This entry does not use the harness the configuration block above pins, and none of its figures is comparable with entries 1 to 59.

| | this entry | the pinned configuration |
|---|---|---|
| instrument | `perf stat -e instructions:u,cycles:u` | wall clock |
| harness | `perf stat` over one process, plus `rexx-arms` | `rexx-bench-suite` |
| reference | the immediately preceding build of this crate | the oracle |
| axes | a pinned-count copy of `samples/rexxcps.rex`, plus `arith`, `varlookup`, `compound`, `strings` | the six classic-Rexx axes |

**So nothing here is a claim against the bar.** No oracle comparison was taken in this sitting. What every figure below says is "this build against the one before it", which is the loop's accept rule, on an instrument the accept rule does not name. The reason for the swap is in the first finding.

The base is `e81d3497db2688adb5f585af57303180c418892f`; the head is `5775ea7d422403f15de7544e531290d663056e65`. Instructions retired on the pinned `rexxcps`: **18,885,258,713 to 14,405,866,386, -23.7%**.

#### Finding 1 -- wall clock could not resolve these changes, and `rexxcps` auto-scales

The sitting began on wall clock and abandoned it. A change measured at +2.3% median was ahead in only **6 of 10** alternating pairs, against an 11% spread within that same sitting; the machine was noticeably noisier than the one entries 1 to 59 were taken on. Retired instructions have a 0.02% spread between runs of one binary here, which is what made a 0.4% change decidable at all.

**`samples/rexxcps.rex` cannot be measured on instructions as it ships.** Line 163, `count=(1%total + 1) * count`, scales its trial-2 loop bound by its own measured time, so a faster build runs *more* of it and the counts do not compare. Every figure below is against a scratch copy with that line pinned, which makes the work fixed and the count meaningful. The reported clauses-per-second is still the right metric for the program as shipped; it is the instrument that had to change, not the program.

#### Finding 2 -- the fixed-work axes are the control, and they killed two changes

**Three separate changes read as a clear win on `rexxcps` and a regression on every fixed-work axis.** The axes caught all three; `rexxcps` alone would have shipped all three.

* **`Rc<TrapTable>` for `caller.traps`, discarded.** `invoke_call` clones the caller's trap table per call; 117 of `int_malloc`'s 131 samples came from that one stack, and the inheritance is documented one-way, so copy-on-write fits. Measured: `rexxcps` +0.48% instructions but **-4.56% cycles**, against `varlookup` +1.2/+1.7% instructions and **+5.4/+12.5% cycles**, `compound` +1.0/+1.4%, `strings` +0.7/+0.9%. A shared empty singleton was tried to remove the `Rc::default()` allocation per activation; it moved `arith` slightly and left `varlookup` and `compound` **identical**, which is what says the cost is the indirection on reads and not the allocation. Discarded. The ceiling it was chasing is still there: 94 samples, 1.0%.
* **Rendering a tagged integer through `i64`'s `Display` into a fixed `fmt::Write` sink, discarded in favour of a digit loop.** Allocation-free and states the contract directly, but reaches the buffer through `core::fmt` where `to_string` has a specialised integer path: 17,415,036,226 against the 17,272,447,974 it replaced, **+0.85%**. The digit loop that replaced it read -4.49%.
* **Widening `arith_small_int` to an operand that spells an integer, rescued by outlining.** Folded into the caller: `rexxcps` -1.70%, every fixed-work axis between +0.3% and +2.2%. With the tagged pair kept in its own arm and the widening behind `#[inline(never)]`: `rexxcps` **-2.53%**, and the IR arm improves on all four axes. The same trick had already worked on `to_number`.

**The general rule this sitting earned:** a change that helps one program and costs the fixed-work axes is usually a function that got too big to inline, not a bad idea. Try outlining the new work before discarding it -- and if outlining does not rescue it, discard it, because `rexxcps` is one program and the axes are the ones the bar is stated against.

#### Finding 3 -- the two instruments disagree in sign, reproducibly

Removing an allocation or a call into libc reads as **more instructions and fewer cycles**; `Rc<TrapTable>` (+0.48% / -4.56%) and the `memcpy` removal (-0.37% / -3.06%) both do it. `malloc`, `free` and a PLT call into a dispatching `memcpy` are few instructions and many cycles. Neither instrument alone would have read those two changes correctly, and the entries below say which instrument each result rests on rather than quoting whichever moved further.

#### Accepted, in order

| commit | change | instructions on pinned `rexxcps` |
|---|---|---:|
| `d6870a358` | `Cow` for `truncated_to`/`round_to`, `into_round`, `check_range(&self)`, inline hints | -2.5% *(wall clock; taken before the instrument swap)* |
| `5a622ac9b` | `Interp::text` asks whether the handle fits **before** building a `Bytes` | -1.52% |
| `aa1eaf1ce` | `Number::parse_bytes`; four callers drop a `from_utf8` guard | -1.45% |
| `54e695632` | `magnitude_order` -- same-signed operands ordered by digits, not by subtracting | -5.74% |
| `dec900730` | a tagged integer rendered into storage the reader already owns | -4.49% |
| `17c3ae4e6` | six small hot-path helpers marked `#[inline]` | -1.08% |
| `089b6c344` | a condition decided from the handle when the handle carries its bytes | -1.09% |
| `838668429` | `FxHash` for `Plan::names` and `Body::Stem`'s tails | -2.21% |
| `a84a3c62e` | a literal's digits validated and accumulated in one pass | -2.78% |
| `4ebf699d5` | `ObjRef::inline_text` copies by a bounded loop, not `memcpy` | -0.37% *(and -3.06% cycles)* |
| `262fa6d68` | `to_number`'s arena arm outlined so the tagged arms inline | -1.13% |
| `d9599bda4` | `assemble` scans once and `drop_front(0)` returns | -0.84% |
| `633dc747c` | a builtin's integer argument answered from the tag | -1.40% |
| `5775ea7d4` | the integer path taken when an operand *spells* an integer | -2.53% |

**One shape accounts for four of them.** `from_utf8` or `str::parse` run over bytes already known to be ASCII digits: `Number::parse` (`aa1eaf1ce`), `to_number`'s three arms (same), `compare.rs`'s own `parse_bytes` helper (same), and `canonical_small_int` (`a84a3c62e`, the most expensive at 169 samples on one line). Worth grepping for the fifth.

**`a84a3c62e` is the only change in the sitting that moved every fixed-work axis** -- `varlookup` 0.965, `compound` 0.976, `arith` 0.983, `strings` 0.991 -- because a literal is evaluated whatever the program is doing.

#### Two claims a test falsified

**`whole_i64`'s doc comment was wrong when written.** It said a value the fast test defers is one `whole_value` still converts by rounding, citing `1234` at `digits` 3 answering `1230`. The test written to assert that **failed**: `whole_value`'s rounding branch applies the same ceiling, so for an `i64` input the two agree on `None` as well as on `Some`. The doc now says that and the test holds both outcomes.

**`1.000000000` is not equal to `1` under the comparison shortcut**, and the first version of `a_trailing_zero_is_not_a_difference_but_a_trailing_digit_is` asserted it was. Ten digits is one past the working precision, so the shortcut correctly declines and defers to the subtraction. The code was right and the expectation was wrong.

#### Environment, for the next sitting

* **`memcap` is not installed on this machine and cannot be reconstructed**: `/sys/fs/cgroup` is empty, so no cgroup can be made. Every `cargo test --workspace` in this sitting ran **uncapped**, contrary to `CLAUDE.md`'s gate.
* **`cargo --offline` twice downgraded `crunchy` 0.2.4 to 0.2.2** in `Cargo.lock` as a side effect of resolving a new dependency; restored both times with `cargo update --offline -p crunchy --precise 0.2.4`.
* **`838668429` is the workspace's first external runtime dependency.** Everything before it was a path dependency plus `criterion` as a dev-dependency. `seahash` was tried first and is **worse than the `RandomState` it replaces** (+0.22%); `foldhash` reads 0.018% better than `rustc-hash` and carries thirteen `unsafe` blocks against its none.
* The gated suite ran 1604 to 1606 passing with **30 failing throughout, the same 30 by name at every step**: 26 are a missing `ootest/` checkout and the rest are pre-existing `gc`/`condition`/`trace` differentials. Every entry above was checked by diffing the failure-name list against the previous build's, not by reading the count.

#### The queue this leaves, with ceilings

* **`Op::Generic` falls back to the tree-walker, 17.6% of the thread through `step_in_temps_frame`.** `compile.rs` has native ops for `Do`/`Loop`, `If`, `Select`, `When`, `Assignment`, `Say`, `Return`/`Exit`, `Push`/`Queue`, `Call::Named`, `Message` and `Expose`; everything else delegates. In `rexxcps`'s inner loop that is `ITERATE`, `LEAVE`, `NOP`, `PARSE VAR` and labels. The first three are cheap to promote -- no expression, no register, one control-flow edge.
* **The arena and the collector, about 8.5%.** `alloc_with` 5.8% total, `Heap::collect` 2.7%.
* **`caller.traps`, 1.0%**, with the `Rc` route closed. Two shapes not tried: move-with-write-back on first mutation, which keeps reads as direct field accesses; or making the clone allocation-free by giving `Trap::label` an `Rc<[u8]>` and indexing the fixed condition set by an enum, which leaves the eager clone alone.
* **Chunking `Heap::slots` into `Vec<Vec<Slot>>` with power-of-two inner lengths** (Moritz's suggestion) so growth never moves a slot. **Measure the growth cost first**: every slot access would gain a dependent load, `Heap::get`/`get_mut` sit under nearly everything, and three changes in this sitting were decided by exactly that kind of per-access tax showing up on the fixed-work axes. Nothing here separates *growth* from *sweeping* inside `Heap::collect`'s 2.7%, and `rexxcps` reaches steady state early. The larger prize if slots stop moving is that handles may not need generation validation per deref and the sweep could go chunk-wise -- a change to the arena's contract, not its layout.

### Entry 61 -- promoting the clauses that compute nothing

`6da78f62c` (`NOP`, `THEN`, `LEAVE`, `ITERATE`, plus `rexx-ir`) and `5e40705d7` (`LABEL`), against `94c4f6464`.

**Same instrument caveat as entry 60**: `perf stat -e instructions:u,cycles:u` and `rexx-arms`, not the pinned wall-clock `rexx-bench-suite` against the oracle. Nothing here is a claim against the phase bar.

#### What `Op::Generic` actually falls back on

Entry 60's queue said the fallback was 17.6% and named `ITERATE`, `LEAVE`, `NOP`, `PARSE VAR` and labels as its inner-loop members. Counted rather than named, under a scratch build that noted each `Op::Generic` entry by instruction kind over the pinned `rexxcps`:

| kind | entries |
| --- | --- |
| `PARSE` | 1,120,002 |
| `THEN` | 700,002 |
| `ITERATE` | 280,000 |
| `TRACE` | 140,200 |
| label | 140,000 |
| `ADDRESS` | 140,000 |
| `LEAVE`, `NOP`, `SIGNAL` | 1 each |
| **total** | **2,520,207** |

**`THEN` was the second-largest member and was not on the queue at all.** `PARSE` is the largest and is untouched. The same sweep over `corpus/lang` plus `bench-programs`, ignoring one program's 1,000,029 `NUMERIC` clauses: labels 149, `PARSE` 88, `SIGNAL` 48, `RAISE` 48, `TRACE` 46, and **`END` six, none of them `EndStyle::Select`**.

#### The per-clause figure, and the floor it has to clear

**A promoted marker clause is 21 user instructions and a promoted label clause is 54**, each measured where it is the only thing that changes and each doubling exactly with the pass count:

* `bench-programs/emptyloop.rex`, whose body is a `NOP`, IR arm: 12,825,630,591 -> 12,300,630,949 at 25,000,000 passes and 25,650,631,343 -> 24,600,631,344 at 50,000,000. 21.000 per pass at both sizes.
* a scratch axis whose loop body is one call to a label whose body is one `RETURN`: 8,722,628,463 -> 8,614,627,830 at 2,000,000 and 17,444,628,561 -> 17,228,628,320 at 4,000,000. 54.000 per pass at both sizes.

The two differ because the marker is a body clause reached with `GRANTING` false and the label is an activation's own first clause reached with it true, which are separate inlinings of `run_ops`.

**The layout floor is up to 0.66% of retired instructions, and this sitting measured it rather than assuming it.** `arith`, `varlookup`, `compound` and `strings` contain none of the promoted kinds, and every one of them still moved -- `varlookup`'s IR arm by +0.66% -- **and so did the tree-walker arm on all four**, which no `Op` change can reach. Three further readings of the same kind:

* on `rexxcps`, promoting the markers alone was +7.46M instructions, the escapes alone +42.9M, and both together +8.33M. Not additive, so not work.
* `6da78f62c` as a whole reads +8,315,409 instructions (+0.058%) on `rexxcps` while removing 980,004 `Generic` entries worth about -20.6M.
* `5e40705d7` reads -7,604,819 against a prediction of 140,000 x 54 = 7,560,000, so there the layout term happened to be near zero.

**So a whole-program instruction count cannot resolve a change of this size on this build, and the fixed-work axis can.** Cycles on `rexxcps` fell at both commits and won every interleaved pair; Moritz reported the same independently.

#### What was declined, with the reason

**`END` stays `Op::Generic`.** It is not a marker -- `EndStyle::Select` raises 7.3 -- and it is stepped essentially never, because every construct answers a `Flow` that resumes past its own `END`: zero over a whole `rexxcps` run, six over the corpus. Promoting it would add an op variant, and this sitting's own numbers price a variant at more than the work it would remove.

**`ELSE` and `OTHERWISE` are markers and are still general.** Each is reached through machinery the marker arm does not touch -- an `ELSE` through the `IF`'s own false target, an `OTHERWISE` through the `Op::EnterOtherwise` at its entry -- so each needs its own witnesses. Corpus counts are 6 and 7.

#### A divergence found while probing -- **and the conclusion drawn from it was wrong. See entry 62.**

The measurement stands and the conclusion does not. Measured 2026-08-17:

```rexx
do zi = 1 to 5
  zlab:
end
say 'done'
```

The oracle prints `done` and exits 0. `rexx-run` answers `rexx-exec: 47.2: Unexpected label.` at rc 120.

This entry originally read that a label inside a `DO` body is legal, that this crate is therefore wrong, and that the defect belongs to `rexx-parse`'s block builder. **All three are false.** This crate is right, the oracle binary on this machine is a 2021 source tree, and the check it lacks was added upstream in 2024. Entry 62 has the provenance and what follows from it.

What survives unchanged is the consequence for measurement: there is no fixed-work axis shaped like `emptyloop` for a label, which is why the label axis above goes through a call.

#### Tooling

`rexx-ir FILE [TRACE-SETTING]` prints the compiled stream of every body, through the new `rexx_exec::render_ir`. The golden serialiser stops being `#[cfg(test)]` and gains two things a reader needs: `Op::Generic` and `Op::Clause` carry the line and source text of the clause they stand for, and abuttal and blank are named rather than rendered by their own spellings, which are the empty string and one space.

#### Queue, revised

* **`PARSE` is now the largest `Op::Generic` member by a factor of 1.6 over everything else left**, 1,120,002 entries on `rexxcps` against 700,002 for the whole marker family that just landed. It has expressions and targets, so it is not a marker-shaped promotion.
* `TRACE` (140,200) and `ADDRESS` (140,000) are next, and both are `rexxcps` artifacts rather than general hot paths.
* The arena and collector, `caller.traps` and the `Heap::slots` chunking carry over from entry 60 unchanged, including the instruction to measure growth before writing the chunked table.

### Entry 62 -- the oracle on this machine is a 2021 tree, and entry 61 misread it

No commits. This entry corrects entry 61 and records an environment fact that changes how every oracle differential taken on this machine must be read.

#### The correction

Entry 61 reported that a label inside a `DO`/`IF`/`SELECT` block is legal, that this crate's 47.2/47.3/47.4 refusals are therefore a defect, and that the defect belongs to `rexx-parse`'s block builder. **This crate is right and the oracle is old.**

`interpreter/parser/LanguageParser.cpp:1240` raises `Error_Unexpected_label_if` / `_select` / `_do` for exactly these shapes, and `interpreter/messages/RexxErrorMessages.h:452` spells 47.2 "Labels are not allowed within a DO/LOOP block; found \"&1\"." So the crate implements the C++ in this repository, which is the specification.

Upstream: `eca92b86c`, 2024-09-05, *"fix code, add messages, update rexxref, fix test groups for [bugs:#1945] No labels should be allowed inside of DO/LOOP, IF, and SELECT groups"*.

`crates/rexx-parse/src/block/tests.rs`'s `a_label_inside_a_block_picks_its_number_from_the_block_kind` was correct as committed and needed no change.

#### The provenance that explains it

| | oracle build on this machine | recorded in `phase-4f-oracle-64a7a7aa4.md` |
|---|---|---|
| `build/bin/rexx` size | 17,144 | 62,600 |
| `build/bin/rexx` mtime | 2025-02-21 | 2026-08-05 |
| `build/bin/rexx` sha256 | `4ed170ca7a1ed053a581e0d20ab62923746a37feb6ebac020ab20b8357b2b4cd` | `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019` |
| `build/lib/librexx.so.4` size | 3,801,176 | 17,853,856 |

**They are not the same binary.** `/home/moritz/dev/repos/ooRexx` is checked out at `f975dde4`, dated **2021-02-13**, and `parse version` answers `REXX-ooRexx_5.0.0(MT)_64-bit 6.05 21 Feb 2025`. That commit is an ancestor of this repository's own tree, so the oracle here is ooRexx as of 2021 and the C++ this crate is written against is three and a half years ahead of it.

Confirming rather than inferring: `/home/moritz/dev/repos/ooRexx/interpreter/parser/LanguageParser.cpp` is 4,371 lines and contains no `Error_Unexpected_label` at all; this repository's is 4,398 lines and raises it.

#### What follows

* **A difference between this crate and the oracle binary is not evidence of a defect in this crate until the oracle's own source is checked.** Entry 61 skipped that step. The rule that catches it is the one already in `CLAUDE.md` about running rather than reasoning, applied one level up: the instrument itself needed checking, not just the claim it made.
* **The 12 corpus programs that disagree under `REXX_CORPUS_GATE=1` on this machine have not been re-examined against this fact.** Some may be the same artifact. Nothing in entries 60 or 61 depends on them -- every result there is base-against-head with the disagreeing set held equal -- but a session that reads "12 disagree" as "12 gaps" would be repeating entry 61's mistake at scale.
* **`phase-4f-oracle-64a7a7aa4.md` records a hash and an mtime for the oracle and does not say which upstream commit it was built from.** A hash identifies a binary; it does not say what the binary implements. The commit is what a later reader needs, and it is why this entry records `f975dde4` rather than only the checksum above.
* Five shapes were measured on this machine, all rejected here and all accepted by the oracle, each read as three separate descriptors: a label in a counted `DO`, in a bare `DO`, in a bounded `LOOP`, in an `IF` branch, and in a `SELECT` both before the first `WHEN` and between two `WHEN`s. `loop forever` with no `LEAVE` was also written and **must not be run** -- it hangs the oracle, which cost ten minutes of this sitting.

### Entry 63 -- the chunked slot table, measured and declined

No commits: the experiment was built, measured and reverted. `Vec<Vec<Slot>>` with power-of-two inner lengths, so a slot never moves once written (Moritz's suggestion, carried in entries 60 and 61 with the instruction to measure growth first).

#### The growth it would remove

Counted under a scratch build that noted every reallocation of `Heap::slots` and every `Heap::resolve`. `Slot` is 96 bytes.

| program | reallocations | bytes moved | peak slots | `resolve` calls |
|---|---:|---:|---:|---:|
| `rexxcps` (pinned) | 14 | 6,291,072 | 32,768 | 13,684,907 |
| `strings` | 14 | 6,291,072 | 32,768 | 30,001,227 |
| `alloc4c` | 14 | 6,291,072 | 32,768 | 10,877,231 |
| `arith` | 14 | 6,291,072 | 32,768 | 7,605,868 |
| `compound` | 0 | 0 | 0 | 10,000,500 |
| `emptyloop` | 0 | 0 | 0 | 0 |
| `varlookup` | 0 | 0 | 0 | 0 |

**The arena's entire growth cost for a whole run is 14 reallocations and 6.29 MB moved.** It reaches 32,768 slots -- 3 MiB -- and stops; the free list serves every allocation after that. `emptyloop` and `varlookup` never touch the arena at all, which is the tagged-handle model working.

#### The tax it would add

Built for real -- a `Slots` type with `locate(i) = (i.bit_width(), i with its highest set bit cleared)`, `push`, `Index`, `IndexMut` -- and `cargo test -p rexx-core` passes under it, so these are numbers from a working implementation rather than from a sketch.

| axis | flat | chunked | delta |
|---|---:|---:|---:|
| `rexxcps` | 14,406,617,615 | 14,776,588,321 | **+2.57%** |
| `strings` | 29,283,644,811 | 30,038,361,177 | **+2.58%** |
| `alloc4c` | 5,537,712,405 | 5,624,108,867 | **+1.56%** |
| `compound` | 16,243,366,482 | 16,403,195,624 | **+0.98%** |

Cycles on `rexxcps`, 7,309,984,762 -> 7,395,997,211, +1.18%, losing every interleaved pair. Every figure is far outside the +/-0.66% layout floor entry 61 established, so these are work rather than layout.

**+369,970,706 instructions on `rexxcps` to remove 6,291,072 bytes of one-time `memcpy`.** Per `resolve` the added cost is 27 instructions on `rexxcps`, 25 on `strings`, 16 on `compound` and 8 on `alloc4c`; the spread is how many of each program's accesses are `get`/`get_mut`, which locate twice -- once inside `resolve` and once in the index that follows it.

**The conclusion does not depend on the implementation being good.** This one costs three dependent loads where the flat table costs one, and a tuned version holding raw chunk pointers in a fixed array could plausibly halve it. Even a perfect one adds at least one dependent load per access, which on `rexxcps` is at least 27,000,000 instructions against a growth cost that is under a million. The margin is about two orders of magnitude.

#### The two secondary prizes, and why neither follows

Entry 60 recorded that the larger prize, if slots stopped moving, is that "handles may not need generation validation per deref and the sweep could go chunk-wise". **Both are wrong, and the first is wrong for a reason already written down.**

* `handle.rs`'s own module doc says the generation exists because **slots are recycled through a free list** -- "without it a handle held across a collection would silently name whatever is allocated into that slot next". That is recycling, not relocation. A chunked table recycles exactly as the flat one does, so the check stays.
* `Heap::collect` already walks `0..slots.len()` over a contiguous array with a flat `marks: Vec<bool>` beside it. That is the best case for a linear sweep; chunking makes it worse, not better.

#### What this does not cover

Every program measured has a live set at or under 32,768 objects, so the largest single copy a reallocation performs here is 3 MiB. A program holding millions of live objects moves proportionally more, and a doubling's last copy is half the final size -- which is a **latency** argument rather than a throughput one, and this sitting measured throughput. Nothing here says anything about a pause-time bound. It says that on every axis this project currently measures, growth is not where the arena's time goes.

### Entry 64 -- following the allocator instead of the arena

`01880fdbb` (one string per rendered number) and `f95e48d97` (compare two parsed numbers without rendering either), against `a28edf4ea`.

Same instrument caveat as entries 60 to 63: `perf stat`, not the pinned wall-clock suite against the oracle.

#### The 5.8% in entry 60's queue does not exist on this build

Entry 60 queued "the arena and the collector, about 8.5% -- `alloc_with` 5.8% total, `Heap::collect` 2.7%". Re-profiled here at `a28edf4ea` on the pinned `rexxcps`:

| symbol | self |
|---|---:|
| `Interp::run_ops::<false>` | 18.24% |
| `Interp::step` | 6.06% |
| `Interp::apply_binary` | 4.67% |
| `Interp::run_loop_with_header` | 4.21% |
| `Interp::alloc_with` | **1.69%** |
| `Heap::collect` | 1.37% |
| `malloc` + `_int_malloc` + `cfree` | **3.18%** |

`alloc_with` inlines `alloc_with_uncollected`, so 1.69% is the whole arena allocation path. **The general-purpose allocator behind the arena is larger than the arena.** Entry 60's figure was another machine and a build fourteen changes older; it was not re-measured before being carried into two later queues, which is the same mistake entry 62 records at a larger scale.

#### Where the allocations are

`LD_PRELOAD` shim over `malloc` recording `__builtin_return_address(0)`, symbolised with `addr2line`. `rexxcps` makes **6,443,166 allocations for 3,770,581 arena objects**, 194.6 MB, and 56% of the calls are seven bytes or fewer.

| allocs | bytes | site |
|---:|---:|---|
| 840,000 | 20,580,000 | `Interp::step` |
| 420,003 x3 | 5,600,045 | `rexx_num::format::render_integer_padded` |
| 280,001 | 1,960,005 | `Interp::clause_site` |
| 280,001 | 13,440,048 | `box_new_uninit`, 48 bytes |
| 280,000 | 4,620,000 | `Interp::step` |
| 280,000 | 1,400,000 | `HashMap<Box<[u8]>, usize>::clone` |
| 140,026 | 49,289,434 | `RawVecInner::finish_grow`, 352 bytes each |
| 140,018 | 25,211,396 | `HashMap::fallible_with_capacity` |

The arena census beside it: 2,800,375 `Body::Text`, 830,205 `Body::Num`, 140,001 other, and 57 collections.

#### The two taken

**`01880fdbb`, three strings per rendered number down to one.** 6,443,166 allocations -> 5,183,158, exactly the 1,260,008 predicted from three per call over 420,003 calls. **-166,583,151 instructions, -1.16%**, which is 132 per allocation removed.

**`f95e48d97`, a comparison of two parsed numbers renders neither.** Of 2,520,002 comparisons reaching the rendering, 1,680,001 had both operands parsed and discarded both renderings; 140,000 are strict and 700,001 genuinely need the fallback. **-380,482,682 instructions, -2.67%**, cycles -3.86%.

**A prediction was written down before the second one and was low.** It said 0.2% to 0.7%, reasoning that `Body::Num` caches its rendered text so a repeat render is a cache read. Measured, a skipped rendering is worth about 226 instructions. The reasoning was not wrong about the cache; it was wrong that the cache was the expensive part.

#### The control that finally worked

Entry 61 had to establish a +/-0.66% layout floor because every axis moved under a change none of them could reach. **Neither change here moved any axis at all**: `arith`, `strings`, `compound` and `varlookup` reach `render_integer_padded` zero times and `compare_values`' rendering zero times, and all four are unchanged to within a few hundred instructions out of seventeen to forty billion.

So the control is inside the same comparison rather than beside it, and that is a property of the *change*, not of better measuring: a change confined to a function the axes never call has its own null result built in. **Prefer a change with that shape when one is available**, because the alternative is entry 61's, where the effect and the relayout are the same size.

#### Running total for the session

`94c4f6464` 14,405,881,606 -> `f95e48d97` 13,859,561,191 on the pinned `rexxcps`, **-546,320,415, -3.79%**, across five commits.

#### Queue

* **`PARSE` is the largest `Op::Generic` member**, 1,120,002 entries, and `Interp::step` is still 6.06% of the profile and 1.12M allocations. The two are the same thing.
* **Per-call activation setup is the largest byte count**, 74 MB of the 194 MB over 140,000 calls: `RawVecInner::finish_grow` at 352 bytes a call, `HashMap::fallible_with_capacity`, and a `HashMap<Box<[u8]>, usize>::clone`.
* **`LeaveOrigin` costs two allocations per `LEAVE`/`ITERATE`**, 280,001 each: the clause text `Interp::clause_site` builds and the 48-byte box around it. The text is read only if the label search fails, so it could be recovered from the instruction index instead of captured.
* The arena itself, at 1.69% plus 1.37%, is no longer worth the queue position entry 60 gave it.

### Entry 65 -- one more for the queue: a loop pass that is a jump rather than a call

No commits. Moritz's suggestion, added here rather than to entry 64's queue because this file is appended to and not rewritten.

**A loop's body is entered by re-entering the driver, once per pass.** `run_repeating`'s own pass loop calls `Interp::run_bounded` for the body range on every iteration, and `run_bounded`'s `BodyEngine::Chunk` arm calls `run_bounded_from_chunk`, which enters `Interp::run_ops` again. So a pass costs a nested Rust frame, a fresh `frames` vector, the range bookkeeping `run_ops` sets up at entry, and an `absorb` on the way out -- where a flattened loop would cost one backward `Op::Jump` inside the driver that is already running.

**This is the one construct the phase flattened only halfway.** `IF` and `SELECT` compile to conditions and jumps in the op stream, and `Op::EnterWhen`/`Op::EnterOtherwise`/`Op::EndBranch` are what replaced their nested `run_bounded` calls. `Op::LoopRun` did not do that: its own doc comment says the construct is resolved by `run_loop_with_header` for both engines and that the one line it changes is which driver steps the body. That was the right call for the task that landed it -- a loop's header re-evaluation, `WHILE`/`UNTIL`, the `LEAVE`/`ITERATE` search and every trace echo are one implementation because of it -- and it is also why the per-pass entry is still there.

**What to measure first**, so this is not queued on structure alone:

* `bench-programs/emptyloop.rex` is the axis that can price it. Its body is one clause, so the per-pass cost is nearly the whole measurement -- it is the same axis that gave entry 61 its 21-instruction figure for a promoted marker clause, by the same argument.
* `Interp::run_loop_with_header` is 4.21% self on the pinned `rexxcps` profile, and the re-entries themselves are inside `run_ops`' own 18.24% rather than beside it, so the flat profile understates the target.
* Count the nested entries before changing anything. `run_chunk_entries` already counts driver entries but is `#[cfg(test)]`; `rexxcps` runs roughly 560,000 loop passes and `emptyloop` runs 50,000,000, so the two axes differ by two orders of magnitude in exactly the quantity at stake.

**The hard part is not the jump, it is what the nested call currently carries.** `run_repeating` re-evaluates the header inside a clause unit of its own each pass, `do_body_outcome` decides `LEAVE`/`ITERATE`/fall-through from the `Flow` the nested call answered, and `HeaderClause` tracks which clause a failing re-test is blamed on -- all of it keyed to the call returning. A flat form has to express those as ops and stream position instead, which is the same shape `Op::EnterWhen` took for `SELECT` and is why that one needed a frame stack in the driver.

### Entry 66 -- the loop pass priced, and a spike that says where the cost may not go

No commits. Entry 65's queue item measured and prototyped; the prototype is not committed and its numbers are why.

Instrument: `perf stat -e instructions:u` on `rexx-run`, two sizes of the same program (2,000,000 and 4,000,000 passes) so the slope is the per-pass cost and startup drops out of it. Every figure below is a slope of that pair, and each reproduced across repeated runs to within a few hundred instructions out of billions.

#### What a loop pass costs

Programs: `do i = 1 to n / end` (empty body) and the same with one, then three, `nop` bodies.

| arm | per pass | per body clause |
|---|---:|---:|
| tree-walker | 721 | 179 |
| compiled stream (`3edfd75f0`) | 816 | 160 |

**The compiled engine is 95 instructions per pass behind the tree-walker and 19 per clause ahead of it, so a loop body has to hold five clauses before the compiled engine wins.** `emptyloop.rex`'s body holds one, and the compiled arm runs it at 976 against the tree-walker's 900.

**The 95 is the nested driver entry, and two instruments agree on it.** The engine A/B above is one. The other is an instruction-level profile of the empty-body program, where `run_ops::<false>` is 11.61% of 816 = 94.7 per pass -- and with an empty body that function has nothing to run, so all of it is entry and exit. The mechanism is in the disassembly: `run_ops`' prologue is six pushes and `sub $0x488,%rsp`, a 1160-byte frame built and torn down once per pass.

The rest of the 816, from the same profile: `run_loop_with_header` 398, `Number::plain_integer` 121, `bind_control` 81, `result_text` 46, `read_at` 44, `do_body_outcome` 16, `novalue_check` 15. **The control variable's own arithmetic is a bigger share of a pass than the driver entry is.**

#### The spike

`Op::LoopRun` sets up a `FlatLoop` in a `run_ops` local, the region ends normally so the counter falls into the body's first op, and the driver's own loop notices the pass end and runs the header re-test. `do_body_outcome` and `loop_advance` are called unchanged, so `LEAVE`/`ITERATE` and the header keep one implementation. Declines to `run_loop_with_header` for `WHILE`/`UNTIL`, `COUNTER`, `OVER`, `DO WITH`, `trace all`, and a loop nested inside one already flat.

**Both arms were built as one binary with the flat path behind a run-time switch**, so the driver's added per-op checks are compiled into both and an arm with the path never taken prices those checks alone.

| arm | per pass | per body clause |
|---|---:|---:|
| `3edfd75f0` | 816 | 160 |
| spike, flat path off | 882 | 172 |
| spike, flat path on | 765 | 182 |

**The flattening itself is worth 117 instructions per pass** (882 -> 765), which is more than the 95 the entry costs, because it also removes `run_bounded`'s absorb and the caller-side spill around the call.

**And the prototype's way of reaching it costs 66 per pass before it is ever taken, plus 12 to 22 per body clause.** The idle 66 is read straight out of the binary: `run_ops`' frame goes from 1160 bytes to 1688, so every nested entry -- which is still what an ineligible loop uses -- pays for a frame holding state it does not use. The per-clause tax is the checks themselves, on the hottest path this interpreter has.

**On a real program the trade is negative.** The pinned `rexxcps`, output identical apart from its own clauses-per-second line: `13,859,520,570` -> `14,168,972,045`, **+309,451,475, +2.23%**. Its loop bodies average tens of clauses, so the per-clause tax is charged tens of times for each per-pass saving. Predicted from the slopes above at about +198M against +309M measured -- same sign and order, and the residue is the layout floor plus the nested loops that decline.

#### What the spike establishes for whoever builds this

**The constraint is not "flatten the loop", it is "add nothing to the driver's per-op path or its frame".** A prototype that pays for the pass out of the clause loses, because a program has far more clauses than passes. Three placements were measured and all three raised the per-clause cost: an `Option<FlatLoop>` local (177 per clause), the same boxed with the flow test reordered (181), and one that folded the pass-end arrival into the existing `pc >= stop` check by making `stop` mutable (182, and mutating `stop` costs the per-op comparison its register).

So the shape to build is the one `Op::EnterWhen` already uses, and Moritz's question names its missing half: **a backward `Op::Jump` at the `END`**, so a pass end is an op the driver decodes when it arrives rather than a condition it tests on every op, with the loop's state in the `frames` stack the loop already checks per op rather than in a new local. `Op::Jump`'s own doc says a backward jump out of a range is not checked and not emitted; one *inside* a range is what this needs, and the driver already executes `pc = *target` for it.

#### A failure mode worth naming

The first spike had no guard against a second loop starting while one was flat, so an inner loop overwrote the outer loop's state. `rexxcps` then reported **`3E+11 REXX clauses per second`** and exited 0, having run 2,353,529 instructions against 13.86 billion. **A loop-flattening defect does not crash; it silently does no work**, and the instruction count is what noticed. Any work here wants a fixed-work axis compared before its output is believed.
