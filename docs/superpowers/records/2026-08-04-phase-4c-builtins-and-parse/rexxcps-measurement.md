# rexxcps measurement

Applies the R1/R2/R3 criteria from `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md:508-510` to `samples/rexxcps.rex`.
This is a measurement only: no source file changed, nothing committed.

## What the benchmark exercises

`rexxcps.rex` (oracle copy at `/home/moritz/dev/repos/ooRexx/samples/rexxcps.rex`) is pure Phase 1-4c surface, no OOP.
It uses: `SIGNAL ON NOVALUE`, `PARSE SOURCE`, `PARSE VERSION`, `TIME('R')`, `FORMAT`, `SUBSTR`, `WORD`, `LENGTH`, numeric `DO` loops (including `DO j=1.1 TO 2.2 BY 1.1`), compound (stem) variables with a variable tail (`acompound.key1.loop`), `SELECT`/`WHEN`, `PARSE VALUE ... WITH ... +5 ...`, `PARSE VAR var pattern (p0) template` (parenthesised-variable pattern), `TRACE VALUE`/`ADDRESS VALUE` with expression arguments, `CALL` to an internal label with comma-continued arguments, `LEAVE`, `ITERATE`, and `EXIT`.
It self-calibrates: it runs an initial trial with `count=100`, and only bumps `count` and runs a second trial if the first trial's total is at or under one second.
No classes, methods, `::REQUIRES`, or message sends anywhere in the file, so none of Phase 5's known gaps are in its path.

## Commands run and exit status

All oracle invocations wrapped per the environment rules; all runs from fresh scratchpad subdirectories; stdout/stderr captured to separate files, never piped, exit status read directly.

* `cargo build --release --bin rexx-run` (run from `rust/`) -- exit 0.
* Single-run sanity checks (directories `rexxcps-run1`, `rexxcps-oracle1`):
  * `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run /tmp/.../rexxcps-run1/rexxcps.rex` -- exit 0.
  * `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /tmp/.../rexxcps-oracle1/rexxcps.rex )` -- exit 0.
* Five-run batches (directories `rexxcps-oracle-batch`, `rexxcps-rust-batch`, one copy of the benchmark per directory, five sequential invocations against that same copy):
  * Oracle, 5x: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /tmp/.../rexxcps-oracle-batch/rexxcps.rex )` -- exit 0 all five times.
  * Rust, 5x: `/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run /tmp/.../rexxcps-rust-batch/rexxcps.rex` -- exit 0 all five times.
* `wc -c` on all ten `.stderr` files -- all zero bytes.
* `/bin/grep -a "Failed" oracle.*.stdout rust.*.stdout` -- zero matches, all ten files.
* `diff -q` between masked stdout, oracle run *i* vs rust run *i*, for i=1..5 -- all five report MATCH.
* Wall-clock timing taken externally around each of the ten batch runs with `date +%s.%N` before/after (not the benchmark's own `TIME('R')`).

The build was `--release`; no debug-build numbers are reported anywhere below.

## R1: stdout equality after masking timing fields

Both interpreters print the identical five-line report shape:

```
----- REXXCPS 2.2 -- Measuring REXX clauses/second -----
 REXX version is: REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026
       System is: LINUX
        Averaged: <count> x <averaging> iterations of 1000 clauses (over <T>s)

     Performance: <cps> REXX clauses per second
```

Masking the two timing-derived fields (the `Averaged:` line's count/averaging/seconds, and the `Performance:` line's clause count) with `sed`, the two sides are byte-identical for every one of the five paired runs.
The `REXX version is:` and `System is:` lines -- not masked, not timing fields -- also matched verbatim in every run, meaning the crate reproduces the oracle's `PARSE VERSION` string exactly.
No `Failed` line was printed by either side in any of the ten runs (the redundant check the spec calls out as insufficient on its own, so it is reported here as corroborating, not as the criterion).
Stderr was empty on all ten runs; exit status was 0 on all ten runs.

**R1 verdict: PASS.** Full stdout equality holds after masking only the timing fields, on every paired run measured, and no evidence of a silently-skipped clause was found.

## R2: the cps ratio

Five runs each, same machine, same session, immediately following the R1 batch above (same processes, same data).

Oracle `Performance:` cps: 16898357, 16821197, 16877794, 16767003, 16744375.
Mean 16,821,745.2; min 16,744,375; max 16,898,357; spread (max-min)/mean = 0.92%.
All five oracle runs bumped `count` from 100 to 200 and ran the second trial (`Averaged: 200 x 100 ... over 1.2s` on every run).

Rust `Performance:` cps: 1673113, 1679544, 1681882, 1686578, 1671227.
Mean 1,678,468.8; min 1,671,227; max 1,686,578; spread (max-min)/mean = 0.91%.
All five rust runs kept `count` at 100 and never ran a second trial (`Averaged: 100 x 100 ... over 5.9-6.0s` on every run) -- the first trial already exceeded one second, so the self-calibration loop exited after trial 1.

`ratio = oracle_cps / rust_cps` using the two means: **10.02**.
Taking the least-favourable-to-rust and most-favourable-to-rust combinations across the five-run spreads gives a ratio range of **9.93 to 10.11** -- a 1.8% band, far narrower than the gap to either the 1.0 or 1.5 threshold.

**R2 verdict: the >1.5 fail band, and not close.** The run-to-run spread (under 1% on each side) is nowhere near large enough to put the true ratio anywhere near the 1.0-1.5 debt band, let alone at or under 1.0.

## R3: external wall-clock cross-check (context, not a pass/fail gate here)

Wall time was measured externally (`date +%s.%N` around each invocation, not the benchmark's internal `TIME('R')`) for all ten batch runs:

* Oracle wall time: mean 1.8054s, spread 1.06% (five runs: 1.796, 1.814, 1.801, 1.800, 1.815s).
* Rust wall time: mean 6.5714s, spread 1.17% (five runs: 6.592, 6.563, 6.519, 6.587, 6.596s).

Because the oracle's calibration bumped `count` (both trials ran: 10M clauses at `count=100`, then 20M at `count=200`, for 30M total clauses of real work) while rust's calibration did not (only the 10M-clause trial ran), the two sides did different amounts of work, so raw wall-time ratio is not directly comparable to the cps ratio -- it has to be normalized by clauses actually executed.
Normalizing: external oracle cps = 30,000,000 / 1.8054s = 16,617,116; external rust cps = 10,000,000 / 6.5714s = 1,521,752.
External ratio = 10.92, against the internal-timer ratio of 10.02 -- an 8.96% relative difference, under the spec's 10% agreement bar.
This is an approximate cross-check (it infers total clauses from the printed `count`/`averaging` values and the calibration logic rather than instrumenting the benchmark directly), but it corroborates that the internal `TIME('R')`-based ratio is not being flattered by a defect in the crate's own timer: an external, holistic wall-clock measurement lands within 9% of it, on the same side.

The oracle's per-clause wall time and internal-timer time are close (1.8s wall for ~1.8s of measured-plus-discarded-trial body time), meaning oracle startup/parse overhead is near zero.
For rust, roughly 0.6-0.7s of the 6.57s wall time is not accounted for by the internal timer's own trial-1 body time (5.9-6.0s), consistent with process startup and parse overhead rather than a per-clause effect -- worth noting since it is the reason the external ratio (10.92) runs slightly higher than the internal one (10.02), not lower.

## ulimit -v vs this crate's 512 MiB stack reservation

**Correction, 2026-08-08: this paragraph had the subject backwards.** This crate reserves 512 MiB of address space via `INTERPRETER_STACK_BYTES` before running anything, inside the 1 GiB `ulimit -v` cap the environment rules require; the oracle has no equivalent fixed reservation, so it runs the same benchmark with roughly twice this crate's effective address-space headroom under the same cap.
This does not affect the cps figures reported here: the benchmark's inner loop does not approach either interpreter's memory ceiling (`rexx-run` and the oracle both completed all ten runs without any allocation failure or signal), so the asymmetry is a headroom difference, not a measured-performance confound.
It should still be kept in mind if this benchmark is later run in a tighter memory budget, since the two interpreters would not be equally squeezed.

## Bottom line

* R1 (correctness before speed): **pass**, exact stdout match after masking only the timing fields, over five paired runs, zero `Failed` lines, zero stderr, exit 0 throughout.
* R2 (cps ratio): **oracle_cps / rust_cps = 10.02** (range 9.93-10.11 across the five-run spread) -- lands in the **fail band (>1.5)**, and the run-to-run variation (under 1% on each side) is far too small to make the fail-vs-debt-vs-pass call ambiguous.
* R3 (external cross-check): informational here, not gated by this task; the externally wall-clocked, clause-count-normalized ratio (10.92) agrees with the internal-timer ratio (10.02) to within 9%, so the failing R2 number is not an artefact of the crate's own `TIME('R')` implementation.

`git status --short` is empty; no source file was touched.
