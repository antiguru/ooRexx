# Task 9 fix round 1

Phase 6.1 Task 9 (simulation policies, invariants, decision trace and replay) is implemented at
`60f21ad1b`, `296cac74f`, `88e818648` and reviewed: **Needs fixes**. Read, in order:

1. `.superpowers/sdd/2026-10-07-phase-6-1/global-constraints.md`: binding rules (memcap on every
   interpreter or test run with `timeout` inside memcap; no `bash -c`; `rm` only literal absolute
   paths, no globs, no variables; stage by path; commit with `-F`).
2. `.superpowers/sdd/2026-10-07-phase-6-1/task-9-brief.md`: the task's requirements.
3. `.superpowers/sdd/2026-10-07-phase-6-1/task-9-report.md`: what was built.
4. `.superpowers/sdd/2026-10-07-phase-6-1/task-9-review.md`: the findings. Its probe programs and
   injection copy are under `/tmp/claude-1000/p61/t9r/` and may be reused.

Scratch: `/tmp/claude-1000/p61/t9f1/`. Own target directory there, `CARGO_INCREMENTAL=0`.

## Fix

* **I1, silent replay divergence.** Under `sim:replay=FILE` the run must refuse loudly (non-zero
  status, a stderr line naming the trace file and the first diverging decision: its index, the kind
  and value the file holds, and what the run did) when:
  * a recorded pick is out of range for the run's ready set (today clamped by `pick.min(last)`);
  * the run asks for a decision of a different kind than the file's next one;
  * a recorded point (preemption step, gc draw, or any other position-keyed decision) is passed
    without the run reaching it;
  * the run ends with decisions left in the file;
  * the run needs a decision after the file is exhausted (today falls back to the front).
  Write the trace hash into the trace header and check it against the file's decisions on load
  (a damaged file is refused before running). Use or delete `pub fn trace_hash` (M3); no dead pub.
  Tests: the reviewer's cases (another program's trace; `pick 999` inserted; a truncated trace; an
  extra trailing decision; a damaged header hash) each refuse with the expected line; the existing
  same-program replay tests stay green. Each test asserts the stderr line, not just the status.
* **I2, quadratic invariant check.** Rewrite `check_invariants` / `has_wake_source` /
  `holds_park_reason` as one pass building per-handle state (ready, sleeper, guard waiter, message
  waiter, free) in vectors indexed by handle, then checking each handle in O(1). Every invariant
  that fires today must still fire with the same message: rerun the reviewer's nine injected
  corruptions (`/tmp/claude-1000/p61/t9r/inj`, test `inj_uncovered_invariants`) against your tree
  and the committed six, and commit the nine as permanent tests beside the six (they are cheap via
  the existing `Corruption` hook). Re-measure the reviewer's scaling probe (n started activities
  parked on one EventSemaphore then posted, `sim:1,uniform:0.5` against `every`, n = 250, 500, 1000,
  2000) and record the table; target near-linear.
* **M1.** A violation is refused once: stop checking after the first refusal set
  `sim.inconsistent`. The tests assert the exact stderr refusal set, not `contains`.
* **M2.** `trace=` with an empty path is refused at parse; a trace that cannot be written changes
  the exit status (non-zero) besides the stderr line; two `trace=` (or two `replay=`) items are
  refused; a comma inside a path names the right file in the refusal, or the parse refuses with a
  message saying paths may not contain commas.
* **M4.** The exploration test asserts all three interleavings of the two-activity program under
  `pre:1,k=3` (reviewer: seeds 1-40 reach all three); widen the seed range as needed.
* **I3 record only.** In the gate record's `## Task 9` section add the base61 column (6.1 base
  `e6af1198b`, binary `/tmp/claude-1000/p61/t1/bin/base/rexx-run`, verify its sha256 against the
  gate record's `## Task 1`) and the running totals for all eight programs, measured after your
  fixes with the reviewer's command (check 5 of the review). Do not try to fix heapshape: a separate
  agent is bisecting the +1.03% that predates Task 9, and the ruling goes to Moritz. Report the
  numbers only.

Per-task check from global-constraints (fmt, clippy -D warnings, plain workspace test, the
`REXX_CORPUS_GATE=1` corpus pair). If a `Loud` constructor is added or relabelled, re-derive
`refusal-sites.tsv` and keep `refusal-dispositions.tsv` green in the same commit.

## Report

Write `.superpowers/sdd/2026-10-07-phase-6-1/task-9-fix1-report.md`: per finding, what changed
(file:line), the test that pins it and that test's command and result; the I2 scaling table; the
perf table with base and base61 columns and the exact command. Commit code and report (report with
`git add -f`). Return only: status (DONE / DONE_WITH_CONCERNS / BLOCKED), commit shas, one-line
test summary, concerns.
