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

## Task 2 — the constructor carries state

Agent `task2-buffer-state`, committed `2d533034a`; brief `task-2-brief.md`, report `task-2-report.md`.
**Two background-job completion notifications were lost** (the mutation run, then the fast checks):
the agent sat idle each time until the controller's pid waiter noticed and a message woke it. Waiters
keyed on the job's pidfile are now part of every dispatch.

**Controller review.** Code read in full (`dispatch.rs`, `body.rs`, `value.rs`, `string.rs`):
argument conversion precedes `buffer_state_mut` everywhere; `Object~copy` deep-clones the boxed
state (`native_copy` clones the body). Probed on the oracle and the built binary
(`scratchpad/task2-review/oracle/q*.rex`): `buf == 'abc'` is `0` on both (Object identity, not a
string compare); `buf == buf`, `~copy` independence, `~class~id`, `isA`, `hasMethod`, the 97.1
condition — all agree; `'abc' == buf`, `length(buf)`, `buf~request('STRING')`, `buf~makeString` are
`MAKESTRING`-loud, rc 120. **No silent divergence.** `identityHash > 0` differs for every object and
is the known unmatchable, not this task's.

**Gates: G1 G2 G6 G7 = 0; G3 G4 G5 = 101** on `refusal_sites.rs` alone — 19 `run.rs` definition
lines shifted by one; re-derived in the follow-up commit below. The fast-check list in every brief
now includes `--test refusal_sites` whenever `src/*.rs` changes.

**Decision (controller, plan amendment): each task files its own witness.** The differential runs
only `SUBSET_FILES` programs, so a witness in `corpus/unfiled.txt` is a witness nothing runs — and
Task 2's own M3 mutation (growth without doubling) was caught by `mutablebuffer_state.rex` and
nothing else. Leaving both witnesses unfiled across Task 3's five commits would leave that class
of defect ungated for the phase's whole middle. Filed here: `phase-5c.txt` + `EXPECTED_SUBSET_5C`
gain the two programs; `unfiled.txt` loses them; the differential reads `359 of 359` (357 at 5d's
close); the pin's inversion is red exactly on `phase_5c_subset_matches_the_committed_list`; verified
in a clean `git archive` extract with its own target directory. Task 4 keeps the extract
verification of the whole set and the close.

**Carried to the conversion commit**: `'abc' == buf` and `length(buf)` reach the buffer through the
required-string protocol (`MAKESTRING`), while `buf == 'abc'` is Object identity on the oracle —
binding `MAKESTRING` must not turn the latter into a string compare.
