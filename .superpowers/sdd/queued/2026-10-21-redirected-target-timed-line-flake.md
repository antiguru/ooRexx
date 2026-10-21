**OPEN, queued 2026-10-21 by the controller; proposed 9d (a flaky gated corpus program).**

# command_redirected_target_waits_elsewhere.rex mismatches on a timed line

The gated corpus run mismatched on one timed line of
`lang/command_redirected_target_waits_elsewhere.rex` on its first run and matched on the rerun,
in Phase 9d Task 6.14 and again in Task 6.15 fix round 3 (report, Concern 2). A gate that passes
on a retry is not a gate. Either make the program's output independent of timing or move it under
`REXX_SWITCH_MODE=sim:SEED` (corpus/sim-gate.tsv).
