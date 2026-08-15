# SDD ledger -- plan: docs/superpowers/plans/2026-08-09-phase-4f-optimisation-loop.md

## Unit 0 -- closed 2026-08-09 as `acb363bc`

The upfront noise characterisation was **withdrawn** mid-run by the coordinator (`ff46ee10`); it conflated the loop's paired accept rule with the gate's absolute verdict and priced the harder one in advance.
What landed instead is the harness that survives the withdrawal and the escalation rule's tooling.

* `rexx_bench::child` -- one capped, directory-pinned child wrapper for both harnesses, removing the `/bin/sh`+`Instant` against bash+`date` split that contaminated the gate's 7.2% figure. The committed baseline's argument vector is asserted byte-identical.
* `rexx-bench-band` -- one pass per invocation, reduction over passes, for the escalation path.
* `rust/bench-control/alloc4c-101.rex` -- the sensitivity control, +1% loop bound, relationship asserted.
* The plan's Unit 0 carries the stopped run's reading at the n it rests on, correcting the summary that pinning widened the spread.

Report: `unit-0-report.md`. No measurement outstanding; no interpreter code touched.
