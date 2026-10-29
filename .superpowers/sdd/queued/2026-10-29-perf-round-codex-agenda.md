**OPEN, queued 2026-10-29 by the controller from an external (codex) review of 9af69058d; Moritz 2026-10-29 "lgtm" to the placement: perf round after 9d.**

# Perf round agenda from the external review

* Collection sweeps every arena slot up to the historical peak (`rexx-core/src/heap.rs:376`): benchmark a large burst, release, then a small steady workload; occupied-slot indexing or reclaimable segments.
* An arithmetic site gives up small-integer arithmetic after one miss (`ir/drive.rs:865`): measure a retry with backoff against the per-op branch cost already measured (about 0.5% on rexxcps).
* Method sends repeat lookup where routine calls cache resolution (`dispatch.rs:3940`): a guarded send cache.
* Worker pool overflow is unbounded, 512 MiB stack reservation each (`scheduler/pool.rs:149`): admission rules that keep progress.
* Measure against a version-matched -O3 oracle: wall time, instructions, allocation volume, GC pauses, one workload matrix.
