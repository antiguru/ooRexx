# Review package: 4d-1 Task 4

Range 08cb102a..22b4609c

22b4609c Fold alloc4c into the current Phase 4d-1 baseline, and flag it for Task 6
d233d1e9 Add alloc4c.rex, the allocation axis restricted to the 4c surface

 docs/superpowers/plans/perf-baseline.md            | 219 ++++++++++++++-------
 rust/bench-programs/README.md                      |   8 +-
 rust/bench-programs/alloc4c.rex                    |  39 ++++
 rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs |   4 +
 rust/crates/rexx-bench/src/lib.rs                  |   1 +
 5 files changed, 201 insertions(+), 70 deletions(-)

## Diff
diff --git a/docs/superpowers/plans/perf-baseline.md b/docs/superpowers/plans/perf-baseline.md
index 758ea0a3..097844b9 100644
--- a/docs/superpowers/plans/perf-baseline.md
+++ b/docs/superpowers/plans/perf-baseline.md
@@ -535,192 +535,277 @@ That contradicts this phase's own no-optimisation rule (Global Constraints); the
 recorded rather than argued away, in `task-2b-brief.md` and again here: this unit can produce a
 *current* baseline and attribution, and cannot produce a pre-optimisation one.
 
 **Same harness, same method.**
 This is another run of the same `rexx-bench-suite`, described in "Method, and the three choices
 that are not the obvious ones" above; that section is not repeated here.
 As before, every heading from "Provenance" to "Axes this crate cannot run" below, and every table
 and paragraph under them, is that program's output byte for byte; the prose before and after that
 range is written here.
 
+**This run also adds `alloc4c`, measured for the first time.**
+Task 4 of this phase gave `alloc.rex` a 4c-surface analogue after finding the object model entirely
+absent on this crate: `.array~of`, `.string~new`, `~size` and `~length` each fail on their own and
+combined, all four with the same `rexx-exec: a message send is not implemented (Phase 5)`, so no
+single construct is the blocker -- the whole surface is. `alloc4c.rex`'s own header states plainly
+what it does and does not preserve from `alloc.rex`; in short, it substitutes a new
+compound-variable tail and a `||` concatenation for the array and string, which measures allocation
+throughput rather than the collection pressure `alloc.rex` was sized to force (see "`alloc4c`: the
+closest axis, and what that means" below). `rexx-run`'s sha256 is unchanged from the run this one
+replaces -- no interpreter code moved, only the benchmark corpus and the harness's axis list did.
+
 ### Provenance
 
 | | |
 |---|---|
-| measured | 2026-08-08T21:02:25+02:00 |
-| repo commit | `16ec27be119bb5fd44760804fb67477c0732a222` |
+| measured | 2026-08-08T22:49:05+02:00 |
+| repo commit | `d233d1e90313b1d5f7f283ea09de749958861f61` |
 | oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
 | oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
 | oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
-| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=12996808 bytes, mtime=2026-08-08 15:02:55.950099476 +0200, sha256=c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967 |
+| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=12996808 bytes, mtime=2026-08-08 21:50:02.437486282 +0200, sha256=c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967 |
 | address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
 | pairs per axis | 9 sampled, 1 warm-up pair(s) discarded, oracle and this crate alternating |
 | pairs for the offset line | 51 sampled, 5 warm-up |
 | statistic | median; interval is the distribution-free sign-test interval for the median at a 95% target |
 | working directory of every child | a fresh empty temporary directory |
 
 ### Fixed per-process offset (`startup.rex`)
 
 **Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two numbers below are each side's own fixed cost, reported so every axis above can be read net of it -- not as a result about which interpreter starts faster.
 
 | side | median | min | max | 95.1% interval | spread |
 |---|---:|---:|---:|---|---:|
-| oracle | 7.353 ms | 5.161 ms | 9.126 ms | 6.997 - 7.710 ms | 53.9 % |
-| this crate | 1.727 ms | 1.245 ms | 2.322 ms | 1.555 - 1.908 ms | 62.4 % |
+| oracle | 7.645 ms | 5.307 ms | 8.620 ms | 7.181 - 7.905 ms | 43.3 % |
+| this crate | 1.797 ms | 1.199 ms | 2.602 ms | 1.695 - 1.929 ms | 78.0 % |
 
 Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.
 
 ### Axes
 
 `iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.
 
 | axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
 |---|---:|---|---:|---:|---:|---|---:|---:|---:|
-| `arith` | 500000 | oracle | 1.1578 s | 1.1476 s | 1.1712 s | 1.1529 - 1.1701 s | 2.04 % | 431867 | 434628 |
-| `arith` | 500000 | this crate | 3.1273 s | 3.1064 s | 3.1385 s | 3.1214 - 3.1372 s | 1.03 % | 159882 | 159971 |
-| `compound` | 5000000 | oracle | 1.1459 s | 1.1300 s | 1.1531 s | 1.1405 - 1.1487 s | 2.02 % | 4363428 | 4391609 |
-| `compound` | 5000000 | this crate | 7.2720 s | 7.2621 s | 7.4648 s | 7.2633 - 7.2859 s | 2.79 % | 687571 | 687735 |
-| `strings` | 3000000 | oracle | 0.8654 s | 0.8499 s | 0.8853 s | 0.8551 - 0.8782 s | 4.08 % | 3466493 | 3496199 |
-| `strings` | 3000000 | this crate | 9.3247 s | 9.2433 s | 9.5101 s | 9.2453 - 9.4594 s | 2.86 % | 321725 | 321785 |
-| `varlookup` | 19000000 | oracle | 1.2064 s | 1.1893 s | 1.2547 s | 1.2009 - 1.2201 s | 5.42 % | 15748922 | 15845500 |
-| `varlookup` | 19000000 | this crate | 5.2328 s | 5.2208 s | 5.2702 s | 5.2224 - 5.2570 s | 0.94 % | 3630966 | 3632165 |
+| `alloc4c` | 1000000 | oracle | 1.1279 s | 1.1144 s | 1.1477 s | 1.1205 - 1.1408 s | 2.95 % | 886579 | 892630 |
+| `alloc4c` | 1000000 | this crate | 2.3422 s | 2.3099 s | 2.3615 s | 2.3134 - 2.3591 s | 2.21 % | 426957 | 427285 |
+| `arith` | 500000 | oracle | 1.1556 s | 1.1470 s | 1.1572 s | 1.1509 - 1.1568 s | 0.88 % | 432666 | 435548 |
+| `arith` | 500000 | this crate | 3.1207 s | 3.1001 s | 3.1378 s | 3.1039 - 3.1296 s | 1.21 % | 160219 | 160312 |
+| `compound` | 5000000 | oracle | 1.1500 s | 1.1279 s | 1.1686 s | 1.1414 - 1.1593 s | 3.55 % | 4348006 | 4377107 |
+| `compound` | 5000000 | this crate | 6.9965 s | 6.9733 s | 7.0259 s | 6.9847 - 7.0071 s | 0.75 % | 714640 | 714823 |
+| `strings` | 3000000 | oracle | 0.8633 s | 0.8485 s | 0.8734 s | 0.8505 - 0.8718 s | 2.89 % | 3475037 | 3506087 |
+| `strings` | 3000000 | this crate | 9.1611 s | 8.9866 s | 9.4877 s | 9.0661 - 9.2260 s | 5.47 % | 327470 | 327534 |
+| `varlookup` | 19000000 | oracle | 1.2133 s | 1.1977 s | 1.2408 s | 1.1999 - 1.2191 s | 3.55 % | 15659908 | 15759214 |
+| `varlookup` | 19000000 | this crate | 5.2800 s | 5.2632 s | 5.3408 s | 5.2670 - 5.2929 s | 1.47 % | 3598469 | 3599695 |
 
 #### Ratios and the gate call
 
 The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.
 
 **The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.
 
 | axis | oracle median | this crate median | ratio | ratio interval | verdict |
 |---|---:|---:|---:|---|---|
-| `arith` | 1.1578 s | 3.1273 s | 2.70x | 2.67x - 2.72x | SLOWER |
-| `compound` | 1.1459 s | 7.2720 s | 6.35x | 6.32x - 6.39x | SLOWER |
-| `strings` | 0.8654 s | 9.3247 s | 10.77x | 10.53x - 11.06x | SLOWER |
-| `varlookup` | 1.2064 s | 5.2328 s | 4.34x | 4.28x - 4.38x | SLOWER |
+| `alloc4c` | 1.1279 s | 2.3422 s | 2.08x | 2.03x - 2.11x | SLOWER |
+| `arith` | 1.1556 s | 3.1207 s | 2.70x | 2.68x - 2.72x | SLOWER |
+| `compound` | 1.1500 s | 6.9965 s | 6.08x | 6.02x - 6.14x | SLOWER |
+| `strings` | 0.8633 s | 9.1611 s | 10.61x | 10.40x - 10.85x | SLOWER |
+| `varlookup` | 1.2133 s | 5.2800 s | 4.35x | 4.32x - 4.41x | SLOWER |
 
 #### Same work on both sides
 
 A wall time is only about the workload if the workload ran. Every sampled run on each side printed the same bytes, and the two sides printed the same bytes as each other.
 
 | axis | stable within a side | identical across sides | stdout |
 |---|---|---|---|
+| `alloc4c` | yes | yes | `12888896` |
 | `arith` | yes | yes | `4629643519330627.7808` |
 | `compound` | yes | yes | `5000000` |
 | `strings` | yes | yes | `138000000` |
 | `varlookup` | yes | yes | `19000000` |
 
 ### `samples/rexxcps.rex`
 
 The oracle's own clauses-per-second benchmark, run from the read-only C++ tree. It self-calibrates: a trial that comes in at or under a second is run again at twice the count, so the two sides do **different amounts of work** and their wall times are not directly comparable. The clauses-per-second figure each side prints is per clause and is the comparable one. Each side's `Averaged:` line is quoted so the asymmetry is visible rather than inferred.
 
 | side | wall median | wall interval | `Averaged:` |
 |---|---:|---|---|
-| oracle | 1.7955 s | 1.7641 - 1.8290 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
-| this crate | 4.6776 s | 4.6605 - 4.7435 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.3s)` |
+| oracle | 1.7832 s | 1.7657 - 1.8173 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
+| this crate | 4.6818 s | 4.6567 - 4.7063 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.3s)` |
 
 | side | median cps | min | max | 96.1% interval | spread |
 |---|---:|---:|---:|---|---:|
-| oracle | 16873935 | 16083829 | 17204168 | 16550975 - 17194214 | 6.64 % |
-| this crate | 2308020 | 2282779 | 2324259 | 2293006 - 2317831 | 1.80 % |
+| oracle | 16936694 | 16561597 | 17197186 | 16604359 - 17149364 | 3.75 % |
+| this crate | 2311938 | 2293261 | 2322914 | 2299072 - 2321553 | 1.28 % |
 
-**Internal cps ratio: 7.31x** (oracle median over this crate's median), interval 7.14x - 7.50x.
+**Internal cps ratio: 7.33x** (oracle median over this crate's median), interval 7.15x - 7.46x.
 
 ### Axes this crate cannot run
 
 Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.
 
 | axis | exit status | message |
 |---|---:|---|
 | `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
 | `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
 | `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
 
-### Spread, and why this run is accepted despite exceeding the old band on two rows
+### `alloc4c`: the closest axis, and what that means
 
-**The bar is the committed baseline's band, taken over the four axis rows and not `rexxcps`.**
+`alloc4c` measured 2.08x, interval 2.03x-2.11x -- the tightest of the five axes, ahead of `arith` at
+2.70x, the next closest.
+That invites the question its own header's preserve/not-preserve split exists to answer: is 2.08x
+this axis's true position on the allocation dimension, or is `alloc4c` simply easier than
+`alloc.rex` would have been?
+
+**The honest answer leans toward the latter, for two structural reasons the header already names.**
+`alloc4c` substitutes a new compound-variable tail and a `||` concatenation for `alloc.rex`'s
+array-of-3 and `.string~new` -- two heap-allocating operations either way, but not the same two.
+First, a compound-variable creation is a single directory insert; it does not, unlike `.array~of`,
+need a class-method dispatch, an array's own allocation, and three element stores, so whatever
+per-operation overhead separates this crate's (entirely unimplemented) message-send path from the
+oracle's is exactly what `alloc4c` cannot exercise.
+Second, `alloc.rex`'s array and string are ephemeral -- rebound every pass, collectible under any
+interpreter that actually collects -- while `alloc4c`'s compound-variable tails accumulate as a
+genuinely live, growing table on both sides; a collector-driven interpreter pays a sweep cost on
+`alloc.rex` that it has no reason to pay on `alloc4c`, and the oracle is plausibly such an
+interpreter given `alloc.rex`'s own header says it is "sized to force multiple collections".
+Neither the message-send overhead nor the oracle's collection behaviour was measured directly here,
+so both are read as plausible directions rather than confirmed causes -- but both point the same way,
+toward `alloc4c` narrowing the ratio relative to what `alloc.rex` would show, for reasons that have
+nothing to do with this crate's own allocator being fast.
+
+**Set against that, two heap allocations a pass is a real and nontrivial amount of work, and 2.08x is
+not close to 1.00x.**
+If two heap-allocating operations per iteration really are the dominant cost on this axis, 2.08x is
+simply the finding: this crate's per-allocation overhead, relative to the oracle's, is smaller here
+than its per-operation overhead is on decimal arithmetic, compound-variable *access*, or string
+search-and-replace.
+Nothing measured in this task -- no collection-pause timing, no allocator call count, no per-side
+breakdown of where either interpreter's time on `alloc4c` actually goes -- distinguishes between that
+explanation and the structural one above; both are consistent with 2.08x, and this task took no
+measurement that would separate them.
+
+**Flag for Task 6's attribution work.** Allocation throughput came out the closest of the five axes
+to the oracle, not the furthest.
+If Task 6 is building a case where the other four axes' slowness is substantially explained by
+allocation cost, this result argues against that case, or at minimum is a complication it needs to
+address rather than pass over: the axis built to isolate allocation is the one axis where this crate
+is *least* slow relative to the oracle.
+Whether that is because allocation genuinely is not this crate's bottleneck, or because `alloc4c`
+undermeasures `alloc.rex`'s own allocation load for the two structural reasons above, is Task 6's
+question to answer; this task's contribution is making the number and the caveat both available to
+it, not resolving which explanation is right.
+
+### Spread, and why this run is accepted despite exceeding the old band on one row
+
+**The bar is the committed baseline's band, taken over the four axis rows `alloc.rex`'s predecessor
+section measured, and not `rexxcps`.**
 The `107febcd` section's this-crate spreads over `arith`, `compound`, `strings`, `varlookup` ran
 1.02% to 2.40%; its oracle spreads over those same four ran 2.29% to 7.11%.
+`alloc4c` has no entry in that band -- it did not exist at `107febcd` -- so its own usability is
+checked separately, below, against the run it is added to rather than against `107febcd`.
 
-**On the oracle side, every row in this run lands inside that band, two of them well inside it.**
-This run's oracle spreads are `arith` 2.04%, `compound` 2.02%, `strings` 4.08%, `varlookup` 5.42%.
-`arith` and `compound` sit below the old band's lower edge (2.29%) -- tighter, not wider -- and
-`strings` and `varlookup` sit below its upper edge (7.11%), `varlookup` by nearly two points.
+**On the oracle side, every row in this run lands inside that band or below it.**
+This run's oracle spreads are `arith` 0.88%, `compound` 3.55%, `strings` 2.89%, `varlookup` 3.55%.
+`arith` sits below the old band's lower edge (2.29%) -- tighter, not wider -- and the other three sit
+inside it, `compound` and `varlookup` closest to its upper edge (7.11%) at roughly half of it.
 No oracle row exceeds the old band.
 
-**On the this-crate side, two of four rows exceed it.**
-This run's this-crate spreads are `arith` 1.03%, `compound` 2.79%, `strings` 2.86%, `varlookup`
-0.94%.
-`arith` and `varlookup` sit inside or below the old band, but `compound` (2.79%) and `strings`
-(2.86%) both sit above its 2.40% upper edge, by 0.39 and 0.46 points respectively.
-That is not stated away: this run is noisier than `107febcd`'s on those two rows, and only those
-two, out of the eight total.
+**On the this-crate side, one of four rows exceeds it, by more than either of the previous run's two
+over-band rows did.**
+This run's this-crate spreads are `arith` 1.21%, `compound` 0.75%, `strings` 5.47%, `varlookup`
+1.47%.
+`arith` and `varlookup` sit inside the old band and `compound` sits below its lower edge (1.02%),
+tighter again; `strings` (5.47%) sits above its 2.40% upper edge by 3.07 points -- a wider excess
+than either of the previous run's two over-band rows (0.39 and 0.46 points), though concentrated in
+one row here rather than spread across two.
+That is not stated away: this run is noisier than `107febcd`'s on `strings`, and only on `strings`,
+out of the eight historical cells.
+
+**`alloc4c`'s own spreads sit inside the band the run it replaces held.**
+2.95% oracle and 2.21% this crate are both inside 0.94%-5.42%, the full range that run's own four
+axes spanned -- the same check applied to a new axis rather than to a historical one, because no
+prior run measured `alloc4c` to compare against directly.
 
 **It is accepted anyway, on the ratio intervals rather than the raw spreads.**
 The gate criterion this task exists to support is per-axis attribution, and that is what the ratio
-interval carries: 2.67x-2.72x, 6.32x-6.39x, 10.53x-11.06x, 4.28x-4.38x.
-None of the four comes near overlapping another, so the four axes remain distinguishable from each
-other despite the two wider rows -- the same conclusion "The spread of ratios across axes is real,
-not measurement noise" reached below for `107febcd`, reached again here on slightly noisier input.
-What the two wider rows cost is precision on `compound`'s and `strings`' individual ratios, not the
-ability to rank the four axes against each other, and ranking them is what Tasks 3 and 4 need.
+interval carries: 2.03x-2.11x, 2.68x-2.72x, 6.02x-6.14x, 10.40x-10.85x, 4.32x-4.41x.
+None of the five comes near overlapping another, so all five axes remain distinguishable from each
+other despite the one wider row -- the same conclusion "The spread of ratios across axes is real,
+not measurement noise" reached below for `107febcd`, reached again here on `strings`' noisier input.
+What the wider row costs is precision on `strings`' own ratio, not the ability to rank the five axes
+against each other, and ranking them is what Tasks 3 and 4 need.
 
 **This is a different bar from the one the indicative run failed.**
 The controller's own contended run, taken with other work on the machine and recorded in
 `task-2b-brief.md` as direction-only, hit 82.77% spread on `strings` and 32.84% on `arith` -- one to
 two orders of magnitude past the committed band, wide enough that its ratio intervals would have
 overlapped each other and said nothing.
-This run's worst row, at 5.42% (oracle `varlookup`, itself inside the old band), is roughly a sixth
-of the indicative run's *best* row (32.84%).
-Exceeding the old band by a few tenths of a point on two rows, when the attribution the band exists
-to protect still holds, is not the same failure as a spread one to two orders of magnitude past it,
-and rejecting this run on that basis would be discarding a usable measurement over noise smaller
-than what the band is meant to catch.
+This run's worst row, at 5.47% (this crate `strings`), is roughly a sixth of the indicative run's
+*best* row (32.84%).
+Exceeding the old band on one row, when the attribution the band exists to protect still holds, is
+not the same failure as a spread one to two orders of magnitude past it, and rejecting this run on
+that basis would be discarding a usable measurement over noise smaller than what the band is meant to
+catch.
 
-**As a coarse cross-check**, three of this run's four ratios land close to the contended run's:
-`arith` 2.70x against 2.60x, `compound` 6.35x against 6.32x, `strings` 10.77x against 10.68x.
-`varlookup` is the exception: 4.34x against 4.28x is also close in absolute terms, but the contended
+**As a coarse cross-check**, `arith`, `compound` and `strings` land close to the contended run's own
+ratios: `arith` 2.70x against 2.60x, `compound` 6.08x against 6.32x, `strings` 10.61x against 10.68x.
+`varlookup` is the exception: 4.35x against 4.28x is also close in absolute terms, but the contended
 run's own spread on that axis was not reported cleanly enough to know what the closeness means.
 This is one sentence of agreement between a clean run and a noisy one, not a second baseline; the
 noisy run's own spreads make it unusable as a check on anything finer than "same order of magnitude."
+`alloc4c` has no figure in the contended run to check against, because it did not exist yet.
 
 ### The internal cps ratio is unstable across runs, more than the wall-clock ratios are
 
-Three measurements of the same tree gave three different internal cps ratios: 7.41x
-(`--self-check`, one pair, explicitly not a baseline), 6.21x (the controller's contended run), and
-7.31x (this run, nine pairs, the accepted one).
+Four measurements of the same tree gave four different internal cps ratios: 7.41x (`--self-check`,
+one pair, explicitly not a baseline), 6.21x (the controller's contended run), 7.31x (the nine-pair
+run this section replaces), and 7.33x (this run, nine pairs, the currently accepted one).
 The 7.41x figure's provenance: the coordinator ran `--self-check` on this tree before dispatching
-this task, and passed the resulting number in a message rather than a committed file, so no output
-file backs it; it is recorded here as a single-pair, explicitly-not-a-baseline figure on that basis,
-not as a measurement this task reproduced.
-That spread -- 6.21x to 7.41x, about 1.2x peak to peak -- is wider than the spread across the four
-wall-clock ratios' repeat measurements in this same task, none of which moved by more than a few
-hundredths of a unit between runs.
-The 7.14x-7.50x interval reported above is this run's own sign-test interval and does not capture
+the task that produced the run this section replaces, and passed the resulting number in a message
+rather than a committed file, so no output file backs it; it is recorded here as a single-pair,
+explicitly-not-a-baseline figure on that basis, not as a measurement any task reproduced.
+That four-measurement spread -- 6.21x to 7.41x, about 1.2x peak to peak -- is wider than the spread
+across the four wall-clock ratios' repeat measurements across these same two runs, none of which
+moved by more than a few hundredths of a unit between them.
+**The two nine-pair runs agree far more tightly with each other than either does with the single-pair
+or contended figures**: 7.31x and 7.33x are 0.3% apart, against a 1.2x peak-to-peak spread across all
+four.
+Two independent nine-pair runs on nearly the same commit agreeing this closely is itself evidence the
+machine was quiet for both -- the same conclusion the four reproduced wall-clock ratios above support
+independently.
+The 7.15x-7.46x interval reported above is this run's own sign-test interval and does not capture
 that between-run movement; it describes how much the median would move if this exact run were
 repeated, not how much it moved when the run itself, the contention on the machine, or both changed.
-Read 7.31x as the accepted figure for this measurement and read 6.21x-7.41x as the honest range
-across what has actually been observed, not as a tighter interval around 7.31x.
+Read 7.33x as the accepted figure for this measurement and read 6.21x-7.41x as the honest range
+across what has actually been observed, not as a tighter interval around 7.33x.
 
 Recorded as an observation about measurement stability, not attributed: nothing in this task looked
-for why the internal cps ratio moves more than the loop-axis ratios do between runs.
+for why the internal cps ratio moves more than the loop-axis ratios do between runs, though the two
+nine-pair figures' tight agreement suggests the movement seen so far is concentrated in the
+single-pair and contended measurements rather than being a property of nine-pair runs generally.
 
 ### What moved between the two baselines, and why
 
 **Every ratio fell, and none fell by the same factor.**
 
 | axis | `107febcd` ratio | this run's ratio | change |
 |---|---:|---:|---:|
 | `arith` | 3.52x | 2.70x | -23% |
-| `compound` | 12.15x | 6.35x | -48% |
-| `strings` | 13.70x | 10.77x | -21% |
-| `varlookup` | 23.07x | 4.34x | -81% |
-| `rexxcps` (internal) | 9.26x | 7.31x | -21% |
+| `compound` | 12.15x | 6.08x | -50% |
+| `strings` | 13.70x | 10.61x | -23% |
+| `varlookup` | 23.07x | 4.35x | -81% |
+| `rexxcps` (internal) | 9.26x | 7.33x | -21% |
+
+`alloc4c` is not in this table: it did not exist at `107febcd`, so it has no prior figure to compare
+against, and it is not one of the four axes the five commits below were attributed against.
 
 The five commits named at the top of this section are what moved these: `3799692d` (inline a
 literal that already spells a small integer), `b6b1d8a9` (small-integer arithmetic without building
 a decimal), `e1d50dda` (step a counted loop without building a decimal per iteration), `c428ec8a`
 (compute a clause's indent once per position instead of once per step), `04ab4af6` (fill the indent
 table in one walk instead of one per position).
 `varlookup`'s far larger drop is consistent with `e1d50dda` landing directly in that axis's
 loop-stepping path, while `arith`, `strings` and the internal `rexxcps` ratio -- which do not depend
 on counted-loop stepping the same way -- moved by a narrower and more similar 21-23%.
 This task did not re-attribute which commit caused which axis's share of its drop; that division of
diff --git a/rust/bench-programs/README.md b/rust/bench-programs/README.md
index bf09ae2c..257ff254 100644
--- a/rust/bench-programs/README.md
+++ b/rust/bench-programs/README.md
@@ -4,28 +4,30 @@ One `.rex` file per D9 performance dimension (`docs/superpowers/plans/2026-07-27
 Global Constraints and D9), each sized to run roughly 0.5-2s under `build/bin/rexx` except
 `startup.rex`, which is deliberately as close to instantaneous as a program can be.
 
 | File | Covers |
 |---|---|
 | `dispatch.rex` | Tight method-send loop: one object, one instance method, 5,000,000 sends |
 | `varlookup.rex` | Plain simple-variable read/write, no stems, 19,000,000 iterations |
 | `compound.rex` | Stem/compound-variable access — the workload the compound-variable memo prototype measured at -24% (`[[compound-variable-memo-prototype]]` in project memory); 500 tails is inside its measured 100-10,000 sweet spot |
 | `strings.rex` | `SUBSTR`/`POS`/`CHANGESTR`/concatenation, 3,000,000 iterations |
 | `arith.rex` | Decimal arithmetic, alternating `NUMERIC DIGITS 9` and `NUMERIC DIGITS 20` every iteration so both settings are exercised throughout the run rather than only at startup |
-| `alloc.rex` | Allocation churn: a fresh `.array` and `.string` every iteration, neither retained past it, sized to force multiple collections |
+| `alloc.rex` | Allocation churn: a fresh `.array` and `.string` every iteration, neither retained past it, sized to force multiple collections. Blocked on this crate -- message sends are Phase 5's |
+| `alloc4c.rex` | Allocation churn restricted to the 4c surface (no message sends): a new compound-variable tail and a concatenated string every iteration. Not the same axis as `alloc.rex` -- see its own header for what carries over and what does not |
 | `startup.rex` | `say 1` — cold-start timing (D2's gate), timed separately with `rexx-time`, not through criterion's statistical sampling |
 
 ## Determinism
 
 Same rule as `corpus/`: byte-identical output on every run of the same interpreter. No `DATE()`,
-`TIME()`, process IDs, or unordered iteration. All seven were run under `build/bin/rexx` and
-confirmed to exit 0 with identical output across repeated runs before being committed.
+`TIME()`, process IDs, or unordered iteration. Every program in this directory was run under
+`build/bin/rexx` and confirmed to exit 0 with identical output across repeated runs before being
+committed.
 
 ## Sizing
 
 Loop counts were tuned by timing each program directly under `build/bin/rexx` (not through the
 criterion harness) and adjusting until each landed between 0.5s and 2s. See
 `docs/superpowers/plans/perf-baseline.md` for the measured numbers this produced.
 
 ## Two things this corpus (see `../corpus/README.md`) learned the hard way, reconfirmed here
 
 `say a"|"b` does not concatenate three values — `"|"b` is read as a **binary string literal** (the
diff --git a/rust/bench-programs/alloc4c.rex b/rust/bench-programs/alloc4c.rex
new file mode 100644
index 00000000..e919b4e2
--- /dev/null
+++ b/rust/bench-programs/alloc4c.rex
@@ -0,0 +1,39 @@
+/* Allocation-churn dimension, restricted to the 4c surface (no message sends).
+
+   `alloc.rex` allocates a fresh `.array` and `.string` every iteration via
+   `.array~of`, `.string~new`, `~size` and `~length` -- all message sends, and
+   this crate implements none yet (`rexx-exec: a message send is not
+   implemented (Phase 5)`; confirmed 2026-08-08 on `.array~of(1,2,3)` alone,
+   on `.string~new("item")` alone, and on the combination, all three the same
+   error). Allocation throughput does not need the object model, so this
+   program covers the same dimension with constructs this crate has: `||`
+   concatenation and compound-variable creation.
+
+   What this preserves from alloc.rex: two heap-allocating operations per
+   iteration (there, one array plus one string; here, one new compound
+   variable plus one concatenated string), and an accumulator whose two
+   terms (a constant 3, standing in for the array's fixed size, and a
+   string's length) stay in tagged-small-integer range for the whole run, so
+   neither the accumulation nor the loop-bound arithmetic itself allocates --
+   exactly as alloc.rex's `a~size`/`s~length` additions do not.
+
+   What this does NOT preserve: alloc.rex's array and string are both
+   rebound to the same loop-local variable every pass, so on a collector that
+   actually swept, they would be garbage before the next iteration starts
+   ("none of them retained past the iteration that made them", per that
+   file's own header). `tab.i` here is a NEW tail every pass -- tab.1,
+   tab.2, ..., tab.n -- so it is a genuinely live, growing table on any
+   interpreter, oracle included; it is not collectible churn. This axis
+   therefore measures raw allocation throughput, not collection pressure, and
+   does not exercise whatever forces a collection in alloc.rex's own design
+   intent ("sized to force multiple collections"). It is not a substitute for
+   a collection-forcing measurement once message sends land; it is the piece
+   of that axis this crate can run today. */
+n = 1000000
+total = 0
+do i = 1 to n
+    tab.i = i
+    s = "item" || i
+    total = total + 3 + length(s)
+end
+say total
diff --git a/rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs b/rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs
index 0bef752e..bb13c2eb 100644
--- a/rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs
+++ b/rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs
@@ -158,20 +158,24 @@ struct Axis {
 /// and the report stays green over whatever is left -- the same defect
 /// `rexx-exec/tests/corpus.rs` guards its phase subset files against, twice
 /// having actually happened there.
 ///
 /// Sorted, because the directory listing this is compared against is sorted.
 const AXES: &[Axis] = &[
     Axis {
         name: "alloc",
         role: Role::Blocked,
     },
+    Axis {
+        name: "alloc4c",
+        role: Role::Loop,
+    },
     Axis {
         name: "arith",
         role: Role::Loop,
     },
     Axis {
         name: "compound",
         role: Role::Loop,
     },
     Axis {
         name: "dispatch",
diff --git a/rust/crates/rexx-bench/src/lib.rs b/rust/crates/rexx-bench/src/lib.rs
index c47dd549..eab214c1 100644
--- a/rust/crates/rexx-bench/src/lib.rs
+++ b/rust/crates/rexx-bench/src/lib.rs
@@ -34,20 +34,21 @@ pub const BINARY_VAR: &str = "REXX_BENCH_BINARY";
 /// so a program added later is either benchmarked or exempted on purpose
 /// rather than silently uncovered.
 pub static PROGRAMS: &[&str] = &[
     "startup",
     "dispatch",
     "varlookup",
     "compound",
     "strings",
     "arith",
     "alloc",
+    "alloc4c",
 ];
 
 /// Programs in `bench-programs/` that the criterion harness deliberately does
 /// not benchmark.
 ///
 /// **Membership here is a claim about the program, and the claim is checked.**
 /// A program belongs here when it measures and reports its own timing: it
 /// prints a figure the harness cannot produce, and timing the whole process
 /// instead would measure the sum of the parts it separates.
 /// `the_exemptions_are_true_of_the_programs_they_name` asserts that in both
