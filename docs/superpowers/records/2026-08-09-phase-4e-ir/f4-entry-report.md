# Phase 4f entry measurement -- working notes

**Every figure this work produced is in `docs/superpowers/plans/phase-4f-record.md`, entry 1, and is not repeated here.**
A restatement of a measurement is a new claim that needs the measurement re-run rather than the prose reread, so these notes carry the process and the evidence trail only.

## What was run, in order

1. `git rev-parse HEAD` -> `131f1b4061545fcd180346c1a470a1fd9da8bcc4`, working tree clean.
2. `cargo clean -p rexx-exec -p rexx-bench --release`, then
   `cargo build --release -p rexx-exec --bin rexx-run -p rexx-bench --bin rexx-bench-suite`.
   The clean is deliberate: a clean `git status` says nothing about `target/`.
3. Harness arm checks (four, listed in the record's own section).
4. `rexx-bench-suite --self-check` on each arm, to confirm every axis completes before spending half an hour.
5. The sitting: `f4-entry-evidence/sitting.log` is the driver's own timestamped log, four blocks in the order `ir`, `tree-walker`, `tree-walker`, `ir`, each `./target/release/rexx-bench-suite --engine ARM` with no other arguments.
6. The sensitivity control, after the sitting:
   `./target/release/rexx-bench-band --header --engine ir --config control-abba --pass 1 --axes alloc4c,<abs>/bench-control/alloc4c-101.rex,<abs>/bench-control/alloc4c-101.rex,alloc4c --pairs 5 --warmup 1`

## Evidence in the tree

`f4-entry-evidence/` holds the primary output, so the record's tables can be rechecked without re-running anything.
**It is not committed** -- `.gitignore` excludes `.superpowers` -- so it is local to this machine, which is why the record quotes every median its ratios are computed from rather than pointing here for them.
`rust/bench-baselines/` was considered and rejected as a home: its README binds that directory to `rexx-arms`' tab-separated rows, and these are `rexx-bench-suite` markdown reports.

| file | what it is |
|---|---|
| `suite-block1-ir.md` .. `suite-block4-ir.md` | each block's `rexx-bench-suite` standard output, unedited, provenance block included |
| `control-alloc4c-101.tsv` | the control pair's rows, one per sampled run, `rexx-bench-band`'s own tab-separated format |
| `sitting.log` | block boundaries, arms, exit statuses and load averages |

The record's per-block ratios are `this crate median / oracle median` recomputed from the four-decimal medians in those files, not copied from the harness's two-decimal ratio column.

## Decisions taken here that the record inherits

* **Unpinned and uncounted** (`Wrapper::default()`), because the only oracle figures on the record were taken that way and both switches are measured to move the ratio. Fixed at the top of the record for every later entry.
* **ABBA on the arm rather than one block each.** The suite measures one arm per invocation, so the two arms cannot share a block; balancing the order is what keeps a drift across the sitting out of the arm difference. The resulting arm column is still a ratio of ratios and the record says so.
* **The control was run even though nothing escalated**, because the entry calls arm differences of two to three per cent real, and that is a sensitivity claim whether or not an axis is near 1.0.

## What was deliberately not done

* No optimisation, no candidate, no attempt.
* `rexx-arms` was not used: it compares two arms of one build and does not reach the oracle.
* No axis was re-run hoping for a better sitting.
* `phase-4e-anchor.md` and `perf-baseline.md` were not edited or re-measured; both already carry their own note saying their figures are tree-walker figures.
