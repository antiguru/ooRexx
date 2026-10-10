# emptyloop wall clock is +4.6% in cycles at fewer instructions

Found by Phase 6.1 Task 2 (accepted then by Moritz, ledger l.48) and re-measured at the Task 12 close. Queued 2026-10-10, left open by Moritz.

emptyloop runs +4.38% to +4.62% in cycles against base61 on the close binary and on both of its layout pads. Its instructions are 0.63% fewer and its context switches equal, and base61's own pad moves it by only +0.07%. So layout alone does not explain it, and the cycles are spent in the loop itself. pingguard (+4.53%, largest pad move 2.78 points) is unattributed too.

Evidence: the wall-clock table in `docs/superpowers/plans/phase-6-1-gate.md` (Task 12) and `.superpowers/sdd/2026-10-07-phase-6-1/task-12-report.md` "## Wall clock". Next step: `perf stat` with frontend and backend stall counters, or a per-instruction cycle profile of `loop_advance` on both binaries. The sandbox gives one PMU slot, so measure one counter per run.
