# Review package: 4d-1 Task 2b

Range 16ec27be..4092fb88

## Commits
4092fb88 Re-measure the Phase 4d-1 baseline after five post-baseline speedups

## Stat
 docs/superpowers/plans/perf-baseline.md | 203 +++++++++++++++++++++++++++++++-
 1 file changed, 202 insertions(+), 1 deletion(-)

## Diff
diff --git a/docs/superpowers/plans/perf-baseline.md b/docs/superpowers/plans/perf-baseline.md
index b6fe7db6..fe095252 100644
--- a/docs/superpowers/plans/perf-baseline.md
+++ b/docs/superpowers/plans/perf-baseline.md
@@ -263,21 +263,30 @@ Under this project's standard `ulimit -v 1048576` this crate reserves 512 MiB of
 (D19's `INTERPRETER_STACK_BYTES`) before running anything and the oracle reserves nothing
 comparable, so under the shared cap this crate has roughly 500 MB of room and the oracle roughly
 1000, and the two do not have equal headroom.
 It does not touch these figures -- the benchmark's inner loop comes nowhere near either ceiling and
 all ten runs completed without an allocation failure or a signal -- but it would matter if this
 benchmark were run under a tighter budget.
 
 No optimisation was attempted and none is proposed here.
 Recording the number is the whole of it.
 
-## Phase 4d-1 -- the interleaved two-interpreter baseline, measured 2026-08-08
+## Phase 4d-1 -- the interleaved two-interpreter baseline, measured 2026-08-08 at `107febcd` (superseded)
+
+**Superseded by the re-measurement below.**
+This section was measured at commit `107febcd`.
+Five speedups landed after that measurement and before this document's next update --
+`3799692d`, `b6b1d8a9`, `e1d50dda`, `c428ec8a`, `04ab4af6` -- and each moved every ratio in the
+table below, so this section no longer describes the interpreter's current performance.
+It is kept rather than replaced: a reader must be able to see that the numbers moved and why,
+not find one set silently swapped for another.
+The current numbers are in "Phase 4d-1 -- re-measured baseline after five speedups" below.
 
 **A new section, not an edit to the ones above.**
 The Phase 0 criterion rows were taken against a different oracle build and this phase's rule is
 that a baseline is measured at gate time, so none of those numbers are reused here.
 Both sides below were measured in one run, on the same machine, alternating.
 
 Produced by `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`.
 **Every heading from "Provenance" to "Axes this crate cannot run", and every table and paragraph
 under them, is that program's output byte for byte.**
 The prose before "Provenance" and after "Axes this crate cannot run" is written here and is not
@@ -508,20 +517,212 @@ cargo build --offline --release -p rexx-exec --bin rexx-run -p rexx-bench
 
 `--self-check` runs a single pair per axis and labels its own output as not a baseline; it exists
 so the harness can be exercised without spending twelve minutes and without producing a table that
 could be mistaken for one.
 
 The suite exits non-zero, and says so at the top of the section it would otherwise have produced,
 when an axis fails to complete or when an axis declared `Role::Blocked` stops failing. The second
 of those is the case that matters after Phase 5: three dimensions would otherwise keep appearing
 under "Axes this crate cannot run" with status 0 and an empty message, timed by nothing.
 
+## Phase 4d-1 -- re-measured baseline after five speedups, measured 2026-08-08
+
+**Why this section exists.**
+The section above was measured at `107febcd`.
+Five speedups landed after it -- `3799692d`, `b6b1d8a9`, `e1d50dda`, `c428ec8a`, `04ab4af6` -- and
+every task below Task 2b in this phase reads a baseline, so the stale numbers had to be replaced
+with current ones rather than annotated in place.
+That contradicts this phase's own no-optimisation rule (Global Constraints); the contradiction is
+recorded rather than argued away, in `task-2b-brief.md` and again here: this unit can produce a
+*current* baseline and attribution, and cannot produce a pre-optimisation one.
+
+**Same harness, same method.**
+This is another run of the same `rexx-bench-suite`, described in "Method, and the three choices
+that are not the obvious ones" above; that section is not repeated here.
+As before, every heading from "Provenance" to "Axes this crate cannot run" below, and every table
+and paragraph under them, is that program's output byte for byte; the prose before and after that
+range is written here.
+
+### Provenance
+
+| | |
+|---|---|
+| measured | 2026-08-08T21:02:25+02:00 |
+| repo commit | `16ec27be119bb5fd44760804fb67477c0732a222` |
+| oracle `bin/rexx` | `/home/moritz/dev/repos/ooRexx/build/bin/rexx` -- size=62600 bytes, mtime=2026-08-05 16:02:20.172072564 +0200, sha256=bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019 |
+| oracle `lib/librexx.so.4` | size=17853856 bytes, mtime=2026-08-05 16:02:20.064306594 +0200, sha256=42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb |
+| oracle `lib/librexxapi.so.4` | size=667792 bytes, mtime=2026-07-30 23:06:46.077533141 +0200, sha256=3536b76379c23fc7e3c4bce97b57d3c4812291ce5d68e71adc507ee690dc7d66 |
+| this crate `rexx-run` | `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run` -- size=12996808 bytes, mtime=2026-08-08 15:02:55.950099476 +0200, sha256=c3b2516069a1b5f5d504613986f00b69e0c0fc892c041958212d45a699c0e967 |
+| address-space cap | `ulimit -v 8388608` KiB, **both sides, every axis** |
+| pairs per axis | 9 sampled, 1 warm-up pair(s) discarded, oracle and this crate alternating |
+| pairs for the offset line | 51 sampled, 5 warm-up |
+| statistic | median; interval is the distribution-free sign-test interval for the median at a 95% target |
+| working directory of every child | a fresh empty temporary directory |
+
+### Fixed per-process offset (`startup.rex`)
+
+**Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two numbers below are each side's own fixed cost, reported so every axis above can be read net of it -- not as a result about which interpreter starts faster.
+
+| side | median | min | max | 95.1% interval | spread |
+|---|---:|---:|---:|---|---:|
+| oracle | 7.353 ms | 5.161 ms | 9.126 ms | 6.997 - 7.710 ms | 53.9 % |
+| this crate | 1.727 ms | 1.245 ms | 2.322 ms | 1.555 - 1.908 ms | 62.4 % |
+
+Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides equally.
+
+### Axes
+
+`iters/s` is the program's own loop bound divided by the median wall time. `iters/s net` divides by the median wall time less that side's per-process offset above.
+
+| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |
+|---|---:|---|---:|---:|---:|---|---:|---:|---:|
+| `arith` | 500000 | oracle | 1.1578 s | 1.1476 s | 1.1712 s | 1.1529 - 1.1701 s | 2.04 % | 431867 | 434628 |
+| `arith` | 500000 | this crate | 3.1273 s | 3.1064 s | 3.1385 s | 3.1214 - 3.1372 s | 1.03 % | 159882 | 159971 |
+| `compound` | 5000000 | oracle | 1.1459 s | 1.1300 s | 1.1531 s | 1.1405 - 1.1487 s | 2.02 % | 4363428 | 4391609 |
+| `compound` | 5000000 | this crate | 7.2720 s | 7.2621 s | 7.4648 s | 7.2633 - 7.2859 s | 2.79 % | 687571 | 687735 |
+| `strings` | 3000000 | oracle | 0.8654 s | 0.8499 s | 0.8853 s | 0.8551 - 0.8782 s | 4.08 % | 3466493 | 3496199 |
+| `strings` | 3000000 | this crate | 9.3247 s | 9.2433 s | 9.5101 s | 9.2453 - 9.4594 s | 2.86 % | 321725 | 321785 |
+| `varlookup` | 19000000 | oracle | 1.2064 s | 1.1893 s | 1.2547 s | 1.2009 - 1.2201 s | 5.42 % | 15748922 | 15845500 |
+| `varlookup` | 19000000 | this crate | 5.2328 s | 5.2208 s | 5.2702 s | 5.2224 - 5.2570 s | 0.94 % | 3630966 | 3632165 |
+
+#### Ratios and the gate call
+
+The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's median over the oracle's) are the same number, because both sides run the same iteration count.
+
+**The ratio interval is indicative, and the verdict is not taken from it.** It divides one side's interval by the other's, so its joint coverage is at least 92.2% by Bonferroni -- one minus the two sides' miss probabilities added -- not the 96.1% either side carries alone. The verdict applies Global Constraints' rule directly: this crate's point estimate against the oracle's interval, slow side.
+
+| axis | oracle median | this crate median | ratio | ratio interval | verdict |
+|---|---:|---:|---:|---|---|
+| `arith` | 1.1578 s | 3.1273 s | 2.70x | 2.67x - 2.72x | SLOWER |
+| `compound` | 1.1459 s | 7.2720 s | 6.35x | 6.32x - 6.39x | SLOWER |
+| `strings` | 0.8654 s | 9.3247 s | 10.77x | 10.53x - 11.06x | SLOWER |
+| `varlookup` | 1.2064 s | 5.2328 s | 4.34x | 4.28x - 4.38x | SLOWER |
+
+#### Same work on both sides
+
+A wall time is only about the workload if the workload ran. Every sampled run on each side printed the same bytes, and the two sides printed the same bytes as each other.
+
+| axis | stable within a side | identical across sides | stdout |
+|---|---|---|---|
+| `arith` | yes | yes | `4629643519330627.7808` |
+| `compound` | yes | yes | `5000000` |
+| `strings` | yes | yes | `138000000` |
+| `varlookup` | yes | yes | `19000000` |
+
+### `samples/rexxcps.rex`
+
+The oracle's own clauses-per-second benchmark, run from the read-only C++ tree. It self-calibrates: a trial that comes in at or under a second is run again at twice the count, so the two sides do **different amounts of work** and their wall times are not directly comparable. The clauses-per-second figure each side prints is per clause and is the comparable one. Each side's `Averaged:` line is quoted so the asymmetry is visible rather than inferred.
+
+| side | wall median | wall interval | `Averaged:` |
+|---|---:|---|---|
+| oracle | 1.7955 s | 1.7641 - 1.8290 s | `Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)` |
+| this crate | 4.6776 s | 4.6605 - 4.7435 s | `Averaged: 100 x 100 iterations of 1000 clauses (over 4.3s)` |
+
+| side | median cps | min | max | 96.1% interval | spread |
+|---|---:|---:|---:|---|---:|
+| oracle | 16873935 | 16083829 | 17204168 | 16550975 - 17194214 | 6.64 % |
+| this crate | 2308020 | 2282779 | 2324259 | 2293006 - 2317831 | 1.80 % |
+
+**Internal cps ratio: 7.31x** (oracle median over this crate's median), interval 7.14x - 7.50x.
+
+### Axes this crate cannot run
+
+Measured here rather than left out of the table, with the status and message each one actually produced. These belong to later tasks in this phase; what belongs to this one is that they are visible.
+
+| axis | exit status | message |
+|---|---:|---|
+| `alloc` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
+| `dispatch` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
+| `heapshape` | 120 | `rexx-exec: a message send is not implemented (Phase 5)` |
+
+### Spread, and why this run is accepted despite exceeding the old band on two sides
+
+**The bar is the committed baseline's band, and this run does not clear it everywhere.**
+The `107febcd` section's this-crate spreads ran 1.02% to 2.40% and its oracle spreads ran 2.29% to
+7.11%.
+This run's this-crate spreads are `arith` 1.03%, `compound` 2.79%, `strings` 2.86%, `varlookup`
+0.94%; its oracle spreads are `arith` 2.04%, `compound` 2.02%, `strings` 4.08%, `varlookup` 5.42%.
+Two oracle rows (`strings`, `varlookup`) and two this-crate rows (`compound`, `strings`) sit above
+the old band's upper end, `strings` on the oracle side by nearly a point.
+None of that is stated away: this run is measurably noisier than `107febcd`'s, on four of the eight
+rows.
+
+**It is accepted anyway, on the ratio intervals rather than the raw spreads.**
+The gate criterion this task exists to support is per-axis attribution, and that is what the ratio
+interval carries: 2.67x-2.72x, 6.32x-6.39x, 10.53x-11.06x, 4.28x-4.38x.
+None of the four comes near overlapping another, so the four axes remain distinguishable from each
+other despite the wider per-row spread -- the same conclusion "The spread of ratios across axes is
+real, not measurement noise" reached below for `107febcd`, reached again here on noisier input.
+What the wider spread costs is precision on each individual ratio, not the ability to rank the four
+axes against each other, and ranking them is what Tasks 3 and 4 need.
+
+**This is a different bar from the one the indicative run failed.**
+The controller's own contended run, taken with other work on the machine and recorded in
+`task-2b-brief.md` as direction-only, hit 82.77% spread on `strings` and 32.84% on `arith` -- one to
+two orders of magnitude past the committed band, wide enough that its ratio intervals would have
+overlapped each other and said nothing.
+This run's worst row is 5.42%, roughly a quarter of the indicative run's *best* row.
+A spread of 2.86% on one row is not the same failure as a spread of 82.77%, and rejecting this run
+because two rows sit a few tenths of a point above `107febcd`'s band, when the attribution the band
+exists to protect still holds, would be discarding a usable measurement over noise smaller than what
+it is meant to catch.
+
+**As a coarse cross-check**, three of this run's four ratios land close to the contended run's:
+`arith` 2.70x against 2.60x, `compound` 6.35x against 6.32x, `strings` 10.77x against 10.68x.
+`varlookup` is the exception: 4.34x against 4.28x is also close in absolute terms, but the contended
+run's own spread on that axis was not reported cleanly enough to know what the closeness means.
+This is one sentence of agreement between a clean run and a noisy one, not a second baseline; the
+noisy run's own spreads make it unusable as a check on anything finer than "same order of magnitude."
+
+### The internal cps ratio is unstable across runs, more than the wall-clock ratios are
+
+Three measurements of the same tree gave three different internal cps ratios: 7.41x
+(`--self-check`, one pair, explicitly not a baseline), 6.21x (the controller's contended run), and
+7.31x (this run, nine pairs, the accepted one).
+That spread -- 6.21x to 7.41x, about 1.2x peak to peak -- is wider than the spread across the four
+wall-clock ratios' repeat measurements in this same task, none of which moved by more than a few
+hundredths of a unit between runs.
+The 7.14x-7.50x interval reported above is this run's own sign-test interval and does not capture
+that between-run movement; it describes how much the median would move if this exact run were
+repeated, not how much it moved when the run itself, the contention on the machine, or both changed.
+Read 7.31x as the accepted figure for this measurement and read 6.21x-7.41x as the honest range
+across what has actually been observed, not as a tighter interval around 7.31x.
+
+Recorded as an observation about measurement stability, not attributed: nothing in this task looked
+for why the internal cps ratio moves more than the loop-axis ratios do between runs.
+
+### What moved between the two baselines, and why
+
+**Every ratio fell, and none fell by the same factor.**
+
+| axis | `107febcd` ratio | this run's ratio | change |
+|---|---:|---:|---:|
+| `arith` | 3.52x | 2.70x | -23% |
+| `compound` | 12.15x | 6.35x | -48% |
+| `strings` | 13.70x | 10.77x | -21% |
+| `varlookup` | 23.07x | 4.34x | -81% |
+| `rexxcps` (internal) | 9.26x | 7.31x | -21% |
+
+The five commits named at the top of this section are what moved these: `3799692d` (inline a
+literal that already spells a small integer), `b6b1d8a9` (small-integer arithmetic without building
+a decimal), `e1d50dda` (step a counted loop without building a decimal per iteration), `c428ec8a`
+(compute a clause's indent once per position instead of once per step), `04ab4af6` (fill the indent
+table in one walk instead of one per position).
+`varlookup`'s far larger drop is consistent with `e1d50dda` landing directly in that axis's
+loop-stepping path, while `arith`, `strings` and the internal `rexxcps` ratio -- which do not depend
+on counted-loop stepping the same way -- moved by a narrower and more similar 21-23%.
+This task did not re-attribute which commit caused which axis's share of its drop; that division of
+labour is unattempted here and is Task 3's or Task 4's if it is wanted.
+
+No optimisation was attempted in this task and none is proposed here.
+The five commits that moved these numbers predate it.
+
 ## What is still missing
 
 - macOS 15 arm64, Windows/MSVC, FreeBSD 14.2, and OpenBSD 7.8 rows. None have been run. CI must
   add a job per platform that builds the C++ oracle, runs this same suite, and either commits
   numbers here or documents why a platform could not produce them (the known OpenBSD SIGSEGV is
   the anticipated case for that one, per Task 0.7's own text). `rexx-bench-suite` is Linux-only as
   written: the address-space cap is a `/bin/sh` builtin, and the fingerprints come from `ldd`,
   `stat` and `sha256sum`.
 - A Rust side for the axes under "Axes this crate cannot run" above. That table is the live list,
   emitted by the suite from the roles in its own source; each entry exits on a message send and is
