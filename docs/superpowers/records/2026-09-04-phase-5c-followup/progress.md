# 5c follow-up — ledger

Plan `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`. Append-only.

## Task 0 — receiver override

Controller, inline. Four claims predicted and confirmed (`task-0-report.md`). Sweep of the other
covered classes measured: 105 sharpenable rows, zero would move today; not landed, Moritz's call.
`defaultSize` found observable via `setBufferSize(0)` and written into the plan's Task 1.

Committed `fdf4c6624`; all seven gates 0 (`task-0-report.md`, Gates).

## Task 1 — where the bytes live

Agent `task1-repr-spike`, in three `git archive` extracts under
`claude-build-scratch/5c-followup-task1/` (kept). Report `task-1-report.md`, brief `task-1-brief.md`.
The controller re-read the sha256s, the four `size_of` probes, the perf cells, the RSS readings, the
subclass `EXPOSE` outputs and the `trace i` lines from the raw files; all match the report.

**D80 decided, by measurement**: `Body::Instance` gains `native: Option<Box<BufferState>>`,
`BufferState { bytes: Vec<u8>, capacity, default_size }`. Append loop: object-variable shape at
2.63× the instructions, 11× the cycles, 31× the wall time and 4,068,068 KB against 17,324 KB peak
RSS of the in-body shape; a separate `Body::Buffer` variant loses `pools`/`own`/`name` and refuses a
subclass's `EXPOSE` the oracle answers. `size_of::<Body>()` stays 80 under every shape probed.

**Review question 3 (`defaultSize`)**: carried. **Review question 1 (Task 3 split)**: adopted the
report's §5 — one core-extraction commit, then readers, mutators, caseless, conversion; written into
the plan. **Review question 2 (sweep)**: still Moritz's.
