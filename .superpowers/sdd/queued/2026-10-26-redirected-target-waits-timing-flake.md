**CLOSED 2026-10-30 by Phase 9d Task 8.8c (949a7806b).**

**OPEN, queued 2026-10-26 by the controller; proposed 9d (Task 8.8).**

# collect_stress the_l0_subset flakes on command_redirected_target_waits_elsewhere

Found by Phase 9d Task 8.3 fix round 2 (task-8.3-report.md, concern 4): the program's
`time('e') > 0.9` check failed in 2 of 3 runs at 3e6dc0426 and 3 of 3 at efe8d3f40 under
collect_stress (which programs failed the first runs was not captured). Related to the queued
redirected-target flake row. Open: make the wait observable without a wall-clock threshold.

## Task 8.8c evidence

The shared redirected-target witness now uses logical wait and guard-release observations. Its focused test runs collection on every allocation, verifies that collections occurred, and preserves command status and all callback lines. Ordinary mode, seeded simulation policies and oracle runs retain the same transcript, and explicit omission controls show each wait remains observable. The program remains in phase-9.txt and the collector inclusion, with the related 2026-10-21 item closed separately.

Evidence: `.superpowers/sdd/2026-10-18-phase-9d/task-8.8c-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/timing/`.
