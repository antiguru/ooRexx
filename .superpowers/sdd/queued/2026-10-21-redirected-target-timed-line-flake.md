**CLOSED 2026-10-30 by Phase 9d Task 8.8c (d7581cc4a).**

**OPEN, queued 2026-10-21 by the controller; proposed 9d (a flaky gated corpus program).**

# command_redirected_target_waits_elsewhere.rex mismatches on a timed line

The gated corpus run mismatched on one timed line of
`lang/command_redirected_target_waits_elsewhere.rex` on its first run and matched on the rerun,
in Phase 9d Task 6.14 and again in Task 6.15 fix round 3 (report, Concern 2). A gate that passes
on a retry is not a gate. Either make the program's output independent of timing or move it under
`REXX_SWITCH_MODE=sim:SEED` (corpus/sim-gate.tsv).

Seen again by the Task 7.1 round-3 re-review: `guard 0 3 waited 0` in 2 of 3 corpus runs under load, 6 of 6 green standalone.

## Task 8.8c evidence

The witness now records callback completion only after SysSleep returns, without consulting a competing alarm. Its command transcript proves continuation and complete output across the unrelated wait, while a deterministic virtual-clock control separately distinguishes the actual sleep from its omission. A notification held on an explicit semaphore exposes the earlier race and cannot change the corrected callback result, with semaphore and guard omission controls preserved. Ordinary, collection-stress, seeded simulation and oracle transcript checks pass, and the related 2026-10-26 item retains a separate closure.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
