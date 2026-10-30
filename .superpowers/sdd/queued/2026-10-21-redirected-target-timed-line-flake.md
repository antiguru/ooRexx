**CLOSED 2026-10-30 by Phase 9d Task 8.8c (949a7806b).**

**OPEN, queued 2026-10-21 by the controller; proposed 9d (a flaky gated corpus program).**

# command_redirected_target_waits_elsewhere.rex mismatches on a timed line

The gated corpus run mismatched on one timed line of
`lang/command_redirected_target_waits_elsewhere.rex` on its first run and matched on the rerun,
in Phase 9d Task 6.14 and again in Task 6.15 fix round 3 (report, Concern 2). A gate that passes
on a retry is not a gate. Either make the program's output independent of timing or move it under
`REXX_SWITCH_MODE=sim:SEED` (corpus/sim-gate.tsv).

Seen again by the Task 7.1 round-3 re-review: `guard 0 3 waited 0` in 2 of 3 corpus runs under load, 6 of 6 green standalone.

## Task 8.8c evidence

The witness now prints completion and release properties instead of elapsed-time thresholds. Callback-started controls prevent command startup from satisfying the sleep and semaphore observations, and an external readiness event establishes guard ownership before command launch. Ordinary, collection-stress and seeded simulation runs agree with the retained oracle transcript, while sleep/wait omissions and an unguarded holder break the observation. The related 2026-10-26 redirected-target item has a separate closure for the same repair.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
