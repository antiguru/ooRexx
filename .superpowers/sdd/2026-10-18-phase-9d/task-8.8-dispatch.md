# Task 8.8: Process items and remaining queued issues

## Scope

Execute section 8.8 of `task-8-batches.md` and the unassigned process, unclear and fixed rows of `task-8-triage.tsv`. Tasks through 8.7c are complete. The starting revision is `0c2ff21ae1844e86c1937b58fc82f080fc43b5ab` on `plan/rust-rewrite` in the existing linked worktree. Tasks 9 and 10 are outside this dispatch.

## Requirements

* Read `task-8-brief.md`, the header and section 8.8 of `task-8-batches.md`, `global-constraints.md`, `task-6-dispatch-rules.md`, and `rust/CLAUDE.md`.
* Treat `task-8.8-preflight.md` as the item index once available. Read each assigned queued file fully, including its disposition and rulings.
* Execute sequential implementation slices with review after each slice. The controller assigns each slice's exact items. Leave every other item unchanged.
* Derive semantics from the local reference documentation. Show regression witnesses red before implementation. Compare oracle stdout, stderr and status separately. Never run a listed oracle crash.
* A new numbered Deviation requires Moritz's ruling. Report the derived rule, oracle behavior, and concrete choice to the controller. Continue independent items while that choice is pending.
* Close each item using the batch header's queued-file and inventory conventions. A fixed claim needs a fresh probe or an existing explicit disposition whose evidence is checked. Reconcile stale inventory rows without replacing their reasons.
* Keep comments. Add no unsafe sites or dependencies. Use existing helpers where they satisfy the contract.

## Execution and checks

* Use `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/codex-p9d-t88-target`, `memcap 8G`, and Cargo `-j 4`. Crate interpreter probes use `memcap 2G timeout`; oracle probes use the documented oracle wrapper in fresh empty directories.
* Check commands run in the foreground with bounded waits. Commit implementation before verification, then freeze the measured source until commands finish. Report BLOCKED with commit and live command details if a foreground service or check cannot finish within the agent turn. Do not start background watchers.
* Run focused regressions, formatting, and clippy for affected crates. At batch completion run the required corpus, coverage, sourceline, collect-stress, refusal and owner checks, the whole suite with a per-test delta, and the workspace test. Existing failures are evidence to revisit in this batch, not permission to ignore them.
* Runtime changes require `callgrind.sh` on every program, with no `-p`, against this slice's recorded BASE. The batch header supersedes the old five-program rule. Record libc-subtracted deltas and scheduling spreads. Keep temporary builds and raw profiles under `/tmp`; copy cited summaries into `docs/superpowers/records/2026-10-18-phase-9d/t8.8/`.
* Do not delete build caches until the controller has completed validation. Do not push, merge, modify another worktree, or touch the user's untracked archive.

## Report

Write the assigned slice report beside this brief. Record changed items, regression evidence, commands and actual exit statuses, commits, remaining items, and any semantic ruling request. Return only status, commits, one test-summary line, and concerns. Do not spawn subagents.
